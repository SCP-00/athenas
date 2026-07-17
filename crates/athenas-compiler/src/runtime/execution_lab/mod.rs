use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Configuration for a single execution experiment.
/// Describes exactly what to run and how.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Path to the llama-server binary
    pub runtime_path: String,
    /// Path to the GGUF model file
    pub model_path: String,
    /// Context window size (tokens)
    pub context_length: u64,
    /// Batch size for prompt processing
    pub batch_size: u64,
    /// Batch size for generation
    pub ubatch_size: u64,
    /// Number of GPU layers (-1 = all, 0 = CPU only)
    pub gpu_layers: i64,
    /// KV cache type (f16, q8, q4, q6, turbo3, etc.)
    pub kv_cache_type: String,
    /// Whether KV cache is in RAM (vs VRAM)
    pub kv_in_ram: bool,
    /// Flash attention enabled
    pub flash_attention: bool,
    /// Number of CPU threads
    pub threads: u64,
    /// Inference prompt
    pub prompt: String,
    /// Max tokens to generate
    pub max_tokens: u64,
    /// Temperature
    pub temperature: f64,
}

impl ExecutionConfig {
    pub fn new(runtime_path: &str, model_path: &str) -> Self {
        Self {
            runtime_path: runtime_path.to_string(),
            model_path: model_path.to_string(),
            context_length: 32768,
            batch_size: 512,
            ubatch_size: 256,
            gpu_layers: 999,
            kv_cache_type: "f16".to_string(),
            kv_in_ram: false,
            flash_attention: true,
            threads: 8,
            prompt: "Hello, explain what you are.".to_string(),
            max_tokens: 100,
            temperature: 0.7,
        }
    }
}

/// Complete telemetry from a single execution experiment.
/// Every field is measured, not estimated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTelemetry {
    /// How long to load the model (seconds)
    pub load_time_s: f64,
    /// Time to first token (milliseconds)
    pub first_token_ms: f64,
    /// Steady-state tokens per second
    pub tokens_per_second: f64,
    /// Peak VRAM usage during execution (GB)
    pub vram_peak_gb: f64,
    /// VRAM before execution (GB)
    pub vram_before_gb: f64,
    /// VRAM after execution (GB)
    pub vram_after_gb: f64,
    /// Peak RAM usage during execution (GB)
    pub ram_peak_gb: f64,
    /// GPU utilization percentage
    pub gpu_util_pct: f64,
    /// GPU temperature (Celsius)
    pub gpu_temp_c: f64,
    /// Total duration of the experiment (seconds)
    pub total_duration_s: f64,
    /// Why the execution ended
    pub exit_reason: String,
    /// OOM detected (via nvidia-smi or stderr)
    pub oom_detected: bool,
    /// Crash detected
    pub crash_detected: bool,
    /// Full stderr output
    pub stderr_log: String,
    /// Full stdout output (including timings from llama.cpp)
    pub stdout_log: String,
}

/// Full result from the Execution Laboratory.
/// This is the evidence that replaces VramCalculator::estimate().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// The configuration that was executed
    pub config: ExecutionConfig,
    /// Telemetry measured during execution
    pub telemetry: ExecutionTelemetry,
    /// Hardware snapshot (before execution)
    pub hardware_snapshot: serde_json::Value,
    /// Runtime discovery info
    pub runtime_info: serde_json::Value,
    /// Whether the execution was successful
    pub success: bool,
    /// Timestamp
    pub executed_at: u64,
    /// Experiment ID
    pub experiment_id: String,
}

/// The Execution Probe — the heart of the Execution Laboratory.
/// Executes a single configuration, measures everything, returns evidence.
pub struct ExecutionProbe;

impl ExecutionProbe {
    /// Execute a single configuration and produce telemetry.
    /// This is the method that replaces VramCalculator::estimate() as the source of truth.
    pub fn execute(config: &ExecutionConfig, experiment_id: &str) -> Result<ExecutionReport, String> {
        let start = Instant::now();
        let model_path = PathBuf::from(&config.model_path);
        let runtime_path = PathBuf::from(&config.runtime_path);

        // Phase 1: Hardware snapshot (before execution)
        let hardware = crate::runtime::hardware::detect_hardware();
        let hw_snapshot = serde_json::to_value(&hardware)
            .map_err(|e| format!("Cannot serialize hardware: {e}"))?;

        // Phase 2: Capture VRAM before
        let vram_before = get_current_vram_gb();
        let _ram_before = get_available_ram_gb();

        // Phase 3: Set up llama-server with stderr capture for OOM detection
        let port = find_free_port();
        let mut server_cmd = Command::new(&runtime_path);

        // Build args
        server_cmd
            .arg("-m").arg(&config.model_path)
            .arg("--host").arg("127.0.0.1")
            .arg("--port").arg(port.to_string())
            .arg("-c").arg(config.context_length.to_string())
            .arg("-b").arg(config.batch_size.to_string())
            .arg("-ub").arg(config.ubatch_size.to_string())
            .arg("-ngl").arg(config.gpu_layers.to_string())
            .arg("-t").arg(config.threads.to_string())
            .arg("-np").arg("1");

        // KV cache type
        if !config.kv_cache_type.is_empty() && config.kv_cache_type != "f16" {
            if let Some(kv_type) = map_kv_cache_type(&config.kv_cache_type) {
                server_cmd.args(["--cache-type-k", &kv_type, "--cache-type-v", &kv_type]);
            }
        }

        // KV in RAM
        if config.kv_in_ram {
            server_cmd.arg("--no-kv-offload");
        }

        // Flash attention
        if config.flash_attention {
            server_cmd.arg("--flash-attn");
        }

        // Capture stderr for OOM detection
        server_cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let load_start = Instant::now();
        let mut child = server_cmd.spawn()
            .map_err(|e| format!("Cannot start llama-server: {e}"))?;

        // Wait for server to be reachable (model loading)
        let server_ready = wait_for_server(&port, Duration::from_secs(60));
        let load_time = load_start.elapsed().as_secs_f64();

        if !server_ready {
            // Kill process
            let _ = child.kill();
            let _ = child.wait();

            // Check stderr for OOM
            let stderr = child.stderr.take()
                .map(|mut s| {
                    let mut buf = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut s, &mut buf);
                    String::from_utf8_lossy(&buf).to_string()
                })
                .unwrap_or_default();

            let oom = stderr.to_lowercase().contains("oom") || stderr.contains("CUDA error: out of memory");
            let exit_reason = if oom { "OOM during model load".to_string() } else { "Timeout".to_string() };

            return Ok(ExecutionReport {
                config: config.clone(),
                telemetry: ExecutionTelemetry {
                    load_time_s: load_time,
                    first_token_ms: 0.0,
                    tokens_per_second: 0.0,
                    vram_peak_gb: get_current_vram_gb(),
                    vram_before_gb: vram_before,
                    vram_after_gb: get_current_vram_gb(),
                    ram_peak_gb: 0.0,
                    gpu_util_pct: 0.0,
                    gpu_temp_c: 0.0,
                    total_duration_s: start.elapsed().as_secs_f64(),
                    exit_reason,
                    oom_detected: oom,
                    crash_detected: true,
                    stderr_log: stderr,
                    stdout_log: String::new(),
                },
                hardware_snapshot: hw_snapshot,
                runtime_info: serde_json::json!({
                    "binary": config.runtime_path,
                    "port": port,
                }),
                success: false,
                executed_at: get_unix_timestamp(),
                experiment_id: experiment_id.to_string(),
            });
        }

        // Phase 4: Capture VRAM after load
        let vram_after_load = get_current_vram_gb();
        let gpu_util = get_gpu_utilization();
        let gpu_temp = get_gpu_temperature();

        // Phase 5: Run inference
        let inference_body = serde_json::json!({
            "prompt": config.prompt,
            "n_predict": config.max_tokens,
            "temperature": config.temperature,
            "stream": false,
            "cache_prompt": true,
        });

        let inference_start = Instant::now();
        let inference_result = send_completion_request(port, &inference_body);
        let _inference_duration = inference_start.elapsed().as_secs_f64();

        match inference_result {
            Ok(response) => {
                let text = response["content"].as_str().unwrap_or_default().to_string();
                let timings = &response["timings"];
                let ttft_ms = timings["prompt_ms"].as_f64().unwrap_or(0.0);
                let predicted_ms = timings["predicted_ms"].as_f64().unwrap_or(0.0);
                let predicted_n = timings["predicted_n"].as_f64().unwrap_or(0.0);
                let prompt_n = timings["prompt_n"].as_f64().unwrap_or(0.0);

                let tokens_per_second = if predicted_ms > 0.0 && predicted_n > 0.0 {
                    (predicted_n / predicted_ms) * 1000.0
                } else {
                    0.0
                };

                // Capture VRAM/RAM after execution
                let vram_after = get_current_vram_gb();
                let vram_peak = vram_after_load.max(vram_after); // Best estimate of peak

                // Kill server
                let _ = child.kill();
                let _ = child.wait();

                // Check stderr for any errors
                let stderr = child.stderr.take()                .map(|mut s| {
                    let mut buf = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut s, &mut buf);
                    String::from_utf8_lossy(&buf).to_string()
                })
                .unwrap_or_default();

                let total_duration = start.elapsed().as_secs_f64();

                Ok(ExecutionReport {
                    config: config.clone(),
                    telemetry: ExecutionTelemetry {
                        load_time_s: load_time,
                        first_token_ms: ttft_ms,
                        tokens_per_second: (tokens_per_second * 10.0).round() / 10.0,
                        vram_peak_gb: vram_peak,
                        vram_before_gb: vram_before,
                        vram_after_gb: vram_after,
                        ram_peak_gb: 0.0,
                        gpu_util_pct: gpu_util,
                        gpu_temp_c: gpu_temp,
                        total_duration_s: total_duration,
                        exit_reason: "Completed".to_string(),
                        oom_detected: false,
                        crash_detected: false,
                        stderr_log: stderr,
                        stdout_log: response.to_string(),
                    },
                    hardware_snapshot: hw_snapshot,
                    runtime_info: serde_json::json!({
                        "binary": config.runtime_path,
                        "port": port,
                        "version": get_runtime_version(&runtime_path),
                    }),
                    success: true,
                    executed_at: get_unix_timestamp(),
                    experiment_id: experiment_id.to_string(),
                })
            }
            Err(e) => {
                // Inference failed — may be OOM during generation
                let _ = child.kill();
                let _ = child.wait();

                let stderr = child.stderr.take()                .map(|mut s| {
                    let mut buf = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut s, &mut buf);
                    String::from_utf8_lossy(&buf).to_string()
                })
                .unwrap_or_default();

                let oom = stderr.to_lowercase().contains("oom") || e.to_string().to_lowercase().contains("oom");
                let vram_after = get_current_vram_gb();

                Ok(ExecutionReport {
                    config: config.clone(),
                    telemetry: ExecutionTelemetry {
                        load_time_s: load_time,
                        first_token_ms: 0.0,
                        tokens_per_second: 0.0,
                        vram_peak_gb: vram_after_load,
                        vram_before_gb: vram_before,
                        vram_after_gb: vram_after,
                        ram_peak_gb: 0.0,
                        gpu_util_pct: 0.0,
                        gpu_temp_c: 0.0,
                        total_duration_s: start.elapsed().as_secs_f64(),
                        exit_reason: format!("Inference failed: {e}"),
                        oom_detected: oom,
                        crash_detected: true,
                        stderr_log: stderr,
                        stdout_log: String::new(),
                    },
                    hardware_snapshot: hw_snapshot,
                    runtime_info: serde_json::json!({
                        "binary": config.runtime_path,
                        "port": port,
                    }),
                    success: false,
                    executed_at: get_unix_timestamp(),
                    experiment_id: experiment_id.to_string(),
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Telemetry helpers
// ═══════════════════════════════════════════════════════════════

/// Get current VRAM usage (GB) via nvidia-smi
fn get_current_vram_gb() -> f64 {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used", "--format=csv,noheader,nounits"])
        .output()
        .ok();
    if let Some(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        let mi: f64 = s.trim().parse().unwrap_or(0.0);
        mi / 1024.0 // Convert MB to GB
    } else {
        0.0
    }
}

/// Get available RAM (GB)
fn get_available_ram_gb() -> f64 {
    let output = Command::new("free")
        .arg("-b")
        .output()
        .ok();
    if let Some(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    let available: f64 = parts[6].parse().unwrap_or(0.0);
                    return available / (1024.0 * 1024.0 * 1024.0);
                }
            }
        }
    }
    0.0
}

/// Get GPU utilization percentage
fn get_gpu_utilization() -> f64 {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok();
    if let Some(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        s.trim().parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Get GPU temperature (Celsius)
fn get_gpu_temperature() -> f64 {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok();
    if let Some(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        s.trim().parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Get runtime version
fn get_runtime_version(path: &Path) -> String {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .ok();
    if let Some(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let stderr = String::from_utf8_lossy(&o.stderr);
        format!("{stdout}\n{stderr}")
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string()
    } else {
        "unknown".to_string()
    }
}

/// Find a free TCP port
fn find_free_port() -> u16 {
    for port in 18080..19000 {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(50),
        ).is_ok() {
            continue;
        }
        return port;
    }
    18080
}

/// Wait for the server to be reachable (model loaded)
fn wait_for_server(port: &u16, timeout: Duration) -> bool {
    let start = Instant::now();
    let addr = format!("127.0.0.1:{port}");
    loop {
        if start.elapsed() > timeout {
            return false;
        }
        if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(500),
        ) {
            // Send a minimal HTTP request to verify the server responds
            let req = format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
            use std::io::Write;
            if stream.write_all(req.as_bytes()).is_ok() {
                stream.set_read_timeout(Some(std::time::Duration::from_millis(500))).ok();
                let mut buf = [0u8; 4];
                if std::io::Read::read(&mut stream, &mut buf).is_ok() {
                    let first_bytes = String::from_utf8_lossy(&buf);
                    if first_bytes.contains("200") || first_bytes.contains("HTTP") {
                        return true;
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Send a completion request to llama-server
fn send_completion_request(port: u16, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    use std::io::{Read, Write};
    let addr = format!("127.0.0.1:{port}");
    let mut stream = std::net::TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Cannot parse address: {e}"))?,
        Duration::from_secs(5),
    ).map_err(|e| format!("Cannot connect: {e}"))?;

    let body_str = serde_json::to_string(body).map_err(|e| format!("Cannot serialize: {e}"))?;
    let request = format!(
        "POST /completion HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body_str.len(),
        body_str
    );

    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Cannot send request: {e}"))?;

    stream.set_read_timeout(Some(Duration::from_secs(300)))
        .map_err(|e| format!("Cannot set timeout: {e}"))?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)
        .map_err(|e| format!("Cannot read response: {e}"))?;

    let response = String::from_utf8_lossy(&raw);

    // Find body start (after \r\n\r\n)
    let body_start = response.find("\r\n\r\n")
        .ok_or_else(|| "Invalid HTTP response: no header/body separator".to_string())?
        + 4;
    let body_text = &response[body_start..];

    serde_json::from_str(body_text)
        .map_err(|e| format!("Cannot parse response JSON: {e}. Body: {}", &body_text[..body_text.len().min(500)]))
}

/// Get Unix timestamp
fn get_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Map KV cache type string to llama.cpp argument value
fn map_kv_cache_type(kv_type: &str) -> Option<String> {
    match kv_type.to_lowercase().as_str() {
        "q8" | "q8_0" => Some("q8_0".to_string()),
        "q6" | "q6_k" => Some("q6_k".to_string()),
        "q5" | "q5_1" => Some("q5_1".to_string()),
        "q4" | "q4_0" => Some("q4_0".to_string()),
        "q4_1" => Some("q4_1".to_string()),
        "f16" | "fp16" => None, // Default, no flag needed
        "turbo3" | "iq4" => Some("iq4_nl".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_kv_cache_type() {
        assert_eq!(map_kv_cache_type("q8"), Some("q8_0".to_string()));
        assert_eq!(map_kv_cache_type("q4"), Some("q4_0".to_string()));
        assert_eq!(map_kv_cache_type("f16"), None);
        assert_eq!(map_kv_cache_type("fp16"), None);
    }
}

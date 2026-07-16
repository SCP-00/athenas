pub mod benchmark;
pub mod benchmarks;
pub mod hardware;
pub mod knowledge;
pub mod knowledge_ir;
pub mod providers;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Configuration for model inference
#[derive(Debug, Clone)]
pub struct InferenceParams {
    pub temperature: f64,
    pub max_tokens: usize,
    pub top_p: f64,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
        }
    }
}

/// Result of a single inference call
#[derive(Debug, Clone, serde::Serialize)]
pub struct InferenceResult {
    pub text: String,
    pub ttft_ms: f64,
    pub tokens_per_second: f64,
    pub total_tokens: usize,
    pub prompt_tokens: usize,
    pub total_duration_ms: f64,
}

/// Rich execution result with full metadata (replaces InferenceResult over time)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    pub inference: InferenceResult,
    pub hardware: hardware::HardwareInfo,
    pub capability: Capability,
    pub model_path: String,
    pub model_info: ModelInfo,
    pub warnings: Vec<String>,
    pub evidence_ref: Option<String>,
}

/// Measurable capability that a model can be benchmarked against
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Capability {
    TextGeneration,
    Coding,
    ToolCalling,
    Translation,
    Reasoning,
    RAG,
    LongContext,
    InstructionFollowing,
}

impl Capability {
    pub fn all() -> &'static [Capability] {
        &[
            Capability::TextGeneration,
            Capability::Coding,
            Capability::ToolCalling,
            Capability::Translation,
            Capability::Reasoning,
            Capability::RAG,
            Capability::LongContext,
            Capability::InstructionFollowing,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Capability::TextGeneration => "text-generation",
            Capability::Coding => "coding",
            Capability::ToolCalling => "tool-calling",
            Capability::Translation => "translation",
            Capability::Reasoning => "reasoning",
            Capability::RAG => "rag",
            Capability::LongContext => "long-context",
            Capability::InstructionFollowing => "instruction-following",
        }
    }

    pub fn from_name(s: &str) -> Option<Capability> {
        Capability::all().iter().find(|c| c.name() == s).copied()
    }
}

/// A model descriptor used for documentation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub path: String,
    pub parameters_b: f64,
    pub quantization: String,
    pub architecture: String,
    pub context_length: usize,
    pub hardware: hardware::HardwareInfo,
}

/// Runtime abstraction — implemented per backend
pub trait Runtime {
    fn name(&self) -> &str;
    fn load_model(&mut self, model_path: &Path) -> anyhow::Result<()>;
    fn complete(&self, prompt: &str, params: &InferenceParams) -> anyhow::Result<InferenceResult>;
    fn unload(&mut self) -> anyhow::Result<()>;
}

// ---------------------------------------------------------------------------
// LlamaServerRuntime — wraps `llama-server` as a subprocess + HTTP client
// ---------------------------------------------------------------------------

pub struct LlamaServerRuntime {
    server_binary: PathBuf,
    model_path: Option<PathBuf>,
    server_process: Option<Child>,
    port: u16,
}

impl LlamaServerRuntime {
    pub fn new() -> Self {
        Self {
            server_binary: PathBuf::from("llama-server"),
            model_path: None,
            server_process: None,
            port: 0,
        }
    }

    pub fn with_server_path(mut self, path: PathBuf) -> Self {
        self.server_binary = path;
        self
    }

    fn find_free_port() -> u16 {
        for port in 18080..19000 {
            if TcpStream::connect_timeout(
                &format!("127.0.0.1:{port}").parse().unwrap(),
                Duration::from_millis(50),
            )
            .is_ok()
            {
                continue;
            }
            return port;
        }
        18080
    }

    fn http_request(&self, method: &str, path: &str, body: Option<&str>) -> anyhow::Result<String> {
        let addr = format!("127.0.0.1:{}", self.port);
        let mut stream = TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(5))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .ok();
        stream
            .set_read_timeout(Some(Duration::from_secs(300)))
            .ok();

        let request = match body {
            Some(b) => format!(
                "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{b}",
                port = self.port,
                len = b.len()
            ),
            None => format!(
                "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n",
                port = self.port
            ),
        };

        stream.write_all(request.as_bytes())?;

        let mut raw = Vec::new();
        stream.read_to_end(&mut raw)?;
        let response = String::from_utf8_lossy(&raw);

        // Check HTTP status line
        if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
            let status_line = response.lines().next().unwrap_or("unknown");
            let body_start = response
                .find("\r\n\r\n")
                .map(|i| i + 4)
                .unwrap_or(0);
            let body = &response[body_start..response.len().min(body_start + 300)];
            anyhow::bail!(
                "llama-server returned {} — {}",
                status_line,
                body.trim()
            );
        }

        // Split header from body
        let body_start = response
            .find("\r\n\r\n")
            .ok_or_else(|| anyhow::anyhow!("Invalid HTTP response: no header/body separator"))?
            + 4;

        Ok(response[body_start..].to_string())
    }

    /// Phase 1: wait until the HTTP server is reachable
    fn wait_for_server_reachable(&self, timeout_secs: u64) -> anyhow::Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "llama-server did not start within {timeout_secs}s (port {})",
                    self.port
                );
            }

            // Probe common endpoints
            let endpoints = ["/health", "/infusion", "/tokenize", "/"];
            for endpoint in &endpoints {
                if self.http_request("GET", endpoint, None).is_ok() {
                    return Ok(());
                }
            }
            // Fallback: TCP
            if TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", self.port).parse()?,
                Duration::from_millis(200),
            )
            .is_ok()
            {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Phase 2: wait until the model is fully loaded (not returning 503)
    fn wait_for_model_ready(&self, timeout_secs: u64) -> anyhow::Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let probe = serde_json::json!({"prompt": "a", "n_predict": 1, "stream": false});
        let probe_str = probe.to_string();

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Model did not finish loading within {timeout_secs}s (port {})",
                    self.port
                );
            }

            match self.http_request("POST", "/completion", Some(&probe_str)) {
                Ok(body) => {
                    // Check if the response is actual completion (not a 503 wrapped as 200)
                    if body.contains("\"content\"") {
                        return Ok(());
                    }
                    // Still loading
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("503") || msg.contains("Loading model") {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }
}

impl Runtime for LlamaServerRuntime {
    fn name(&self) -> &str {
        "llama.cpp (server)"
    }

    fn load_model(&mut self, model_path: &Path) -> anyhow::Result<()> {
        self.model_path = Some(model_path.to_path_buf());
        self.port = Self::find_free_port();

        let model_str = model_path.to_string_lossy();

        println!("  🚀 Starting llama-server on port {}...", self.port);
        println!("  📦 Model: {}", model_str);

        // Use the configured binary path (respects --server-path)
        let binary = &self.server_binary;

        let child = Command::new(binary)
            .arg("-m")
            .arg(model_str.as_ref())
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.port.to_string())
            .arg("-np")
            .arg("1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to start {}: {e}\n\
                     Make sure it's installed or use --server-path to specify the location.\n\
                     Download from: https://github.com/ggml-org/llama.cpp",
                    binary.display()
                )
            })?;

        self.server_process = Some(child);
        self.wait_for_server_reachable(30)?;
        println!("  🔌 Server reachable, waiting for model...");
        self.wait_for_model_ready(120)?;
        println!("  ✅ Model ready");

        Ok(())
    }

    fn complete(
        &self,
        prompt: &str,
        params: &InferenceParams,
    ) -> anyhow::Result<InferenceResult> {
        let body = serde_json::json!({
            "prompt": prompt,
            "n_predict": params.max_tokens,
            "temperature": params.temperature,
            "top_p": params.top_p,
            "stream": false,
            "cache_prompt": true,
        });

        let body_str = serde_json::to_string(&body)?;
        let start = Instant::now();
        let response_body = self.http_request("POST", "/completion", Some(&body_str))?;
        let wall_clock_ms = start.elapsed().as_secs_f64() * 1000.0;

        let result: serde_json::Value = serde_json::from_str(&response_body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse llama-server response: {e}\nBody (first 500 chars): {body}",
                body = &response_body[..response_body.len().min(500)]
            )
        })?;

        let text = result["content"].as_str().unwrap_or_default().to_string();

        // llama.cpp timings (available since late 2024)
        let timings = &result["timings"];
        let ttft_ms = timings["prompt_ms"].as_f64().unwrap_or(0.0);
        let predicted_ms = timings["predicted_ms"].as_f64().unwrap_or(0.0);
        let predicted_n = timings["predicted_n"].as_f64().unwrap_or(0.0);
        let prompt_n = timings["prompt_n"].as_f64().unwrap_or(0.0);

        let tokens_per_second = if predicted_ms > 0.0 && predicted_n > 0.0 {
            (predicted_n / predicted_ms) * 1000.0
        } else if ttft_ms > 0.0 && !text.is_empty() {
            // Fallback: estimate from wall clock
            let gen_time = wall_clock_ms - ttft_ms;
            if gen_time > 0.0 {
                (text.split_whitespace().count() as f64 / gen_time) * 1000.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(InferenceResult {
            text,
            ttft_ms,
            tokens_per_second: (tokens_per_second * 10.0).round() / 10.0,
            total_tokens: predicted_n as usize,
            prompt_tokens: prompt_n as usize,
            total_duration_ms: wall_clock_ms,
        })
    }

    fn unload(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.server_process.take() {
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }
}

impl Drop for LlamaServerRuntime {
    fn drop(&mut self) {
        self.unload().ok();
    }
}

// ---------------------------------------------------------------------------
// Helper: detect GGUF models in common directories
// ---------------------------------------------------------------------------

pub fn find_model(override_path: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(p) = override_path {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
        anyhow::bail!("Model file not found: {}", p.display());
    }

    // Build candidate directories (portable: uses $HOME if available)
    let home = std::env::var("HOME").unwrap_or_default();
    let mut candidates = vec![
        "models".to_string(),
        "../models".to_string(),
        "/models".to_string(),
        "/usr/share/models".to_string(),
    ];
    if !home.is_empty() {
        candidates.push(format!("{home}/models"));
        // also check common paths the user might have
        candidates.push(format!("{home}/Models"));
    }

    for dir in &candidates {
        let d = Path::new(dir);
        if d.is_dir() {
            if let Ok(entries) = std::fs::read_dir(d) {
                let mut gguf: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "gguf")
                            .unwrap_or(false)
                    })
                    .map(|e| e.path())
                    .collect();
                gguf.sort();
                if !gguf.is_empty() {
                    // Prefer Q4_K_M quant (good quality/speed balance)
                    if let Some(q4) = gguf.iter().find(|p| {
                        p.to_string_lossy()
                            .to_lowercase()
                            .contains("q4_k_m")
                    }) {
                        return Ok(q4.clone());
                    }
                    return Ok(gguf[0].clone());
                }
            }
        }
    }

    anyhow::bail!(
        "No GGUF model found. Use --model <path> to specify.\n\
         Searched: {:?}",
        candidates
    )
}

/// Convenience wrapper: run inference with InferenceParams built from max_tokens
pub fn run_benchmark(rt: &impl Runtime, prompt: &str, max_tokens: usize) -> anyhow::Result<InferenceResult> {
    let params = InferenceParams {
        max_tokens,
        ..Default::default()
    };
    rt.complete(prompt, &params)
}

/// Find ALL GGUF models in common directories (recursive, for doctor/certify)
pub fn find_all_models() -> Vec<ModelInfo> {
    let hw = hardware::detect_hardware();
    let home = std::env::var("HOME").unwrap_or_default();
    let mut candidates = vec![
        "models".to_string(),
        "../models".to_string(),
        "/models".to_string(),
        "/usr/share/models".to_string(),
    ];
    if !home.is_empty() {
        candidates.push(format!("{home}/models"));
        candidates.push(format!("{home}/Models"));
    }

    let mut seen = std::collections::HashSet::new();
    let mut models = Vec::new();
    for dir in &candidates {
        let d = Path::new(dir);
        if !d.is_dir() {
            continue;
        }
        // Walk recursively to find all .gguf files
        let walker = walkdir::WalkDir::new(d)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target"
            });
        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                // Canonicalize to deduplicate (e.g., ../models/foo == /home/user/models/foo)
                let canonical = path.canonicalize().ok();
                let key = canonical.as_deref().unwrap_or(path);
                if !seen.insert(key.to_string_lossy().to_string()) {
                    continue; // Already added
                }
                // Skip vision projection files (mmproj) — they're not standalone LLMs
                let fname = path.file_stem().unwrap_or_default().to_string_lossy();
                if fname.contains("mmproj") || fname.contains("MMPROJ") {
                    continue;
                }
                models.push(infer_model_info(path, Some(&hw)));
            }
        }
    }
    // Sort by parameter count (largest first)
    models.sort_by(|a, b| b.parameters_b.partial_cmp(&a.parameters_b).unwrap_or(std::cmp::Ordering::Equal));
    models
}

/// Infer basic metadata from a model filename
pub fn infer_model_info(path: &Path, hw: Option<&hardware::HardwareInfo>) -> ModelInfo {
    let fname = path.file_stem().unwrap_or_default().to_string_lossy();
    let full = path.to_string_lossy();

    let params = if full.contains("27b") || full.contains("27B") {
        27.0
    } else if full.contains("9b") || full.contains("9B") {
        9.0
    } else if full.contains("4b") || full.contains("4B") {
        4.0
    } else if full.contains("7b") || full.contains("7B") {
        7.0
    } else if full.contains("8b") || full.contains("8B") {
        8.0
    } else {
        0.0
    };

    let quant = if full.contains("q4_k_m") {
        "Q4_K_M"
    } else if full.contains("q4_0") {
        "Q4_0"
    } else if full.contains("q8_0") {
        "Q8_0"
    } else if full.contains("q2_k") {
        "Q2_K"
    } else if full.contains("q3_k") {
        "Q3_K"
    } else if full.contains("q5_k") {
        "Q5_K"
    } else if full.contains("q6_k") {
        "Q6_K"
    } else if full.contains("q1_0") || full.contains("q1_") {
        "Q1_0"
    } else {
        "unknown"
    };

    // Model ID from the filename
    let safe_name = fname
        .replace(|c: char| !c.is_alphanumeric(), "-")
        .trim_matches('-')
        .to_uppercase();
    let truncated = if safe_name.len() > 20 {
        &safe_name[..20]
    } else {
        &safe_name[..]
    };
    let id = format!("MODEL-{truncated}");

    ModelInfo {
        id,
        path: path.to_string_lossy().to_string(),
        parameters_b: params,
        quantization: quant.to_string(),
        architecture: "Transformer".to_string(),
        context_length: 32768,
        hardware: hw.cloned().unwrap_or_else(hardware::detect_hardware),
    }
}

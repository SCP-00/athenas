use std::path::Path;
use std::process::Command;

/// Hardware profile — auto-detected, never manually specified
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Vec<GpuInfo>,
    pub os: OsInfo,
    pub kernel: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub available_gb: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub model: String,
    pub vram_gb: f64,
    pub driver_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

/// Detect all hardware information
pub fn detect_hardware() -> HardwareInfo {
    HardwareInfo {
        cpu: detect_cpu(),
        memory: detect_memory(),
        gpu: detect_gpu(),
        os: detect_os(),
        kernel: detect_kernel(),
    }
}

fn detect_cpu() -> CpuInfo {
    let model = read_first_match("/proc/cpuinfo", "model name")
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let cores = read_first_match("/proc/cpuinfo", "cpu cores")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let threads = count_cpu_threads();
    CpuInfo { model, cores, threads }
}

fn count_cpu_threads() -> usize {
    // Count "processor" lines in /proc/cpuinfo
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        content
            .lines()
            .filter(|l| l.starts_with("processor"))
            .count()
    } else {
        0
    }
}

fn detect_memory() -> MemoryInfo {
    let total_kb = read_first_match("/proc/meminfo", "MemTotal")
        .and_then(|s| parse_kb_value(&s))
        .unwrap_or(0.0);
    let available_kb = read_first_match("/proc/meminfo", "MemAvailable")
        .and_then(|s| parse_kb_value(&s))
        .unwrap_or(0.0);
    MemoryInfo {
        total_gb: (total_kb / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        available_gb: (available_kb / 1024.0 / 1024.0 * 10.0).round() / 10.0,
    }
}

fn parse_kb_value(s: &str) -> Option<f64> {
    // Parse "123456 kB" → 123456.0
    let num: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse::<f64>().ok()
}

fn detect_gpu() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    // NVIDIA via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let model = parts[0].to_string();
                let vram_mb: f64 = parts[1].parse().unwrap_or(0.0);
                let driver = parts[2].to_string();
                gpus.push(GpuInfo {
                    model,
                    vram_gb: (vram_mb / 1024.0 * 10.0).round() / 10.0,
                    driver_version: driver,
                });
            }
        }
    }

    // AMD via rocm-smi (if available)
    if gpus.is_empty() {
        if let Ok(output) = Command::new("rocm-smi")
            .args(["--showproductname", "--csv"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                if !line.trim().is_empty() {
                    gpus.push(GpuInfo {
                        model: line.to_string(),
                        vram_gb: 0.0, // rocm-smi doesn't easily give VRAM in CSV
                        driver_version: "ROCm".to_string(),
                    });
                }
            }
        }
    }

    // Apple Metal via system_profiler (macOS only)
    if gpus.is_empty() && cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Chipset Model") {
                    let model = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    gpus.push(GpuInfo {
                        model,
                        vram_gb: 0.0,
                        driver_version: "Apple Metal".to_string(),
                    });
                }
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            model: "Unknown (no GPU detected)".to_string(),
            vram_gb: 0.0,
            driver_version: "N/A".to_string(),
        });
    }

    gpus
}

fn detect_os() -> OsInfo {
    let name = if cfg!(target_os = "linux") {
        read_first_match("/etc/os-release", "PRETTY_NAME")
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_else(|| "Linux".to_string())
    } else if cfg!(target_os = "macos") {
        Command::new("sw_vers")
            .arg("-productName")
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "macOS".to_string())
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        std::env::consts::OS.to_string()
    };

    let version = if cfg!(target_os = "linux") {
        read_first_match("/etc/os-release", "VERSION_ID")
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_default()
    } else if cfg!(target_os = "macos") {
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    OsInfo {
        name,
        version,
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn detect_kernel() -> String {
    std::fs::read_to_string("/proc/version")
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        })
}

fn read_first_match(path: &str, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(Path::new(path)).ok()?;
    for line in content.lines() {
        // Skip comments
        if line.trim().starts_with('#') {
            continue;
        }
        if let Some(value) = line.strip_prefix(key) {
            // value is like ": Model Name   "
            let value = value.trim_start_matches(':').trim();
            return Some(value.to_string());
        }
        // Also try "key=value" format (os-release style: KEY="VALUE" or KEY=VALUE)
        let eq = format!("{key}=");
        if let Some(value) = line.strip_prefix(&eq) {
            let cleaned = value.trim().trim_matches('"');
            return Some(cleaned.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardware() {
        let hw = detect_hardware();
        // Should at least detect something
        assert!(!hw.cpu.model.is_empty(), "CPU model should not be empty");
        assert!(hw.memory.total_gb > 0.0, "Total RAM should be > 0");
        assert!(!hw.os.name.is_empty(), "OS name should not be empty");
        assert!(!hw.kernel.is_empty(), "Kernel should not be empty");
    }

    #[test]
    fn test_cpu_detection() {
        let cpu = detect_cpu();
        assert!(cpu.threads >= cpu.cores, "Threads should be >= cores");
    }

    #[test]
    fn test_gpu_detection() {
        let gpus = detect_gpu();
        // GPU might or might not be present, but should not panic
        assert!(!gpus.is_empty(), "Should have at least one GPU entry");
    }
}

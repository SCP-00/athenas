/// Runtime Fingerprint — Forensic identity of a runtime binary and its ecosystem.
///
/// Replaces RuntimeIdentity. The runtime is NOT a single binary — it's
/// an ecosystem of executable + shared libraries + dependencies.
///
/// Each fingerprint includes:
/// - SHA256 of the executable and all .so libraries
/// - BuildID from ELF notes
/// - Full ldd dependency graph
/// - Binary size, file type, modification date
/// - Source repository, commit, version

pub mod capability;
pub mod normalization;
pub mod validation;
pub mod experiment_validation;
pub mod evidence_store;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Complete forensic fingerprint of a runtime installation.
/// Captures the executable, all shared libraries, and the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFingerprint {
    /// Runtime family (e.g., "llama.cpp")
    pub family: String,
    /// Variant / fork name (e.g., "official", "turboquant", "bonsai")
    pub variant: String,
    /// Display name
    pub display_name: String,
    /// Canonical path to the executable
    pub executable_path: String,
    /// Executable SHA256
    pub executable_sha256: String,
    /// Executable BuildID (from ELF .note.gnu.build-id)
    pub executable_build_id: Option<String>,
    /// Executable size in bytes
    pub executable_size_bytes: u64,
    /// File type (from `file` command)
    pub file_type: String,
    /// Modification timestamp
    pub executable_modified_at: String,
    /// Version string (from --version)
    pub version: Option<String>,
    /// Git commit hash (from --version or build info)
    pub commit: Option<String>,
    /// Compiler info (from the binary or build metadata)
    pub compiler: Option<String>,
    /// CUDA version detected
    pub cuda_version: Option<String>,
    /// Flash Attention: declared support
    pub supports_flash_attention: bool,
    /// Turbo3 / KV quant: declared support
    pub supports_turbo3: bool,
    /// Bonsai: declared support
    pub supports_bonsai: bool,
    /// ISWA: declared support
    pub supports_iswa: bool,
    /// Vision: declared support
    pub supports_vision: bool,
    /// Embeddings: declared support
    pub supports_embeddings: bool,
    /// All discovered shared libraries
    pub libraries: Vec<LibraryFingerprint>,
    /// Full ldd output (tabulated)
    pub ldd_entries: Vec<LddEntry>,
    /// Link-time RPATH
    pub rpath: Option<String>,
    /// Build directory path (if known)
    pub build_directory: Option<String>,
    /// Source repository URL (if known)
    pub repository: Option<String>,
}

/// Fingerprint of a single shared library in the runtime ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFingerprint {
    /// Library name (e.g., "libllama-server-impl.so")
    pub name: String,
    /// Full path
    pub path: String,
    /// SHA256 hash
    pub sha256: String,
    /// BuildID (from ELF notes)
    pub build_id: Option<String>,
    /// Size in bytes
    pub size_bytes: u64,
}

/// A single ldd entry (dependency) in the runtime's library graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LddEntry {
    /// Library name (e.g., "libllama-server-impl.so")
    pub library: String,
    /// Resolved path (or "not found" if missing)
    pub resolved_path: String,
    /// Load address (e.g., "0x7f...")
    pub load_address: String,
}

// ═══════════════════════════════════════════════════════════════
// Fingerprint Engine
// ═══════════════════════════════════════════════════════════════

/// Generate a complete RuntimeFingerprint for a given binary path.
///
/// This is the main entry point. It:
/// 1. Hashes the executable (SHA256)
/// 2. Extracts BuildID from ELF
/// 3. Runs ldd to discover all dependencies
/// 4. Hashes all discovered .so files
/// 5. Extracts version, commit, compiler info
/// 6. Detects capabilities from --help
pub fn fingerprint_runtime(path: &Path) -> RuntimeFingerprint {
    let fname = path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Determine family and variant from path and name
    let path_lower = path.to_string_lossy().to_lowercase();
    let (family, variant, display_name) = if path_lower.contains("prism") || path_lower.contains("bonsai") {
        ("llama.cpp".to_string(), "bonsai".to_string(), "PrismML Bonsai".to_string())
    } else if path_lower.contains("turboquant") {
        ("llama.cpp".to_string(), "turboquant".to_string(), "TurboQuant".to_string())
    } else if fname.contains("llama") {
        ("llama.cpp".to_string(), "official".to_string(), "llama.cpp Official".to_string())
    } else if fname.contains("ollama") {
        ("ollama".to_string(), "official".to_string(), "Ollama".to_string())
    } else {
        ("unknown".to_string(), "unknown".to_string(), fname.clone())
    };

    // Executable info
    let executable_sha256 = sha256_of_file(path).unwrap_or_default();
    let executable_build_id = extract_build_id(path);
    let executable_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let file_type = get_file_type(path);
    let executable_modified_at = std::fs::metadata(path)
        .and_then(|m| m.modified().map(|t| {
            use std::time::SystemTime;
            let d = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            format!("{}", d.as_secs())
        }))
        .unwrap_or_default();

    // Version and build info
    let version = get_version(path);
    let commit = extract_commit(&version);
    let compiler = extract_compiler(path);

    // ldd analysis
    let ldd_entries = run_ldd(path);
    let rpath = extract_rpath(path);

    // Discover and hash all .so libraries in the same directory
    let libraries = discover_libraries(path);

    // Detect capabilities from --help
    let help_text = get_help_text(path);
    let help_lower = help_text.to_lowercase();
    let supports_flash_attention = help_lower.contains("flash") || help_lower.contains("flash-attn");
    let supports_turbo3 = path_lower.contains("turboquant") || help_lower.contains("turbo3")
        || help_lower.contains("cache-type-k");
    let supports_bonsai = path_lower.contains("prism") || path_lower.contains("bonsai")
        || help_lower.contains("bonsai") || help_lower.contains("memory-hybrid")
        || help_lower.contains("memory-recurrent");
    let supports_iswa = help_lower.contains("iswa") || help_lower.contains("importance-based");
    let supports_vision = help_lower.contains("mmproj") || help_lower.contains("multimodal")
        || help_lower.contains("llava") || help_lower.contains("qwen2vl");
    let supports_embeddings = help_lower.contains("embed") || help_lower.contains("--embd");

    // CUDA version from system
    let cuda_version = get_cuda_version();

    RuntimeFingerprint {
        family,
        variant,
        display_name,
        executable_path: path.to_string_lossy().to_string(),
        executable_sha256,
        executable_build_id,
        executable_size_bytes,
        file_type,
        executable_modified_at,
        version,
        commit,
        compiler,
        cuda_version,
        supports_flash_attention,
        supports_turbo3,
        supports_bonsai,
        supports_iswa,
        supports_vision,
        supports_embeddings,
        libraries,
        ldd_entries,
        rpath,
        build_directory: infer_build_directory(path),
        repository: None,
    }
}

/// Generate fingerprints for all runtimes on the system.
pub fn fingerprint_all_runtimes() -> Vec<RuntimeFingerprint> {
    let runtimes = crate::runtime::runtime_discovery::RuntimeProber::probe_all();
    let mut fingerprints = Vec::new();
    for rt in &runtimes {
        let path = Path::new(&rt.binary_path);
        if path.exists() {
            fingerprints.push(fingerprint_runtime(path));
        }
    }
    fingerprints
}

// ═══════════════════════════════════════════════════════════════
// Private helpers
// ═══════════════════════════════════════════════════════════════

/// SHA256 hash of a file
fn sha256_of_file(path: &Path) -> Option<String> {
    use std::fs::File;
    use std::io::Read;
    use sha2::{Sha256, Digest};
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let hash = hasher.finalize();
    Some(format!("{:x}", hash))
}

/// Extract BuildID from ELF binary (read .note.gnu.build-id section)
fn extract_build_id(path: &Path) -> Option<String> {
    // Use readelf to extract BuildID
    let output = Command::new("readelf")
        .args(["-n", &path.to_string_lossy()])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Look for "Build ID: <hex>"
    for line in stdout.lines() {
        if let Some(bid) = line.strip_prefix("    Build ID: ") {
            return Some(bid.trim().to_string());
        }
        // Alternative format: "Build ID: <hex>"
        if let Some(bid) = line.strip_prefix("Build ID: ") {
            return Some(bid.trim().to_string());
        }
    }
    None
}

/// Run ldd and parse entries
fn run_ldd(path: &Path) -> Vec<LddEntry> {
    let output = Command::new("ldd")
        .arg(path.to_string_lossy().as_ref())
        .output()
        .ok();
    let mut entries = Vec::new();
    if let Some(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.contains("linux-vdso") {
                continue;
            }
            // Parse: "library.so => /path/to/lib.so (0xaddr)"
            // or:    "library.so => not found"
            // or:    "library.so (0xaddr)" (directly loaded)
            let (library, rest) = if let Some(idx) = line.find(" => ") {
                let lib = line[..idx].trim().to_string();
                let rest = line[idx + 4..].trim().to_string();
                (lib, rest)
            } else {
                // No =>: it's a direct dependency
                let lib = line.split_whitespace().next()
                    .unwrap_or(line)
                    .to_string();
                (lib, String::new())
            };

            let (resolved_path, load_address) = if rest.contains("not found") {
                ("not found".to_string(), String::new())
            } else {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].trim_matches(|c| c == '(' || c == ')').to_string())
                } else {
                    (rest.clone(), String::new())
                }
            };

            entries.push(LddEntry { library, resolved_path, load_address });
        }
    }
    entries
}

/// Discover and hash all .so libraries in the same directory as the binary
fn discover_libraries(path: &Path) -> Vec<LibraryFingerprint> {
    let mut libraries = Vec::new();
    let parent_dir = match path.parent() {
        Some(p) => p,
        None => return libraries,
    };
    if let Ok(entries) = std::fs::read_dir(parent_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".so") {
                continue;
            }
            let sha256 = sha256_of_file(&entry_path).unwrap_or_default();
            let build_id = extract_build_id(&entry_path);
            let size_bytes = std::fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0);
            libraries.push(LibraryFingerprint {
                name: fname,
                path: entry_path.to_string_lossy().to_string(),
                sha256,
                build_id,
                size_bytes,
            });
        }
    }
    libraries.sort_by(|a, b| a.name.cmp(&b.name));
    libraries
}

/// Get version string from --version
fn get_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    combined.lines().next().map(|s| s.trim().to_string())
}

/// Extract commit hash from version string
fn extract_commit(version: &Option<String>) -> Option<String> {
    let v = version.as_ref()?;
    // Version strings often contain commit hash in parentheses
    // e.g., "build: 505b1ed (100)"
    // e.g., "version 3d42fa1"
    for part in v.split_whitespace() {
        let clean = part.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
        if clean.len() >= 7 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(clean.to_string());
        }
    }
    None
}

/// Extract compiler info from binary (using `file` or strings)
fn extract_compiler(path: &Path) -> Option<String> {
    // Use `file` command to get compiler hints
    let output = Command::new("file").arg(path).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let text_lower = text.to_lowercase();
    if text_lower.contains("gcc") {
        Some("GCC".to_string())
    } else if text_lower.contains("clang") || text_lower.contains("llvm") {
        Some("Clang/LLVM".to_string())
    } else {
        // Try strings for compiler identification
        let strings_out = Command::new("strings")
            .args([path.to_string_lossy().as_ref()])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&strings_out.stdout);
        for line in s.lines() {
            let l = line.to_lowercase();
            if l.contains("gcc") && l.contains("version") {
                return Some(line.trim().to_string());
            }
            if l.contains("clang") && l.contains("version") {
                return Some(line.trim().to_string());
            }
        }
        None
    }
    .map(|s| s.to_string())
}

/// Extract RPATH from ELF binary
fn extract_rpath(path: &Path) -> Option<String> {
    let output = Command::new("readelf")
        .args(["-d", &path.to_string_lossy()])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("RPATH") || line.contains("RUNPATH") {
            if let Some(val) = line.split('[').nth(1) {
                if let Some(end) = val.find(']') {
                    return Some(val[..end].to_string());
                }
            }
        }
    }
    None
}

/// Get file type description
fn get_file_type(path: &Path) -> String {
    let output = Command::new("file")
        .arg("-b") // brief output
        .arg(path)
        .output()
        .ok();
    output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get CUDA version from system
fn get_cuda_version() -> Option<String> {
    let output = Command::new("nvcc")
        .arg("--version")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    // Parse: "Cuda compilation tools, release X.Y, VX.Y.Z"
    for line in text.lines() {
        if line.contains("release") {
            return Some(line.trim().to_string());
        }
    }
    // Fallback: nvidia-smi
    let smi = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()?;
    let driver = String::from_utf8_lossy(&smi.stdout).trim().to_string();
    if !driver.is_empty() {
        Some(format!("Driver {}", driver))
    } else {
        None
    }
}

/// Infer build directory from binary path
fn infer_build_directory(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    // Check if path contains "build/"
    if let Some(idx) = path_str.rfind("/build/") {
        Some(path_str[..idx].to_string() + "/build")
    } else {
        None
    }
}

/// Get help text for capability detection
fn get_help_text(path: &Path) -> String {
    let output = Command::new(path).arg("--help").output().ok();
    match output {
        Some(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            format!("{}\n{}", stdout, stderr)
        }
        None => String::new(),
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_of_file() {
        // SHA256 of self
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Cargo.toml");
        let hash = sha256_of_file(&path);
        assert!(hash.is_some());
        assert_eq!(hash.unwrap().len(), 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_get_file_type() {
        let path = Path::new("/bin/sh");
        let ftype = get_file_type(path);
        assert!(ftype.contains("ELF") || ftype.contains("symbolic") || !ftype.is_empty());
    }
}

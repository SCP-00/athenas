/// ProbeValidation — Scientific instrument that verifies output quality.
///
/// "Un runtime puede producir 300 tok/s y generar basura. Eso no sirve."
///
/// This module checks:
/// - UTF-8 validity
/// - Non-empty output
/// - Token repetition / loops
/// - Proper EOS termination
/// - No corruption
/// - Minimum quality criteria
///
/// Does NOT require an LLM. Purely deterministic analysis.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
/// Validation result for a single generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall pass/fail
    pub passed: bool,

    // ── Basic checks ──
    /// Output is not empty
    pub non_empty: bool,
    /// Output is valid UTF-8 (or lossy-converted cleanly)
    pub utf8_valid: bool,
    /// Server stopped correctly (EOS or max tokens reached)
    pub stopped_correctly: bool,

    // ── Quality checks ──
    /// Token repetition score (0.0 = none, 1.0 = all repeated)
    pub repetition_score: f64,
    /// Did the output enter an infinite loop? (repeating same token)
    pub infinite_loop_detected: bool,
    /// Maximum consecutive repeated token count
    pub max_consecutive_repeats: u64,
    /// Total unique n-grams / total n-grams (diversity measure)
    pub diversity_score: f64,

    // ── Structural checks ──
    /// Number of tokens generated
    pub token_count: u64,
    /// Number of lines
    pub line_count: u64,
    /// Output is corrupt (unexpected binary data)
    pub corrupt_output: bool,
    /// Contains obvious garbage (control chars, invalid sequences)
    pub contains_garbage: bool,

    // ── Metadata ──
    /// Warnings
    pub warnings: Vec<String>,
    /// Raw output snippet (first 200 chars) for debugging
    pub output_snippet: String,
}

impl ValidationResult {
    /// Validate a generated output string.
    /// Performs all deterministic checks.
    pub fn validate(output: &str, expected_tokens: u64, stopped_correctly: bool) -> Self {
        let mut warnings = Vec::new();

        // Basic checks
        let non_empty = !output.is_empty();
        let utf8_valid = true; // Rust strings are always valid UTF-8

        if !non_empty {
            warnings.push("Output is empty".to_string());
        }

        // Token counting (approximate: split by whitespace)
        let tokens: Vec<&str> = output.split_whitespace().collect();
        let token_count = tokens.len() as u64;

        if token_count < expected_tokens.saturating_sub(10) {
            warnings.push(format!(
                "Output shorter than expected: got {} tokens, expected ~{}",
                token_count, expected_tokens
            ));
        }

        // Corruption detection (unexpected null bytes or control chars)
        let contains_garbage = output.contains('\0')
            || output.chars().filter(|c| c.is_control() && *c != '\n' && *c != '\r' && *c != '\t').count() > 5;

        if contains_garbage {
            warnings.push("Output contains control characters or garbage".to_string());
        }

        // Infinite loop detection (same word repeated many times)
        let infinite_loop_detected = if token_count > 0 {
            let max_repeats = max_consecutive_repeats(&tokens);
            max_repeats > (token_count / 2) as usize && max_repeats > 5
        } else {
            false
        };

        let max_consecutive_repeats = if token_count > 0 {
            max_consecutive_repeats(&tokens) as u64
        } else {
            0
        };

        if infinite_loop_detected {
            warnings.push(format!(
                "Infinite loop detected: {} consecutive repeats of the same token",
                max_consecutive_repeats
            ));
        }

        // Repetition score (2-gram overlap)
        let repetition_score = if token_count >= 4 {
            calculate_repetition_score(&tokens)
        } else {
            0.0
        };

        if repetition_score > 0.5 {
            warnings.push(format!(
                "High repetition score: {:.2} (may indicate loop or degeneration)",
                repetition_score
            ));
        }

        // Diversity score (unique bigrams / total bigrams)
        let diversity_score = if token_count >= 2 {
            calculate_diversity_score(&tokens)
        } else {
            1.0
        };

        if diversity_score < 0.2 {
            warnings.push(format!(
                "Low diversity score: {:.2} (output is very repetitive)",
                diversity_score
            ));
        }

        // Line count
        let line_count = output.lines().count() as u64;

        // Output snippet
        let output_snippet = output.chars().take(200).collect::<String>();

        // Overall pass: all critical checks pass
        let passed = non_empty
            && !infinite_loop_detected
            && !contains_garbage
            && stopped_correctly;

        Self {
            passed,
            non_empty,
            utf8_valid,
            stopped_correctly,
            repetition_score,
            infinite_loop_detected,
            max_consecutive_repeats,
            diversity_score,
            token_count,
            line_count,
            corrupt_output: contains_garbage,
            contains_garbage,
            warnings,
            output_snippet,
        }
    }

    /// Quick validation — just check that output is non-empty and not garbage
    pub fn quick(output: &str) -> bool {
        !output.is_empty()
            && max_consecutive_repeats(&output.split_whitespace().collect::<Vec<_>>()) < 10
    }
}

// ═══════════════════════════════════════════════════════════════
// Analysis Algorithms
// ═══════════════════════════════════════════════════════════════

/// Find the maximum number of consecutive identical tokens
fn max_consecutive_repeats(tokens: &[&str]) -> usize {
    if tokens.is_empty() { return 0; }
    let mut max_count = 1;
    let mut current_count = 1;
    for i in 1..tokens.len() {
        if tokens[i] == tokens[i - 1] {
            current_count += 1;
            if current_count > max_count {
                max_count = current_count;
            }
        } else {
            current_count = 1;
        }
    }
    max_count
}

/// Calculate repetition score using 2-gram overlap.
/// Score = percentage of bigrams that are identical to the previous bigram.
/// 0.0 = no repetition, 1.0 = all repeated.
fn calculate_repetition_score(tokens: &[&str]) -> f64 {
    if tokens.len() < 4 { return 0.0; }
    let mut repeats = 0;
    let mut total = 0;
    for i in 2..tokens.len() - 1 {
        let current = (tokens[i], tokens[i + 1]);
        let previous = (tokens[i - 2], tokens[i - 1]);
        if current == previous {
            repeats += 1;
        }
        total += 1;
    }
    if total == 0 { 0.0 } else { repeats as f64 / total as f64 }
}

/// Calculate diversity score = unique bigrams / total bigrams.
/// 1.0 = completely diverse, 0.0 = all the same.
fn calculate_diversity_score(tokens: &[&str]) -> f64 {
    if tokens.len() < 2 { return 1.0; }
    let mut bigrams = std::collections::HashSet::new();
    let mut total = 0;
    for i in 0..tokens.len() - 1 {
        bigrams.insert((tokens[i], tokens[i + 1]));
        total += 1;
    }
    if total == 0 { 1.0 } else { bigrams.len() as f64 / total as f64 }
}

// ═══════════════════════════════════════════════════════════════
// Server Stderr Analysis
// ═══════════════════════════════════════════════════════════════

/// Analyze stderr output from llama-server for diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StderrAnalysis {
    /// Model loaded successfully
    pub model_loaded: bool,
    /// Load time reported (seconds)
    pub reported_load_time_s: Option<f64>,
    /// Flash attention was used (stderr mentions it)
    pub flash_attention_active: bool,
    /// KV cache type reported
    pub reported_kv_cache_type: Option<String>,
    /// CUDA used
    pub cuda_used: bool,
    /// Any errors found
    pub errors: Vec<String>,
    /// OOM detected
    pub oom_detected: bool,
    /// OOM during load or inference
    pub oom_phase: Option<String>,
    /// Any warnings
    pub warnings: Vec<String>,
    /// Timings reported
    pub timings: HashMap<String, f64>,
    /// Thread count used
    pub thread_count: Option<u64>,
    /// Batch size reported
    pub batch_size: Option<u64>,
}

impl StderrAnalysis {
    /// Analyze stderr output from llama-server.
    pub fn analyze(stderr: &str) -> Self {
        let lower = stderr.to_lowercase();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut timings = HashMap::new();
        let mut reported_load_time_s = None;
        let mut reported_kv_cache_type = None;
        let mut thread_count = None;
        let mut batch_size = None;

        // Model loaded
        let model_loaded = lower.contains("llama_model_loader") || lower.contains("model loaded")
            || lower.contains("load_model") || (lower.contains("success") && lower.contains("load"));

        // Load time
        for line in stderr.lines() {
            let l = line.to_lowercase();
            if l.contains("load time") || l.contains("model load") {
                if let Some(secs) = extract_seconds(line) {
                    reported_load_time_s = Some(secs);
                }
            }
            // KV cache type
            if l.contains("cache_type") || l.contains("cache type") || l.contains("kv cache") {
                for part in ["f16", "q8_0", "q4_0", "q4_1", "q6_k", "iq4_nl", "turbo3"] {
                    if l.contains(part) {
                        reported_kv_cache_type = Some(part.to_string());
                        break;
                    }
                }
            }
            // Thread count
            if l.contains("threads") || l.contains("n_threads") {
                if let Some(n) = extract_number(line) {
                    thread_count = Some(n);
                }
            }
            // Batch size
            if l.contains("batch") {
                if let Some(n) = extract_number(line) {
                    batch_size = Some(n);
                }
            }
            // Timings
            if l.contains("ms") && l.contains("/") && l.contains("tok") {
                if let Some(ms) = extract_timing(line) {
                    timings.insert("tokens_per_second".to_string(), ms);
                }
            }
        }

        // Flash attention
        let flash_attention_active = lower.contains("flash attention") || lower.contains("flash_attn")
            || lower.contains("flash-attn: on") || lower.contains("flash_attn: on");

        // CUDA
        let cuda_used = lower.contains("cuda") && (lower.contains("device") || lower.contains("gpu"));

        // OOM
        let oom_detected = lower.contains("oom") || lower.contains("out of memory")
            || lower.contains("cuda error: out of memory");
        let oom_phase = if oom_detected {
            if lower.contains("load") { Some("load".to_string()) }
            else if lower.contains("kv") || lower.contains("cache") { Some("kv_cache".to_string()) }
            else if lower.contains("inference") || lower.contains("generate") { Some("inference".to_string()) }
            else { Some("unknown".to_string()) }
        } else {
            None
        };

        // Extract errors
        for line in stderr.lines() {
            let l = line.to_lowercase();
            if l.contains("error:") || l.contains("fatal:") || l.contains("failed:") {
                errors.push(line.trim().to_string());
            }
        }

        // Extract warnings
        for line in stderr.lines() {
            let l = line.to_lowercase();
            if l.contains("warning:") || l.contains("warn:") {
                warnings.push(line.trim().to_string());
            }
        }

        Self {
            model_loaded,
            reported_load_time_s,
            flash_attention_active,
            reported_kv_cache_type,
            cuda_used,
            errors,
            oom_detected,
            oom_phase,
            warnings,
            timings,
            thread_count,
            batch_size,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Parsing helpers
// ═══════════════════════════════════════════════════════════════

/// Extract a seconds value from a line like "model load time: 4.23s"
fn extract_seconds(line: &str) -> Option<f64> {
    let lower = line.to_lowercase();
    // Try to find a number before "s" or "ms"
    if let Some(end) = lower.rfind(|c: char| c == 's' || c == ' ') {
        let before = &lower[..end];
        let after_space = before.split_whitespace().last()?;
        // Remove trailing "s" or "ms"
        let num_str = after_space.trim_end_matches('s').trim_end_matches('m');
        num_str.parse::<f64>().ok()
    } else {
        None
    }
}

/// Extract a number from a line
fn extract_number(line: &str) -> Option<u64> {
    let mut nums = Vec::new();
    for word in line.split_whitespace() {
        if let Ok(n) = word.parse::<u64>() {
            nums.push(n);
        }
    }
    nums.into_iter().next()
}

/// Extract a timing value (ms per token or tokens per second)
fn extract_timing(line: &str) -> Option<f64> {
    // Look for patterns like "X.XX ms / Y tok"
    let parts: Vec<&str> = line.split_whitespace().collect();
    for i in 0..parts.len().saturating_sub(2) {
        if (parts[i].ends_with("ms") || parts[i].ends_with("s")) && parts[i + 1] == "/" {
            let num_str = parts[i].trim_end_matches('s').trim_end_matches('m');
            if let Ok(val) = num_str.parse::<f64>() {
                return Some(val);
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════
// Quality Score
// ═══════════════════════════════════════════════════════════════

/// Compute a quality score (0.0 to 1.0) from a validation result.
/// Combines multiple metrics into a single score.
pub fn quality_score(result: &ValidationResult) -> f64 {
    // If output is empty, score is 0.0 regardless of other factors
    if !result.non_empty {
        return 0.0;
    }

    let mut score = 0.0;

    // Base score: output exists and is valid
    if result.non_empty { score += 0.2; }
    if result.utf8_valid { score += 0.1; }
    if result.stopped_correctly { score += 0.15; }

    // Quality: no loops or garbage
    if !result.infinite_loop_detected { score += 0.2; }
    if !result.contains_garbage { score += 0.1; }

    // Diversity (up to 0.15)
    score += result.diversity_score * 0.15;

    // Repetition penalty (up to -0.1)
    score -= result.repetition_score * 0.1;

    // Clamp
    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_output() {
        let result = ValidationResult::validate("Hello, I am a helpful assistant.", 10, true);
        assert!(result.passed);
        assert!(result.non_empty);
        assert!(!result.infinite_loop_detected);
        // max_consecutive_repeats starts at 1 (single token always "repeats" once)
        assert!(result.max_consecutive_repeats >= 1);
    }

    #[test]
    fn test_empty_output() {
        let result = ValidationResult::validate("", 10, true);
        assert!(!result.passed);
        assert!(!result.non_empty);
    }

    #[test]
    fn test_infinite_loop() {
        let output = "hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello hello";
        let result = ValidationResult::validate(output, 100, true);
        assert!(result.infinite_loop_detected);
        assert!(result.max_consecutive_repeats >= 16);
    }

    #[test]
    fn test_garbage_output() {
        let output = "valid text \0 null byte \0 more garbage";
        let result = ValidationResult::validate(output, 10, true);
        assert!(result.contains_garbage);
    }

    #[test]
    fn test_repetition_score() {
        let tokens = ["a", "b", "a", "b", "a", "b", "c", "d", "e", "f"];
        let score = calculate_repetition_score(&tokens);
        assert!(score > 0.0); // "a b" repeats at least once
    }

    #[test]
    fn test_diversity_score() {
        let tokens = ["a", "b", "c", "d", "e", "f"];
        let score = calculate_diversity_score(&tokens);
        assert!((score - 1.0).abs() < 0.01); // All unique bigrams
    }

    #[test]
    fn test_low_diversity() {
        let tokens = ["a", "a", "a", "a", "a", "a"];
        let score = calculate_diversity_score(&tokens);
        assert!(score < 0.5); // Very low diversity
    }

    #[test]
    fn test_stderr_analysis_normal() {
        let stderr = "llama_model_loader: loaded successfully\nflash_attn: on\ncuda device: 0\n";
        let analysis = StderrAnalysis::analyze(stderr);
        assert!(analysis.model_loaded);
        assert!(analysis.flash_attention_active);
        assert!(analysis.cuda_used);
        assert!(!analysis.oom_detected);
    }

    #[test]
    fn test_stderr_analysis_oom() {
        let stderr = "CUDA error: out of memory\nllama_model_loader: failed to allocate\n";
        let analysis = StderrAnalysis::analyze(stderr);
        assert!(analysis.oom_detected);
        assert!(!analysis.errors.is_empty());
    }

    #[test]
    fn test_quality_score_perfect() {
        let result = ValidationResult::validate("Hello, I am a helpful assistant.", 10, true);
        let score = quality_score(&result);
        assert!(score > 0.8);
    }

    #[test]
    fn test_quality_score_empty() {
        let result = ValidationResult::validate("", 10, true);
        let score = quality_score(&result);
        // Empty output should have a low quality score (0.0 after fix)
        assert!(score < 0.1);
    }

    #[test]
    fn test_quick_validation() {
        assert!(ValidationResult::quick("Hello, I am a helpful assistant."));
        assert!(!ValidationResult::quick(""));
    }
}

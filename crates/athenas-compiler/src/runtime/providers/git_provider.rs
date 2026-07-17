use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Git Tool Result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitResult {
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub structured: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// GitProvider — runs git operations and returns structured results
// ---------------------------------------------------------------------------

pub struct GitProvider;

impl GitProvider {
    pub fn new() -> Self {
        Self
    }

    /// Check if git is available and we're in a git repository
    pub fn is_available() -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run a git command in the given working directory
    fn run(args: &[&str], working_dir: Option<&Path>) -> GitResult {
        let command_str = format!("git {}", args.join(" "));
        let mut cmd = Command::new("git");
        cmd.args(args);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                let success = output.status.success();

                GitResult {
                    command: command_str,
                    success,
                    stdout,
                    stderr,
                    exit_code,
                    structured: None,
                }
            }
            Err(e) => GitResult {
                command: command_str,
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to execute git: {e}"),
                exit_code: -1,
                structured: None,
            },
        }
    }

    /// Parse git diff --stat output into structured format
    fn parse_diff_stat(output: &str) -> serde_json::Value {
        let mut files = Vec::new();
        for line in output.lines() {
            // Typical format: "src/main.rs | 5 ++---"
            if let Some(pipe_pos) = line.find('|') {
                let file = line[..pipe_pos].trim();
                let rest = line[pipe_pos + 1..].trim();
                let parts: Vec<&str> = rest.split_whitespace().collect();
                let changes = parts.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                files.push(serde_json::json!({
                    "file": file,
                    "changes": changes,
                }));
            }
        }
        serde_json::json!({ "files": files, "total": files.len() })
    }

    /// Parse git log --oneline output into structured format
    fn parse_log_oneline(output: &str) -> serde_json::Value {
        let mut commits = Vec::new();
        for line in output.lines() {
            if let Some(space_pos) = line.find(' ') {
                let hash = &line[..space_pos];
                let message = line[space_pos + 1..].trim();
                commits.push(serde_json::json!({
                    "hash": hash,
                    "message": message,
                }));
            }
        }
        serde_json::json!({ "commits": commits, "total": commits.len() })
    }

    // ── Public API ──

    /// git status
    pub fn status(working_dir: Option<&Path>) -> GitResult {
        let mut result = Self::run(&["status", "--short"], working_dir);
        if result.success {
            let files: Vec<&str> = result.stdout.lines().collect();
            result.structured = Some(serde_json::json!({
                "modified": files.iter().filter(|l| l.starts_with(" M") || l.starts_with("M ")).count(),
                "added": files.iter().filter(|l| l.starts_with("??")).count(),
                "staged": files.iter().filter(|l| l.starts_with(' ')).count(),
                "total": files.len(),
                "entries": files,
            }));
        }
        result
    }

    /// git diff [--staged] [path]
    pub fn diff(staged: bool, path: Option<&str>, working_dir: Option<&Path>) -> GitResult {
        let mut args = vec!["diff", "--stat"];
        if staged {
            args.push("--staged");
        }
        if let Some(p) = path {
            args.push(p);
        }
        let mut result = Self::run(&args, working_dir);
        if result.success {
            result.structured = Some(Self::parse_diff_stat(&result.stdout));
        }
        result
    }

    /// git log [count]
    pub fn log(count: usize, working_dir: Option<&Path>) -> GitResult {
        let count_str = format!("-{count}");
        let mut result = Self::run(&["log", "--oneline", &count_str], working_dir);
        if result.success {
            result.structured = Some(Self::parse_log_oneline(&result.stdout));
        }
        result
    }

    /// git show <ref>
    pub fn show(rev: &str, working_dir: Option<&Path>) -> GitResult {
        Self::run(&["show", "--stat", rev], working_dir)
    }

    /// git blame <file>
    pub fn blame(file: &str, working_dir: Option<&Path>) -> GitResult {
        Self::run(&["blame", "--line-porcelain", file], working_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_available() {
        // This test may fail if not in a git repo, which is fine
        let available = GitProvider::is_available();
        // We don't assert because CI may not have git
        println!("Git available: {available}");
    }

    #[test]
    fn test_parse_diff_stat() {
        let output = "src/main.rs | 5 ++---\nsrc/lib.rs | 10 ++++++++--\n";
        let parsed = GitProvider::parse_diff_stat(output);
        let files = parsed["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0]["file"], "src/main.rs");
        assert_eq!(files[1]["changes"], 10);
    }

    #[test]
    fn test_parse_log_oneline() {
        let output = "abc123 Fix the bug\ndef456 Add new feature\n";
        let parsed = GitProvider::parse_log_oneline(output);
        let commits = parsed["commits"].as_array().unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0]["hash"], "abc123");
        assert_eq!(commits[1]["message"], "Add new feature");
    }

    #[test]
    fn test_empty_diff_stat() {
        let parsed = GitProvider::parse_diff_stat("");
        assert_eq!(parsed["total"], 0);
    }

    #[test]
    fn test_empty_log() {
        let parsed = GitProvider::parse_log_oneline("");
        assert_eq!(parsed["total"], 0);
    }
}

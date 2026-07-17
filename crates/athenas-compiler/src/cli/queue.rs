use std::path::Path;
use std::time::SystemTime;

use crate::runtime::experiment_queue::{Experiment, ExperimentQueue, ExperimentStatus};

/// Generate a unique experiment ID
fn generate_experiment_id() -> String {
    let ts = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("EXP-{ts}")
}

/// Handle `ath queue` subcommands
pub fn run_queue(
    action: &str,
    model_path: Option<&Path>,
    experiment_id: Option<&str>,
    days: u64,
    json_output: bool,
) -> anyhow::Result<i32> {
    let state_dir = Path::new(".state");
    let mut queue = ExperimentQueue::load(state_dir);

    match action {
        "add" => {
            let mp = model_path.ok_or_else(|| {
                anyhow::anyhow!("--model <path> is required for 'add'")
            })?;
            let exp_id = experiment_id
                .map(|s| s.to_string())
                .unwrap_or_else(generate_experiment_id);
            let experiment = Experiment::new(&exp_id, &mp.to_string_lossy());
            queue.enqueue(experiment).map_err(|e| anyhow::anyhow!(e))?;

            if json_output {
                println!("{{\"status\":\"queued\",\"id\":\"{exp_id}\",\"model\":\"{}\"}}",
                    mp.display());
            } else {
                println!("  ✅ Experiment {exp_id} queued");
                println!("  📦 Model: {}", mp.display());
                println!("  📋 Use: ath queue process   — to process now");
                println!("  📋 Use: ath queue list      — to see queue status");
            }
            Ok(0)
        }

        "list" => {
            let all = queue.list(None);
            let counts = queue.count_by_status();

            if json_output {
                let output: Vec<serde_json::Value> = all.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "model": e.model_path,
                        "status": format!("{:?}", e.status),
                        "created_at": e.created_at,
                        "phases_completed": e.completed_phases.len(),
                        "phases_failed": e.failed_phases.len(),
                        "retry_count": e.retry_count,
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "experiments": output,
                    "counts": counts,
                }))?);
            } else {
                println!("╔══════════════════════════════════════════╗");
                println!("║     Experiment Queue                     ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
                println!("📊 Status: {:?}", counts);
                println!();
                if all.is_empty() {
                    println!("  Queue is empty.");
                } else {
                    for exp in all {
                        let status_icon = match exp.status {
                            ExperimentStatus::Queued => "⏳",
                            ExperimentStatus::Running => "🔄",
                            ExperimentStatus::Completed => "✅",
                            ExperimentStatus::Failed(_) => "❌",
                            ExperimentStatus::Blocked(_) => "⛔",
                            ExperimentStatus::Cancelled => "🚫",
                        };
                        let phases = format!("{}/{} phases",
                            exp.completed_phases.len(),
                            exp.completed_phases.len() + exp.failed_phases.len());
                        println!("  {status_icon} {} — {} — {} — {} retries",
                            exp.id, exp.model_path, phases, exp.retry_count);
                    }
                }
                println!();
            }
            Ok(0)
        }

        "process" => {
            if !json_output {
                println!("  🔄 Processing next experiment...");
            }
            match queue.process_next(state_dir) {
                Ok(Some(exp_id)) => {
                    if json_output {
                        println!("{}", serde_json::json!({"id": exp_id, "status": "completed"}));
                    } else {
                        println!();
                        println!("  ✅ Experiment {exp_id} processed");
                        println!("  📁 Artifacts: .state/experiments/{exp_id}/");
                    }
                    Ok(0)
                }
                Ok(None) => {
                    if json_output {
                        println!("{{\"status\":\"empty\"}}");
                    } else {
                        println!("  Queue is empty. Use: ath queue add --model <path>");
                    }
                    Ok(0)
                }
                Err(e) => {
                    Err(anyhow::anyhow!("Processing failed: {e}"))
                }
            }
        }

        "process-all" => {
            let mut results: Vec<serde_json::Value> = Vec::new();
            loop {
                match queue.process_next(state_dir) {
                    Ok(Some(id)) => {
                        let exp = queue.get(&id);
                        let status = exp.map(|e| format!("{:?}", e.status))
                            .unwrap_or_else(|| "unknown".to_string());
                        results.push(serde_json::json!({"id": id, "status": status}));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("  ⚠ Error processing experiment: {e}");
                        continue;
                    }
                }
            }
            if json_output {
                println!("{}", serde_json::json!({"processed": results}));
            } else {
                println!("  ✅ Processed {} experiments", results.len());
                for r in &results {
                    let icon = match r["status"].as_str().unwrap_or("") {
                        s if s.contains("Completed") => "✅",
                        s if s.contains("Failed") => "❌",
                        _ => "⏳",
                    };
                    println!("    {icon} {} — {}", r["id"], r["status"]);
                }
            }
            Ok(0)
        }

        "show" => {
            let id = experiment_id.ok_or_else(|| {
                anyhow::anyhow!("--experiment <id> is required for 'show'")
            })?;
            match queue.get(id) {
                Some(exp) => {
                    if json_output {
                        println!("{}", serde_json::to_string_pretty(exp)?);
                    } else {
                        println!("╔══════════════════════════════════════════╗");
                        println!("║     Experiment Details                   ║");
                        println!("╚══════════════════════════════════════════╝");
                        println!();
                        println!("  📋 ID:       {}", exp.id);
                        println!("  📦 Model:    {}", exp.model_path);
                        println!("  🔵 Status:   {:?}", exp.status);
                        println!("  🔄 Retries:  {}/{}", exp.retry_count, exp.max_retries);
                        println!();
                        println!("  ✅ Phases completed: {}", exp.completed_phases.len());
                        if !exp.completed_phases.is_empty() {
                            for p in &exp.completed_phases {
                                println!("    ✅ {p}");
                            }
                        }
                        println!("  ❌ Phases failed: {}", exp.failed_phases.len());
                        if !exp.failed_phases.is_empty() {
                            for p in &exp.failed_phases {
                                println!("    ❌ {p}");
                            }
                        }
                        if let Some(ref error) = exp.error {
                            println!("  ⚠ Error: {error}");
                        }
                        if let Some(ref result) = exp.result {
                            println!();
                            println!("📊 Results:");
                            println!("  Duration: {:.0}s", result.duration_seconds);
                            println!("  Phases: {}/{} complete", result.completed_phases, result.total_phases);
                            println!("  Runtimes found: {}", result.runtimes_found);
                        }
                        println!();
                    }
                    Ok(0)
                }
                None => {
                    Err(anyhow::anyhow!("Experiment {id} not found"))
                }
            }
        }

        "clean" => {
            let cleaned = queue.clean(days);
            if json_output {
                println!("{{\"cleaned\":{cleaned}}}");
            } else {
                println!("  🧹 Cleaned {cleaned} experiments older than {days} days");
            }
            Ok(0)
        }

        _ => {
            Err(anyhow::anyhow!(
                "Unknown action '{action}'. Available: add, list, process, process-all, show, clean"
            ))
        }
    }
}

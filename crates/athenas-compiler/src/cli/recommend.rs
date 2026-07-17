use std::path::Path;

use crate::runtime::scheduler::{
    CapabilityScheduler, Objective, SchedulerEngine, TaskDescriptor,
};

/// Run `ath recommend` — recommend the best configuration for a task
pub fn run_recommend(
    task_name: Option<&str>,
    objective_name: Option<&str>,
    graph_path: Option<&Path>,
    json_output: bool,
) -> anyhow::Result<i32> {
    let graph_path = graph_path.unwrap_or(Path::new(".athena/graph.json"));

    // Build the engine (load graph if available)
    let engine = if graph_path.exists() {
        SchedulerEngine::load_graph(graph_path)
    } else {
        SchedulerEngine::load_default()
    };

    match task_name {
        // Specific task
        Some(name) => {
            // Find the matching task profile
            let task = find_task(name)
                .map(|t| {
                    if let Some(obj) = objective_name.and_then(Objective::from_name) {
                        t.with_objective(obj)
                    } else {
                        t
                    }
                })
                .unwrap_or_else(|| {
                    // Generic task from user input
                    let mut t = TaskDescriptor::new(name);
                    if let Some(obj) = objective_name.and_then(Objective::from_name) {
                        t = t.with_objective(obj);
                    }
                    t
                });

            let config = engine.select_configuration(&task)?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("{}", config.display());
            }

            Ok(0)
        }
        // List available tasks
        None => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&engine.available_tasks())?
                );
            } else {
                println!("╔══════════════════════════════════════════╗");
                println!("║  Capability Scheduler — Available Tasks  ║");
                println!("╚══════════════════════════════════════════╝");
                println!();

                let tasks = engine.available_tasks();
                for task in &tasks {
                    println!("  🎯  {task}");
                }
                println!();
                println!(
                    "  Use: ath recommend \"<task name>\"");
                println!(
                    "  Use: ath recommend \"<task name>\" --objective \"<objective>\"");
                println!();
                println!("📊 Objectives:");
                for obj in Objective::all() {
                    println!("     {:?}", obj);
                }
                println!();
                println!("💡 Tip: Define new task profiles in runtime/scheduler.rs");
            }
            Ok(0)
        }
    }
}

/// Find a built-in task profile by name
fn find_task(name: &str) -> Option<TaskDescriptor> {
    let lower = name.to_lowercase();
    TaskDescriptor::all_profiles()
        .into_iter()
        .find(|t| t.name.to_lowercase() == lower || t.name.to_lowercase().contains(&lower))
}

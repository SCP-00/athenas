use std::path::Path;

use crate::runtime::experiment::CertificationEngine;

/// Run the autonomous certification engine on a model.
/// This is the main entry point for Chatty's CertificationEngine vision.
/// It discovers hardware, runtimes, plans experiments, recovers from OOM,
/// persists evidence, and generates a knowledge-focused report.
pub fn run_certify_model(model_path: &Path, skip_known: bool, json_output: bool) -> anyhow::Result<i32> {
    if !model_path.exists() {
        anyhow::bail!("Model file not found: {}", model_path.display());
    }

    let experiment_id = format!("CERT-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    if !json_output {
        println!(
            "╔══════════════════════════════════════════════════════════╗\n\
             ║     Athena Autonomous Certification Engine v0.1.0        ║\n\
             ╚══════════════════════════════════════════════════════════╝\n"
        );
        println!("🔬 Experiment: {}\n", experiment_id);
        println!("📋 Model: {}\n", model_path.display());
    }

    // Phase 1: Initialize the engine
    if !json_output {
        println!("📡 Phase 1: Discovering hardware and runtimes...");
    }

    let state_dir = Path::new(".state");
    let mut engine = CertificationEngine::new(state_dir);

    engine.discover()?;

    let hw = engine.hardware.as_ref().ok_or_else(|| anyhow::anyhow!("Hardware detection failed"))?;

    if !json_output {
        for gpu in &hw.gpu {
            println!("  🎮 GPU: {} ({} GB VRAM)", gpu.model, gpu.vram_gb);
        }
        println!("  💾 RAM: {:.1} GB available", hw.memory.available_gb);
        println!("  🖥  CPU: {} ({} cores)", hw.cpu.model, hw.cpu.cores);

        if engine.runtimes.is_empty() {
            println!("  ⚠ No runtimes found!");
        } else {
            println!("  ✓ {} runtime(s) discovered", engine.runtimes.len());
            for rt in &engine.runtimes {
                let score = rt.capability_score();
                println!("     [{:.2}] {} ({})", score, rt.display_name, rt.binary_path);
            }
        }
        println!();
    }

    // Phase 2-7: Run the full certification
    if !json_output {
        println!("🔬 Phase 2-7: Running autonomous certification...");
        println!("     This will evaluate multiple configurations");
        println!("     to find the optimal setup for this model + hardware.\n");
    }

    let report = engine.certify(model_path, &experiment_id, skip_known)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!();
        println!("{}", report.display());
    }

    Ok(0)
}

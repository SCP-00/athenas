use std::path::Path;
use std::time::SystemTime;

use crate::runtime::phase::{
    ArtifactStore, Phase, PhaseContext, PhaseOutput, PhaseStatus,
    phases::{CapabilityDiscoveryPhase, ExecutionLabPhase, GgufInspectionPhase, HardwarePhase,
             MemoryHypothesisPhase, PhaseRegistry, RuntimeCapabilitiesPhase,
             RuntimeDiscoveryPhase, RuntimeFingerprintPhase, register_all_phases},
};

/// Run a single phase and persist the result.
pub fn run_phase(
    phase_id: &str,
    experiment_id: Option<&str>,
    model_path: Option<&Path>,
    runtime_path: Option<&Path>,
    json_output: bool,
) -> anyhow::Result<i32> {
    // Generate experiment ID if not provided
    let exp_id = experiment_id.unwrap_or_else(|| {
        let ts = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Box::leak(format!("EXP-{ts}").into_boxed_str())
        // Note: leaked string is acceptable for CLI commands
    }).to_string();

    // Register all phases
    let mut registry = PhaseRegistry::new();
    register_all_phases(&mut registry);

    // Get the phase
    let phase = registry.get(phase_id)
        .ok_or_else(|| {
            let available = registry.list();
            anyhow::anyhow!("Unknown phase '{phase_id}'. Available: {:?}", available)
        })?;

    // Create a specialized phase if it needs a model path or runtime path
    let specialized_phase: Box<dyn Phase> = match phase_id {
        "PHASE-0004-gguf-inspection" => {
            let mp = model_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0004 requires --model <path>"
            ))?;
            Box::new(GgufInspectionPhase::new(&mp.to_string_lossy()))
        }
        "PHASE-0005-memory-hypothesis" => {
            let mp = model_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0005 requires --model <path>"
            ))?;
            Box::new(MemoryHypothesisPhase::new(&mp.to_string_lossy()))
        }
        "PHASE-0006-execution-lab" => {
            let rt = runtime_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0006 requires --runtime <path> (llama-server binary)"
            ))?;
            let mp = model_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0006 requires --model <path>"
            ))?;
            Box::new(ExecutionLabPhase::new(
                &rt.to_string_lossy(),
                &mp.to_string_lossy(),
            ))
        }
        "PHASE-0007-runtime-fingerprint" => {
            let rt = runtime_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0007 requires --runtime <path> (llama-server binary)"
            ))?;
            Box::new(RuntimeFingerprintPhase::new(&rt.to_string_lossy()))
        }
        "PHASE-0008-capability-discovery" => {
            let rt = runtime_path.ok_or_else(|| anyhow::anyhow!(
                "PHASE-0008 requires --runtime <path> (llama-server binary)"
            ))?;
            Box::new(CapabilityDiscoveryPhase::new(&rt.to_string_lossy()))
        }
        _ => Box::new(PlaceholderPhase(phase_id.to_string())),
    };

    // Use the specialized phase if applicable, otherwise use the registry phase
    let active_phase: &dyn Phase = match phase_id {
        "PHASE-0004-gguf-inspection" | "PHASE-0005-memory-hypothesis" | "PHASE-0006-execution-lab"
        | "PHASE-0007-runtime-fingerprint" | "PHASE-0008-capability-discovery" => &*specialized_phase,
        _ => phase,
    };

    // Initialize context and store
    let ctx = PhaseContext::new(&exp_id, Path::new(".state"));
    let mut store = ArtifactStore::new(Path::new(".state"));

    // Run any dependent phases if needed
    for input in active_phase.inputs() {
        if !store.phase_exists(&exp_id, input) {
            if !json_output {
                eprintln!("  ⚡ Running dependency: {input}");
            }
            // Find and run the dependency phase
            if let Some(dep_phase) = registry.get(input) {
                let dep_output = dep_phase.execute(&ctx, &store)
                    .map_err(|e| anyhow::anyhow!("Dependency {input} failed: {e}"))?;
                store.save_phase(&exp_id, &dep_output)
                    .map_err(|e| anyhow::anyhow!("Cannot save dependency {input}: {e}"))?;
            }
        }
    }

    // Execute the phase
    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Athena Phase Pipeline v0.1.0          ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("🔬 Phase:   {}", active_phase.id());
        println!("❓ Question: {}", active_phase.question());
        println!("📋 Exp ID:  {}", exp_id);
        println!();
        print!("⚡ Executing...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }

    let start = std::time::Instant::now();
    let mut output = active_phase.execute(&ctx, &store)
        .map_err(|e| anyhow::anyhow!("Phase execution failed: {e}"))?;
    output.duration_ms = start.elapsed().as_millis() as u64;

    // Save to artifact store
    store.save_phase(&exp_id, &output)
        .map_err(|e| anyhow::anyhow!("Cannot save phase result: {e}"))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(" ✅");
        println!();
        println!("📊 Results:");
        println!("  Status:    {:?}", output.status);
        println!("  Duration:  {} ms", output.duration_ms);
        println!();
        println!("📈 Metrics:");
        for (key, value) in &output.metrics.values {
            let unit = output.metrics.units.get(key).cloned().unwrap_or_default();
            println!("  {}: {} {}", key, value, unit);
        }
        println!();
        println!("📁 Artifacts saved to: .state/experiments/{exp_id}/phases/{}/", active_phase.id());
        println!();
        println!("✅ Phase complete!");
    }

    Ok(0)
}

/// List all registered phases
pub fn list_phases() -> anyhow::Result<i32> {
    let mut registry = PhaseRegistry::new();
    register_all_phases(&mut registry);

    println!("╔══════════════════════════════════════════╗");
    println!("║     Athena Phase Pipeline — Phases       ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    let phases = registry.list();
    for id in &phases {
        if let Some(phase) = registry.get(id) {
            println!("  📌 {}", id);
            println!("     ❓ {}", phase.question());
            println!("     💬 {}", phase.description());
            if !phase.inputs().is_empty() {
                println!("     ⬆  Depends on: {:?}", phase.inputs());
            }
            println!();
        }
    }

    println!("  Total: {} phases registered", phases.len());
    Ok(0)
}

/// Placeholder phase for phases that haven't been fully implemented yet
struct PlaceholderPhase(String);

impl Phase for PlaceholderPhase {
    fn id(&self) -> &str { &self.0 }
    fn question(&self) -> &str { "Not implemented yet" }
    fn description(&self) -> &str { "This phase has not been implemented yet." }
    fn execute(&self, _ctx: &PhaseContext, _store: &dyn crate::runtime::phase::core::ArtifactStoreRead) -> Result<PhaseOutput, String> {
        Err(format!("Phase {} is not implemented yet", self.0))
    }
}

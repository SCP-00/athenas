use std::path::Path;

use crate::runtime::model_intelligence::gguf::read_gguf_metadata;
use crate::runtime::model_intelligence::model_profile::ModelProfile;
use crate::runtime::runtime_discovery::RuntimeProber;

/// Run `ath analyze <model.gguf>` — full model intelligence pipeline.
/// Reads GGUF metadata, analyzes hardware, calculates memory, discovers runtimes, and recommends configurations.
pub fn run_analyze(model_path: &Path, json_output: bool) -> anyhow::Result<i32> {
    if !model_path.exists() {
        anyhow::bail!("Model file not found: {}", model_path.display());
    }
    if model_path.extension().map(|e| e != "gguf").unwrap_or(true) {
        anyhow::bail!("File must have .gguf extension");
    }

    if !json_output {
        println!(
            "╔══════════════════════════════════════════╗\n\
             ║     Athena Model Intelligence v0.1.0       ║\n\
             ╚══════════════════════════════════════════╝\n"
        );
        println!("🔍 Analyzing: {}", model_path.display());
        println!();
    }

    // Phase 1: Read GGUF metadata
    if !json_output {
        println!("📖 Phase 1: Reading GGUF metadata...");
    }
    let metadata = read_gguf_metadata(model_path)
        .map_err(|e| anyhow::anyhow!("GGUF read failed: {e}"))?;

    if !json_output {
        println!("  ✓ GGUF v{} — {} metadata keys, {} tensors",
            metadata.gguf_version, metadata.metadata_kv_count, metadata.tensor_count);
        if let Some(arch) = &metadata.architecture {
            println!("  🧠 Architecture: {}", arch);
        }
        if let Some(ctx) = metadata.context_length {
            println!("  📏 Context: {} ({}K tokens)", ctx, ctx / 1024);
        }
        println!();
    }

    // Phase 2: Full profile (metadata + memory analysis)
    if !json_output {
        println!("📊 Phase 2: Memory analysis...");
    }
    let profile = ModelProfile::analyze(model_path)
        .map_err(|e| anyhow::anyhow!("Profile analysis failed: {e}"))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&profile)?);
    } else {
        println!("{}", profile.display());
    }

    // Phase 3: Runtime Discovery
    if !json_output {
        println!("\n📡 Phase 3: Discovering runtimes...");
        let runtimes = RuntimeProber::probe_all();
        if runtimes.is_empty() {
            println!("  ⚠️  No runtimes found. Install llama.cpp or Ollama.");
        } else {
            println!("  ✓ Found {} runtime{}", runtimes.len(), if runtimes.len() == 1 { "" } else { "s" });
            println!();
            for rt in &runtimes {
                let score = rt.capability_score();
                println!("  [{:.3}] {} v{}", score, rt.display_name,
                    rt.version.as_ref().map(|v| {
                        // Extract just the first part before "(probed"
                        v.split(" (probed").next().unwrap_or(v).to_string()
                    }).unwrap_or_else(|| "?".to_string()));
                println!("        Path:  {}", rt.binary_path);
                println!("        Caps:  flash={} cuda={} kv_quant={} embed={} vision={} bonsai={} spec={} grammar={}",
                    yesno(rt.supports_flash_attention),
                    yesno(rt.supports_cuda),
                    yesno(rt.supports_kv_cache_quant),
                    yesno(rt.supports_embeddings),
                    yesno(rt.supports_vision),
                    yesno(rt.supports_bonsai),
                    yesno(rt.supports_speculative_decoding),
                    yesno(rt.supports_grammar),
                );
                if !rt.special_binaries.is_empty() {
                    println!("        Special: {}", rt.special_binaries.join(", "));
                }
            }

            // Recommend best runtime for this model
            let best = &runtimes[0];
            println!();
            println!("  🏆 Recommended runtime for this model:");
            println!("       {} ({})", best.display_name, best.binary_path);
            println!("       Capability score: {:.3} ({} capabilities)", best.capability_score(), best.capability_count());
            if !best.kv_cache_types.is_empty() {
                println!("       Best KV types: {}", best.kv_cache_types.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
            }
        }
    }

    println!("\n✅ Analysis complete.");
    Ok(0)
}

fn yesno(v: bool) -> &'static str {
    if v { "✅" } else { "❌" }
}

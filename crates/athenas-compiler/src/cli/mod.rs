pub mod analyze;
pub mod certify_model;
pub mod phase;
pub mod queue;
pub mod inspect;
pub mod recommend;

pub use analyze::run_analyze;
pub use certify_model::run_certify_model;
pub use inspect::run_inspect;
pub use phase::list_phases;
pub use phase::run_phase;
pub use queue::run_queue;
pub use recommend::run_recommend;

use colored::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::parser;
use crate::graph;
use crate::generators;
use crate::validator;
use crate::runtime;
use crate::runtime::benchmark::BenchmarkRegistry;
use crate::runtime::scheduler::CapabilityScheduler;
use crate::runtime::{InferenceParams, Runtime};

// ── Run subcommand ──

pub fn run_validate(root: &Path, schemas_dir: &Path, verbose: bool) -> anyhow::Result<i32> {
    println!("{} Loading schemas...", "📋".bold());
    let schemas = validator::load_schemas(schemas_dir)?;
    println!("  Loaded {} document type schemas", schemas.len().to_string().bright_green());

    println!("{} Scanning project: {}", "🔍".bold(), root.display());
    let documents = parser::parse_all_documents(root)?;

    if documents.is_empty() {
        println!("  {}", "ℹ No documents found".dimmed());
        return Ok(0);
    }

    println!("  Found {} markdown documents with valid front-matter", documents.len().to_string().bright_green());

    if verbose {
        for doc in &documents {
            let doc_type = doc.id.split('-').next().unwrap_or("?");
            let status = doc.front_matter.get("status").and_then(|v| v.as_str()).unwrap_or("?");
            println!("    {} {} — {} ({}, {})", "📄".dimmed(), doc.id.bright_white(), doc.path.dimmed(), doc_type, status);
        }
        println!();
    }

    let mut exit_code = validate_documents_internal(&documents, &schemas);

    let g = graph::build_knowledge_graph(&documents);
    let diagnostics = generators::generate_diagnostics(&documents, &g);
    if !diagnostics.is_empty() {
        println!("  {} Diagnostics:", "ℹ".bold());
        for diag in &diagnostics {
            let level = match diag.severity.as_str() {
                "error" => "✖".red(),
                "warning" => "⚠".yellow(),
                _ => "ℹ".dimmed(),
            };
            println!("    {} [{}] {} — {}", level, diag.id, diag.message, diag.path.dimmed());
            if diag.severity == "error" { exit_code = 1; }
        }
    }

    println!();
    let status = if exit_code == 0 { "✓ Validation PASSED".bright_green() } else { "✖ Validation FAILED".bright_red() };
    println!("{}", status);
    println!("  Documents: {} | Edges: {} | Errors: {} | Warnings: {}",
        g.metadata.total_nodes.to_string().bright_white(),
        g.metadata.total_edges.to_string().bright_cyan(),
        diagnostics.iter().filter(|d| d.severity == "error").count().to_string().bright_red(),
        diagnostics.iter().filter(|d| d.severity == "warning").count().to_string().yellow(),
    );

    Ok(exit_code)
}

pub fn run_graph_cmd(root: &Path, output_dir: &Path, verbose: bool) -> anyhow::Result<i32> {
    println!("{} Scanning project: {}", "🔍".bold(), root.display());
    let documents = parser::parse_all_documents(root)?;
    if documents.is_empty() {
        println!("  {}", "ℹ No documents found".dimmed());
        return Ok(0);
    }
    println!("  Found {} documents", documents.len().to_string().bright_green());
    if verbose {
        for doc in &documents {
            println!("    {} {} — {}", "📄".dimmed(), doc.id.bright_white(), doc.path.dimmed());
        }
        println!();
    }
    println!("{} Building knowledge graph...", "⚙".bold());
    let g = graph::build_knowledge_graph(&documents);
    println!("  Nodes: {} | Edges: {} | Types: {}",
        g.metadata.total_nodes.to_string().bright_green(),
        g.metadata.total_edges.to_string().bright_cyan(),
        g.metadata.doc_types.len().to_string().bright_yellow(),
    );
    std::fs::create_dir_all(output_dir)?;
    let json = serde_json::to_string_pretty(&g)?;
    let output_path = output_dir.join("graph.json");
    std::fs::write(&output_path, json)?;
    println!("  ✓ Graph written to {}", output_path.display().to_string().bright_white());
    println!();
    println!("{}", "✓ Graph complete!".bright_green().bold());
    Ok(0)
}

pub fn run_build(root: &Path, output_dir: &Path, schemas_dir: &Path, _verbose: bool) -> anyhow::Result<i32> {
    println!("{} Phase 1: Load schemas", "1️⃣".bold());
    let schemas = validator::load_schemas(schemas_dir)?;
    println!("  Loaded {} document type schemas", schemas.len().to_string().bright_green());

    println!("{} Phase 2: Parse documents", "2️⃣".bold());
    let documents = parser::parse_all_documents(root)?;
    println!("  Found {} documents", documents.len().to_string().bright_green());

    println!("{} Phase 3: Validate", "3️⃣".bold());
    let exit_code = validate_documents_internal(&documents, &schemas);
    if exit_code != 0 {
        println!("  {} Build aborted — validation errors found", "⚠".yellow());
        return Ok(exit_code);
    }

    println!();
    println!("{} Phase 4: Compile knowledge graph", "4️⃣".bold());
    let output = generators::compile_all(&documents, root);
    println!("  Nodes: {} | Edges: {} | Types: {} | Diagnostics: {}",
        output.graph.metadata.total_nodes.to_string().bright_green(),
        output.graph.metadata.total_edges.to_string().bright_cyan(),
        output.graph.metadata.doc_types.len().to_string().bright_yellow(),
        output.diagnostics.len().to_string().bright_red(),
    );

    println!();
    println!("{} Phase 5: Generate artifacts", "5️⃣".bold());
    generators::write_output(&output, output_dir)?;

    println!();
    println!("{}", "✓ Build complete!".bright_green().bold());
    println!("  Output: {}", output_dir.display().to_string().bright_white());
    Ok(0)
}

pub fn run_doctor(json_output: bool) -> anyhow::Result<i32> {
    use runtime::hardware;
    use runtime::Capability;

    println!("╔══════════════════════════════════════════╗");
    println!("║        Athenas System Doctor             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    println!("{} Detecting hardware...", "🔍".bold());
    let hw = hardware::detect_hardware();

    println!("{} Discovering models...", "📦".bold());
    let models = runtime::find_all_models();

    if json_output {
        let output = serde_json::json!({
            "hardware": hw, "models": models,
            "capabilities": runtime::Capability::all().iter().map(|c| c.name()).collect::<Vec<_>>(),
            "platform": std::env::consts::ARCH,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("  🖥  CPU: {} ({} cores, {} threads)", hw.cpu.model, hw.cpu.cores, hw.cpu.threads);
        for gpu in &hw.gpu {
            println!("  🎮 GPU: {} ({} GB VRAM, driver {})", gpu.model, gpu.vram_gb, gpu.driver_version);
        }
        println!("  💾 RAM: {:.0} GB total ({:.0} GB available)", hw.memory.total_gb, hw.memory.available_gb);
        println!("  💻 OS: {} {} ({})", hw.os.name, hw.os.version, hw.os.arch);
        println!("  🐧 Kernel: {}", hw.kernel);
        println!();
        println!("📦 Models discovered:");
        if models.is_empty() {
            println!("  No GGUF models found.");
        } else {
            for m in &models {
                println!("  {} ({:.0}B, {})", m.id, m.parameters_b, m.quantization);
                println!("     Path: {}", m.path);
            }
        }
        println!();
        println!("🎯 Available capabilities:");
        for c in runtime::Capability::all() {
            println!("  - {}", c.name());
        }
        println!();
    }
    Ok(0)
}

pub fn run_inference(
    input_model_path: Option<&Path>,
    prompt: Option<&str>,
    workspace: Option<&Path>,
    max_tokens: usize,
    temperature: f64,
    json_output: bool,
    input_server_path: Option<&Path>,
    _verbose: bool,
) -> anyhow::Result<i32> {
    // ── Auto-selection via Scheduler ──
    let resolved_model: PathBuf;
    let resolved_server: Option<PathBuf>;

    if input_model_path.is_none() {
        if !json_output {
            println!("  🔄 No model specified — auto-selecting best configuration...");
        }

        // Use the Scheduler to find the best configuration
        let scheduler = runtime::scheduler::SchedulerEngine::load_default();
        let task = runtime::scheduler::TaskDescriptor::new("Default").with_capabilities(
            if workspace.is_some() {
                vec!["coding", "reasoning", "tool-calling"]
            } else {
                vec!["text-generation", "reasoning"]
            }
        );

        match scheduler.select_configuration(&task) {
            Ok(config) => {
                if !json_output {
                    println!("  💡 Recommended: {} — {:.0}B, capability {:.1}%",
                        config.model, config.model_params_b, config.capability_score * 100.0);
                    println!();
                }

                // Find the model by ID pattern
                let all_models = runtime::find_all_models();
                let found = all_models.iter().find(|m| {
                    config.model.contains(&m.id) || m.id.contains(&config.model)
                });

                match found {
                    Some(m) => {
                        resolved_model = PathBuf::from(&m.path);
                        resolved_server = input_server_path.map(|p| p.to_path_buf());
                    }
                    None => {
                        if !json_output {
                            println!("  ⚠ Could not find '{}' — trying first available model...", config.model);
                        }
                        let first = runtime::find_model(None)?;
                        resolved_model = first;
                        resolved_server = input_server_path.map(|p| p.to_path_buf());
                    }
                }
            }
            Err(_) => {
                if !json_output {
                    println!("  ⚠ Scheduler unavailable — using first available model");
                }
                let first = runtime::find_model(None)?;
                resolved_model = first;
                resolved_server = input_server_path.map(|p| p.to_path_buf());
            }
        }
    } else {
        resolved_model = input_model_path.unwrap().to_path_buf();
        resolved_server = input_server_path.map(|p| p.to_path_buf());
    };

    let model = runtime::find_model(Some(&resolved_model))?;
    let info = runtime::infer_model_info(&model, None);

    let workspace_prompt = workspace.and_then(|w| {
        let config_path = w.join("athenas.json");
        if config_path.exists() {
            std::fs::read_to_string(&config_path).ok().and_then(|content| {
                serde_json::from_str::<serde_json::Value>(&content).ok()
                    .and_then(|v| v["system_prompt"].as_str().map(|s| s.to_string()))
            })
        } else { None }
    });

    let prompt_text = match (workspace_prompt, prompt) {
        (Some(wp), None) => { if !json_output { println!("  📖 Using system prompt from workspace ({} chars)", wp.len()); } wp }
        (Some(wp), Some(user_prompt)) => format!("{}\n\n### User Request\n\n{user_prompt}", wp),
        (None, Some(p)) => p.to_string(),
        (None, None) => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    if prompt_text.trim().is_empty() {
        anyhow::bail!("No prompt provided. Use --prompt or pipe input via stdin.");
    }

    // ── Git context (if in a repo) ──
    if !json_output && runtime::providers::git_provider::GitProvider::is_available() {
        let status = runtime::providers::git_provider::GitProvider::status(None);
        if status.success && status.structured.is_some() {
            let stats = status.structured.as_ref().unwrap();
            let total = stats["total"].as_i64().unwrap_or(0);
            if total > 0 {
                println!("  📂 Git repo: {} file(s) modified ({}, {}, staged)",
                    total, stats["modified"], stats["added"]);
            }
        }
    }

    let mut rt_builder = runtime::LlamaServerRuntime::new();
    if let Some(sp) = &resolved_server {
        rt_builder = rt_builder.with_server_path(sp.clone());
    }

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Athenas Runtime v0.1.0 — Spike       ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("📋 Model: {}", info.path);
        println!("📏 Parameters: {:.0}B | Quant: {} | Context: {}", info.parameters_b, info.quantization, info.context_length);
        println!("💬 Prompt: {} chars", prompt_text.len());
        println!("🎯 Max tokens: {} | Temperature: {}", max_tokens, temperature);
        println!();
    }

    let mut rt = rt_builder;
    let load_start = Instant::now();
    rt.load_model(&model)?;
    let load_time = load_start.elapsed();

    if !json_output {
        println!();
        println!("⏱  Model loaded in {:.1}s", load_time.as_secs_f64());
        println!();
        println!("⚡ Generating...");
    }

    let params = InferenceParams { max_tokens, temperature, ..Default::default() };
    let result = rt.complete(&prompt_text, &params)?;
    rt.unload()?;

    if json_output {
        let output = serde_json::json!({
            "runtime": rt.name(), "model": info,
            "prompt": { "text": prompt_text, "chars": prompt_text.len() },
            "inference": {
                "text": result.text,
                "performance": {
                    "ttft_ms": result.ttft_ms, "tokens_per_second": result.tokens_per_second,
                    "total_tokens": result.total_tokens, "prompt_tokens": result.prompt_tokens,
                    "total_duration_ms": result.total_duration_ms,
                }
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!();
        println!("📝 Response:");
        println!("{}", "─".repeat(60));
        println!("{}", result.text);
        println!("{}", "─".repeat(60));
        println!();
        println!("📊 Performance:");
        println!("  ⏱  TTFT (Time to First Token):  {:.1} ms", result.ttft_ms);
        println!("  🚀 Tokens/sec:                  {:.1}", result.tokens_per_second);
        println!("  📊 Total tokens generated:      {}", result.total_tokens);
        println!("  📊 Prompt tokens processed:     {}", result.prompt_tokens);
        println!("  ⏱  Total duration:              {:.1} ms", result.total_duration_ms);
        println!();
        println!("{} Spike complete!", "✓".bright_green());
    }
    Ok(0)
}

pub fn run_pack(action: &crate::PackAction, packs_dir: Option<&Path>, json_output: bool) -> anyhow::Result<i32> {
    let dir = packs_dir.map(|p| p.to_path_buf()).unwrap_or_else(runtime::knowledge::default_packs_dir);
    let packs = runtime::knowledge::discover_packs(&dir);

    match action {
        crate::PackAction::List => {
            if json_output { println!("{}", serde_json::to_string_pretty(&packs)?); return Ok(0); }
            println!("╔══════════════════════════════════════════╗");
            println!("║        Athenas Knowledge Packs           ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            if packs.is_empty() {
                println!("  No knowledge packs found in: {}", dir.display());
            } else {
                for pack in &packs {
                    println!("  📦 {} (v{})", pack.name, pack.version);
                    println!("     ID: {}", pack.id);
                    println!("     {}", pack.description);
                    println!("     Languages: {} | Tools: {} | Knowledge items: {}",
                        pack.languages.join(", "), pack.tools.len(), pack.knowledge.len());
                    if !pack.depends_on.is_empty() { println!("     Depends on: {}", pack.depends_on.join(", ")); }
                    println!();
                }
            }
        }
        crate::PackAction::Show { id } => {
            let pack = runtime::knowledge::find_pack(&packs, id)
                .ok_or_else(|| anyhow::anyhow!("Pack '{id}' not found"))?;
            let tools = runtime::knowledge::check_tools(pack);
            let installed_count = tools.iter().filter(|t| t.installed).count();

            if json_output {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "pack": pack, "tools_status": tools }))?);
            } else {
                println!("╔══════════════════════════════════════════╗");
                println!("║        Pack Details                       ║");
                println!("╚══════════════════════════════════════════╝");
                println!();
                println!("  📦 {} (v{})", pack.name, pack.version);
                println!("     ID: {}", pack.id);
                println!("     {}", pack.description);
                println!("     🔧 Tools ({}/{})", installed_count, tools.len());
                for tool in &tools {
                    let status = if tool.installed { format!("✓ {}", tool.version.as_deref().unwrap_or("installed")) } else { "✗ not found".to_string() };
                    println!("        {} — {} ({status})", tool.name, tool.description);
                }
                if let Some(system) = pack.prompts.get("system") {
                    let preview: String = system.chars().take(200).collect();
                    println!();
                    println!("  🧠 System prompt (preview):");
                    println!("     {preview}...");
                }
            }
        }
    }
    Ok(0)
}

pub fn run_workspace(action: &crate::WorkspaceAction) -> anyhow::Result<i32> {
    use crate::runtime::knowledge;

    match action {
        crate::WorkspaceAction::List => {
            let packs_dir = knowledge::default_packs_dir();
            let packs = knowledge::discover_packs(&packs_dir);
            println!("╔══════════════════════════════════════════╗");
            println!("║      Athenas Workspaces                   ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("  Available workspaces (from knowledge packs):");
            println!();
            for pack in &packs {
                println!("  🏗️  {} ({})", pack.name, pack.id);
                println!("     $ ath workspace create {}", pack.id);
                println!();
            }
        }
        crate::WorkspaceAction::Create { pack_id, output } => {
            let packs_dir = knowledge::default_packs_dir();
            let packs = knowledge::discover_packs(&packs_dir);
            let pack = knowledge::find_pack(&packs, pack_id)
                .ok_or_else(|| anyhow::anyhow!("Pack '{pack_id}' not found"))?;

            let workspace_dir = output.join(format!("workspace-{pack_id}"));
            std::fs::create_dir_all(&workspace_dir)?;
            let tools = knowledge::check_tools(pack);
            let config = serde_json::json!({
                "workspace": { "name": format!("{} Workspace", pack.name), "pack": pack.id, "pack_version": pack.version, "languages": pack.languages },
                "tools": tools.iter().map(|t| serde_json::json!({ "name": t.name, "command": t.command, "installed": t.installed, "version": t.version })).collect::<Vec<_>>(),
                "model_config": { "temperature": 0.7, "max_tokens": 2048 },
                "system_prompt": knowledge::build_system_prompt(pack, None),
            });
            let config_path = workspace_dir.join("athenas.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

            println!("╔══════════════════════════════════════════╗");
            println!("║        Workspace Created                  ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("  🏗️  Workspace: {}", workspace_dir.display());
            println!("  📦 Pack: {} (v{})", pack.name, pack.version);
            println!("  🔧 Tools available: {}", tools.iter().filter(|t| t.installed).count());
            println!();
            println!("  Config: {}", config_path.display());
            println!("  To use: cd {} && ath run", workspace_dir.display());
        }
    }
    Ok(0)
}

pub fn run_knowledge_build(provider_name: &str, query: &str, output_dir: &Path, json_output: bool) -> anyhow::Result<i32> {
    println!("╔══════════════════════════════════════════╗");
    println!("║     Athenas Knowledge Builder             ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("  Provider: {provider_name}");
    println!("  Query:    {query}");
    println!();

    match provider_name {
        "man" => {
            let provider = runtime::providers::man_provider::ManProvider::new();
            let (ir, pack) = runtime::knowledge::build_knowledge(&provider, query)
                .map_err(|e| anyhow::anyhow!("Build failed: {e}"))?;

            std::fs::create_dir_all(output_dir)?;
            let ir_path = output_dir.join(format!("{}-ir.json", query.replace(' ', "-")));
            std::fs::write(&ir_path, serde_json::to_string_pretty(&ir)?)?;
            let pack_path = output_dir.join(format!("{}.yaml", query.replace(' ', "-")));
            std::fs::write(&pack_path, serde_yaml::to_string(&pack)?)?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "provider": provider_name, "query": query, "ir": ir, "pack": pack }))?);
            } else {
                println!();
                println!("📊 Build Report");
                println!("{}", "─".repeat(50));
                println!("  Raw bytes:       {}", ir.metrics.raw_bytes);
                println!("  Items extracted: {}", ir.metrics.items_extracted);
                println!("  Dedup removed:   {}", ir.metrics.dedup_removed);
                println!("  Validation:      {}", if ir.validation.valid { "✓ PASS" } else { "✖ FAIL" });
                println!("  Compile time:    {:.0} ms", ir.metrics.compile_time_ms);
                println!();
                println!("  📁 IR:     {}", ir_path.display());
                println!("  📦 Pack:   {}", pack_path.display());
            }
            Ok(0)
        }
        _ => anyhow::bail!("Unknown provider '{provider_name}'. Available providers: man"),
    }
}

pub fn run_certify(
    registry: &BenchmarkRegistry,
    model_path: Option<&Path>,
    benchmark: Option<&str>,
    capability_name: &str,
    pack_id: Option<&str>,
    level: u8,
    workspace_path: Option<&Path>,
    max_tokens: usize,
    server_path: Option<&Path>,
    json_output: bool,
    mock: bool,
) -> anyhow::Result<i32> {
    use crate::runtime::benchmark;
    use runtime::Capability;

    // Benchmark Engine mode
    if let Some(benchmark_id) = benchmark {
        if capability_name != "text-generation" {
            eprintln!("  ⚠ --capability is ignored when --benchmark is set\n");
        }
        let runner = registry.get(benchmark_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown benchmark '{benchmark_id}'. Available: {:?}", registry.list()))?;

        if mock {
            let _report = benchmark::run_benchmark_certify_mock(runner, level, pack_id, max_tokens, json_output)?;
            return Ok(0);
        }
        let model = model_path.ok_or_else(|| anyhow::anyhow!("Model path required for benchmark certification"))?;
        let _report = benchmark::run_benchmark_certify(runner, level, pack_id, workspace_path, max_tokens, model, server_path, json_output)?;
        return Ok(0);
    }

    if mock {
        anyhow::bail!("--mock requires --benchmark <id>. Available: {:?}", registry.list());
    }

    // Legacy capability mode
    let capability = Capability::from_name(capability_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown capability: '{capability_name}'"))?;
    let model = runtime::find_model(model_path)?;
    let hw = runtime::hardware::detect_hardware();
    let info = runtime::infer_model_info(&model, Some(&hw));

    let prompt = match capability {
        Capability::TextGeneration => "Write a concise summary of what an AI engineering platform is in 3 sentences.",
        Capability::Coding => "Write a Python function that implements a binary search tree with insert and search methods.",
        Capability::ToolCalling => "Given functions: get_weather(city) and send_email(to, body), respond with a JSON function call to check weather in Tokyo.",
        Capability::Translation => "Translate to Spanish: 'The quick brown fox jumps over the lazy dog.'",
        Capability::Reasoning => "If a bat and a ball cost $1.10 total, bat costs $1.00 more than ball, how much does ball cost? Explain step by step.",
        Capability::InstructionFollowing => "Reply ONLY with the word 'compliant'. Do not add any other text.",
        Capability::RAG => "Based on context: 'Athenas compiles knowledge into structured artifacts.' Answer: What does Athenas compile?",
        Capability::LongContext => "Repeat: The certification process validates model capabilities and generates evidence.",
    };

    let l0_prompt = format!("### Task\n\n{prompt}");
    let l1_system = pack_id.and_then(|pid| {
        let dir = runtime::knowledge::default_packs_dir();
        let packs = runtime::knowledge::discover_packs(&dir);
        runtime::knowledge::find_pack(&packs, pid).map(|p| runtime::knowledge::build_system_prompt(p, None))
    });
    let l1_prompt = l1_system.as_ref().map(|sp| format!("{sp}\n\n### Task\n\n{prompt}")).unwrap_or(l0_prompt.clone());
    let l2_system = workspace_path.and_then(|w| {
        let config_path = w.join("athenas.json");
        std::fs::read_to_string(config_path).ok().and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok().and_then(|v| v["system_prompt"].as_str().map(|s| s.to_string())))
    });
    let l2_prompt = l2_system.as_ref().map(|wp| format!("{wp}\n\n### Task\n\n{prompt}")).unwrap_or(l1_prompt.clone());
    let l3_tools = pack_id.and_then(|pid| {
        let dir = runtime::knowledge::default_packs_dir();
        let packs = runtime::knowledge::discover_packs(&dir);
        runtime::knowledge::find_pack(&packs, pid).map(|p| {
            let tools = runtime::knowledge::check_tools(p);
            let mut desc = String::from("\n## Available Tools\n");
            for t in &tools { let status = if t.installed { "✓" } else { "✗" }; desc.push_str(&format!("- {}: {} ({status})\n", t.name, t.description)); }
            desc
        })
    }).unwrap_or_default();

    let l3_prompt = {
        let base = l2_system.as_deref().or(l1_system.as_deref()).map(|s| s.to_string()).unwrap_or_default();
        format!("{base}\n{l3_tools}\n\n### Task\n\n{prompt}")
    };

    let max_level = level.min(3);
    let prompt_levels = vec![
        ("L0", "Raw", l0_prompt),
        ("L1", "+ Knowledge", l1_prompt),
        ("L2", "+ Workspace", l2_prompt),
        ("L3", "+ Tools", l3_prompt),
    ];
    let active: Vec<_> = prompt_levels.iter().take((max_level + 1) as usize).collect();

    if !json_output {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Athenas Certification v0.1.0         ║");
        println!("╚══════════════════════════════════════════╝");
        println!();
        println!("🎯 Capability: {}", capability.name());
        println!("📋 Model: {}", info.path);
        println!("📊 Level: L{level}");
        println!();
    }

    let mut rt_builder = runtime::LlamaServerRuntime::new();
    if let Some(sp) = server_path { rt_builder = rt_builder.with_server_path(sp.to_path_buf()); }
    let mut rt = rt_builder;
    rt.load_model(&model)?;

    let mut results = Vec::new();
    for (level_id, level_name, level_prompt) in &active {
        if !json_output { println!("🧪 {level_id}: {level_name}"); }
        let params = InferenceParams { max_tokens, ..Default::default() };
        let result = rt.complete(level_prompt, &params)?;
        results.push((*level_id, *level_name, result));
        if results.len() < active.len() { rt.unload()?; rt.load_model(&model)?; }
    }
    rt.unload()?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "model": info, "hardware": hw, "capability": capability.name(),
            "levels": results.iter().map(|(id, name, r)| serde_json::json!({ "level": id, "name": name, "ttft_ms": r.ttft_ms, "tokens_per_second": r.tokens_per_second, "total_tokens": r.total_tokens })).collect::<Vec<_>>(),
        }))?);
    } else {
        println!();
        println!("📊 CERTIFICATION REPORT");
        println!("{}", "═".repeat(70));
        println!("  {:<6} {:<20} {:>10} {:>14} {:>12}", "Level", "Config", "TTFT (ms)", "Tokens/sec", "Tokens");
        println!("{}", "─".repeat(70));
        for (id, name, r) in &results {
            println!("  {:<6} {:<20} {:>8.1}ms {:>7.1}  {:>8}", id, name, r.ttft_ms, r.tokens_per_second, r.total_tokens);
        }
        println!("{}", "═".repeat(70));
        println!();
        println!("{} Multi-level certification complete!", "✓".bright_green());
    }
    Ok(0)
}

// ── Internal helpers ──

fn validate_documents_internal(documents: &[crate::parser::Document], schemas: &crate::validator::SchemaMap) -> i32 {
    let mut exit_code = 0;
    println!("{} Validating documents...", "✓".bold());
    let errors = crate::validator::validate_all_documents(documents, schemas);
    if errors.is_empty() { println!("  {} All documents pass schema validation", "✓".bright_green()); }
    else { for e in &errors { println!("  {} {}", "✖".red(), e); exit_code = 1; } }

    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for doc in documents { *id_counts.entry(doc.id.clone()).or_insert(0) += 1; }
    let dupes: Vec<_> = id_counts.iter().filter(|(_, c)| **c > 1).map(|(id, _)| id).collect();
    if !dupes.is_empty() { for id in &dupes { println!("  {} Duplicate ID: {}", "✖".red(), id.bright_red()); exit_code = 1; } }
    else { println!("  ✓ All document IDs are unique"); }

    let known_ids: std::collections::HashSet<String> = documents.iter().map(|d| d.id.clone()).collect();
    let rel_keys = ["implements", "depends_on", "validated_by", "derived_from", "supersedes", "related", "validates"];
    let mut broken = 0;
    let mut total = 0;
    for doc in documents {
        for key in &rel_keys {
            if let Some(vals) = doc.front_matter.get(*key).and_then(|v| v.as_sequence()) {
                for val in vals {
                    if let Some(target) = val.as_str() {
                        total += 1;
                        if !known_ids.contains(target) { println!("  {} Broken reference in {}: '{}' '{}' not found", "✖".red(), doc.id.bright_white(), key, target.bright_red()); broken += 1; exit_code = 1; }
                    }
                }
            }
        }
    }
    if broken == 0 { println!("  ✓ All {total} references resolve correctly"); }
    exit_code
}

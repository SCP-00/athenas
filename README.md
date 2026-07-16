# Athenas

An engineering platform for local artificial intelligence. Manages the complete lifecycle of local LLMs: discovery, profiling, benchmarking, certification, knowledge compilation, and execution across multiple runtimes.

---

## Philosophy

Large language models are commodities. Inference runtimes are commodities. The only lasting differentiation is the **engineering system** that surrounds them.

Athenas does not ship knowledge. Athenas ships the **ability to discover, validate, compile, and reuse knowledge** from live sources — man pages, CLI help output, language servers, package registries, and official documentation.

The model is one interchangeable component inside a larger engineering system. The platform's competitive advantage is not the model itself, but the environment it assembles around the model.

---

## The Problem

LLMs fail in production for reasons that have nothing to do with model quality:

- **Static knowledge.** Documentation shipped with a model becomes stale within weeks. The model doesn't know about Python 3.14, CUDA 13.5, or the latest compiler flags.
- **Environment mismatch.** The model has no awareness of the user's hardware, installed tools, available debuggers, or system capabilities.
- **Missing tools.** A model asked to fix a Go compilation error has no access to the Go compiler, the debugger, or the test runner. It guesses instead of verifying.
- **No reproducibility.** Two users running the same model on the same prompt get different results because their environments differ. Benchmarks are not comparable.
- **No measurement.** Without multi-layered benchmarks, there is no way to know whether a knowledge pack, a tool, or a workspace configuration actually improves outcomes.

---

## The Solution

Athenas structures the interaction between models and their environment as a **compilation pipeline**:

```
Sources (man pages, --help, LSP, APIs)
    ↓
Provider (discover → fetch → validate → normalize)
    ↓
Knowledge IR (intermediate representation)
    ↓
Compiler (compile → deduplicate → index)
    ↓
Knowledge Pack (versioned, cached, reproducible)
    ↓
Workspace (projection of system + tool + knowledge graphs)
    ↓
Certification (L0..L5 per-layer measurement)
    ↓
Agent (planning, tool execution, iterative repair)
```

Each layer is independently replaceable. The pipeline never changes — only the providers do.

---

## Architecture

### Engine Layers

| Layer | Component | Responsibility | Status |
|-------|-----------|----------------|--------|
| L0 | Doctor | Hardware detection, model discovery, capability enumeration | ✅ |
| L1 | Knowledge | Provider-based pack generation from live sources | ✅ |
| L2 | Workspace | Environment generation from project detection | 🚧 |
| L3 | Tools | Tool orchestration, permission, schema | 🚧 |
| L4 | Agent | Planning, iteration, self-correction | 🚧 |
| L5 | Experience | Procedural memory, skill caching | ❌ |

### Engine Layer — Doctor

Detects system capabilities, not just hardware. Reports installed languages, toolchains, debuggers, containers, and editors as a connected capability graph.

```
ath doctor
  🖥  CPU: Intel i5-13420H (12 cores, 16 threads)
  🎮 GPU: NVIDIA RTX 3050 6GB (driver 595.84)
  💾 RAM: 15.3 GB total (10.4 GB available)
  💻 OS: Kali Linux 2026 (x86_64)
  📦 Models: Qwen 4B (Q4_K_M), Qwen 9B (IQ3_XXS)
  🎯 Capabilities: text-generation, coding, tool-calling, reasoning
```

Use `--json` for machine-readable output.

### Engine Layer — Knowledge

The Knowledge Compiler transforms live documentation into structured, versioned, cached artifacts. It follows the same pipeline as the documentation compiler:

```
Markdown → Parser → AST → Graph → Artifacts
Sources  → Provider → IR  → Pack → Workspace
```

The trait that every provider implements:

```rust
trait KnowledgeProvider {
    fn id(&self) -> &str;
    fn discover(&self, query: &str) -> Result<Vec<KnowledgeSource>, String>;
    fn fetch(&self, source: &KnowledgeSource) -> Result<RawKnowledge, String>;
    fn validate(&self, raw: &RawKnowledge) -> Result<ValidationReport, String>;
    fn normalize(&self, raw: RawKnowledge) -> Result<KnowledgeIR, String>;
}
```

Providers never create packs. They emit KnowledgeIR. The compiler transforms IR into packs. This separation keeps providers reusable and the compiler extensible.

**Available providers:**

| Provider | Source | Status |
|----------|--------|--------|
| `man` | Unix man pages | ✅ |
| `help` | CLI --help output | 🚧 |
| `lsp` | Language server protocol | 🚧 |
| `doc` | Official documentation | ❌ |

Use `ath knowledge build <provider> <query>` to generate a pack:

```
ath knowledge build man go-build
```

This produces a structured build report showing extracted items by type, deduplication metrics, validation status, and compilation time.

### Engine Layer — Certification

Certification measures the contribution of each architectural layer independently. Instead of asking "how fast is this model?", it answers "how much does each layer improve this model?"

| Level | Configuration | Expected Improvement |
|-------|--------------|---------------------|
| L0 | Raw model | Baseline |
| L1 | + Knowledge Pack | +15-25% |
| L2 | + Workspace | +10-15% |
| L3 | + Tool Execution | +25-35% |
| L4 | + Iterative Repair | +5-15% |
| L5 | + Experience Cache | +5-10% |

Use `ath certify --pack <id>` to run a two-phase benchmark (raw vs packed) that measures the knowledge layer's contribution directly:

```
📊 COMPARISON: Raw vs Packed
Metric               Raw           Packed        Δ
TTFT (ms)             71.0           68.2      -3.9%
Tokens/sec            42.3           44.1
Generated tokens       100            100
```

---

## Commands

### Documentation Compiler

| Command | Description |
|---------|-------------|
| `ath build` | Full pipeline: validate → graph → artifacts (default) |
| `ath validate` | Schema validation, ID uniqueness, reference integrity |
| `ath graph` | Build knowledge graph from markdown documents |

### System

| Command | Description |
|---------|-------------|
| `ath doctor` | Detect hardware, discover models, list capabilities |
| `ath doctor --json` | Machine-readable hardware and model report |

### Inference

| Command | Description |
|---------|-------------|
| `ath run --prompt "..."` | Run inference via llama.cpp |
| `ath run --workspace workspace-go` | Run with workspace system prompt |
| `ath run -m model.gguf --json` | Structured JSON output |

### Certification

| Command | Description |
|---------|-------------|
| `ath certify` | Benchmark a model against a capability |
| `ath certify --pack go` | Two-phase benchmark (raw vs packed) |
| `ath certify --capability coding` | Test specific capability |

### Knowledge Packs

| Command | Description |
|---------|-------------|
| `ath pack list` | List all available knowledge packs |
| `ath pack show go` | Show pack details, tool status, benchmarks |
| `ath knowledge build man go-build` | Generate pack from man page |

### Workspaces

| Command | Description |
|---------|-------------|
| `ath workspace list` | List available workspaces |
| `ath workspace create go` | Generate workspace with system prompt |

---

## Project Layout

```
├── crates/
│   └── athenas-compiler/         # Single Rust crate
│       └── src/
│           ├── main.rs           # CLI entry point (ath)
│           ├── parser.rs         # Markdown front-matter parser
│           ├── validator.rs      # JSON Schema validation
│           ├── graph.rs          # Knowledge graph builder
│           ├── generators.rs     # Artifact generators
│           ├── lib.rs            # Utility library
│           └── runtime/
│               ├── mod.rs        # Runtime trait + LlamaServerRuntime
│               ├── hardware.rs   # Hardware auto-detection
│               ├── knowledge.rs  # KnowledgeProvider trait + compiler
│               ├── knowledge_ir.rs # Intermediate representation
│               └── providers/
│                   ├── mod.rs    # Provider module declarations
│                   └── man_provider.rs # Man page provider
├── schemas/                      # JSON Schema per document type (16)
├── knowledge/
│   ├── packs/                    # Static knowledge packs (YAML)
│   └── ontology/                 # Entity ontology definitions
├── .github/workflows/            # 8 CI/CD workflows
│   ├── validate.yml              # PR validation
│   ├── knowledge.yml             # Knowledge graph build
│   ├── rust.yml                  # Cargo build + test + clippy
│   ├── documentation.yml         # Doc site generation
│   └── ...
├── .state/                       # Machine-readable project state
├── CONST-0001.md                 # Project constitution
├── CURRENT_CONTEXT.md            # Daily agent context
├── BOOTSTRAP.md                  # Agent onboarding
├── knowledge.md                  # Project index
└── README.md                     # This file
```

---

## Design Principles

### Everything is a compiler.

The same pipeline — source → parser → IR → compilation → artifact — applies to documentation, knowledge, and eventually tools and workspaces. Consistency across domains makes the architecture teachable to future agents.

### Everything is reproducible.

Given the same sources and providers, Athena produces identical artifacts. Man pages are deterministic. `--help` output is deterministic. LSP responses are deterministic. Caches are portable.

### Everything is measurable.

Every compilation produces a `CompileMetrics` report: raw bytes, items extracted, validation errors, deduplication count, compilation time. Every certification produces a per-layer improvement delta. If a feature cannot produce a metric, it does not belong.

### Knowledge is generated, not shipped.

Static knowledge packs are compiled artifacts. They are generated, cached, versioned, and disposable. The canonical source is always a live provider — a man page, a CLI, an API. The repository never contains knowledge that can become stale.

### Artifacts are disposable; sources are truth.

Generated packs, search indexes, and knowledge graphs are never edited by hand. They are rebuilt from sources. This guarantees consistency and prevents documentation rot.

### Providers emit IR; compilers emit artifacts.

Providers never know about KnowledgePack, YAML, or search indexes. They emit `KnowledgeIR`. The compiler transforms `KnowledgeIR` into artifacts. New providers can be added without modifying the compiler. New artifact formats can be added without modifying providers.

---

## Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| M0 | Foundation: Constitution, architecture, compiler specification | ✅ |
| M1 | Documentation compiler: validate, graph, build, artifacts | ✅ |
| M2 | Runtime spike: `ath run` with llama.cpp integration | ✅ |
| M3 | Certification engine: `ath doctor`, `ath certify` | ✅ |
| M4 | Knowledge engine: KnowledgeProvider trait, ManProvider | ✅ |
| M5 | Workspace engine: `ath workspace`, environment generation | 🚧 |
| M6 | Tool engine: unified tool registry, permissions, schemas | 🚧 |
| M7 | Agent engine: planning, iteration, self-correction | 🚧 |
| M8 | Dashboard: Astro-based visualization of all subsystems | 🚧 |
| M9 | Experience engine: procedural memory, skill caching | ❌ |

---

## Contributing

Athenas follows a strict engineering process documented in `CONST-0001` and `DIRECTIVE-0001`. Key rules:

- **Knowledge precedes implementation.** Never write code without a specification.
- **Evidence outweighs intuition.** Back every claim with data.
- **Traceability is absolute.** Reference by ID, never by name.
- **No optimization without measurement.** Baseline first, then optimize.
- **Capabilities over implementations.** Depend on interfaces, not tools.
- **Architecture must survive technology changes.** Design for longevity.
- **Every experiment becomes knowledge.** Never discard data.
- **Write for humans and agents.** Machine-readable metadata in every document.

Read `BOOTSTRAP.md` first — it contains the complete onboarding for new contributors.

---

## License

MIT

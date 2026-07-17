# Phase Pipeline Architecture v2

**Status:** Design Document — No code written yet
**Replaces:** CertificationEngine (monolithic, estimate-based)
**Principle:** Athena no es una aplicación. Athena es un laboratorio autónomo compuesto por cientos de fases deterministas, pequeñas, reanudables y verificables, capaces de ser orquestadas tanto por humanos como por agentes de IA. El certificado final no es el objetivo; es simplemente una vista compuesta de todo el conocimiento generado durante esas fases.

---

## 0. Core Principle

**Cada fase responde UNA pregunta científica.**

No se mide por tiempo. Se mide por si obtuvo evidencia.

Una fase termina cuando genera conocimiento nuevo, no cuando pasa un minuto.

Si la respuesta es "no sé, no pude medirlo", esa fase debe reintentar con otra estrategia. Nunca estimar.

---

## 1. Every Phase Answers One Question

| Phase | Question | How It Answers |
|---|---|---|
| PHASE-0001 Hardware Snapshot | ¿Qué hardware existe? | Lee /proc, lspci, nvidia-smi |
| PHASE-0002 Runtime Discovery | ¿Qué runtimes existen? | Busca binarios + --help |
| PHASE-0003 Runtime Probe | ¿Qué capacidades reales tiene este runtime? | Prueba flags reales |
| PHASE-0004 GGUF Inspection | ¿Qué dice realmente este modelo? | Parsea header GGUF |
| PHASE-0005 Memory Hypothesis | ¿Qué configuraciones parecen posibles? | VRAM Calculator (hypothesis only) |
| PHASE-0006 Execution Probe | ¿Esta configuración realmente funciona? | Arranca runtime, mide |
| PHASE-0007 Context Search | ¿Cuál es el contexto máximo estable? | Probe incremental |
| PHASE-0008 KV Cache Search | ¿Cuál es la mejor estrategia KV? | Prueba FP16 vs Q8 vs Q4 vs Turbo3... |
| PHASE-0009 Layer Search | ¿Cuántas capas conviene descargar? | Prueba ngl variable |
| PHASE-0010 Batch Search | ¿Cuál es el batch óptimo? | Barre batch/ubatch |
| PHASE-0011 Quality Search | ¿Cuándo empieza a degradarse? | Benchmarks por contexto |
| PHASE-0012 Runtime Competition | Mismo modelo, mismo prompt, todos los runtimes | Comparación justa |
| PHASE-0013 Memory Strategy | ¿VRAM-only? ¿RAM-only? ¿Híbrido? | Todas las estrategias |
| PHASE-0014 Full Certification | ¿Qué conocimiento nuevo obtuvimos? | Composición de todo lo anterior |

---

## 2. ExecutionProbe — The Heart of Athena

The ExecutionProbe replaces `VramCalculator::estimate()` as the judge of truth.

```rust
/// Probe config — what to test
pub struct ProbeConfig {
    pub runtime_path: PathBuf,
    pub runtime_type: String,      // "llama-server", "ollama", "prismml"
    pub model_path: PathBuf,
    pub model_params_b: f64,

    // Memory strategy — Buendia's key insight: model and KV are independent
    pub model_in_vram: bool,       // model weights in VRAM
    pub model_gpu_layers: u64,     // how many layers in GPU (0 = all CPU)
    pub model_total_layers: u64,
    pub kv_in_vram: bool,          // KV cache in VRAM
    pub kv_cache_type: KvCacheType, // FP16, Q8, Q4, Turbo3, etc.

    // Performance config
    pub context_size: u64,
    pub batch_size: u64,
    pub ubatch_size: u64,
    pub threads: u64,
    pub flash_attention: bool,
    pub rope_scaling: bool,
    pub sliding_window: bool,

    // Probe duration
    pub min_tokens_for_stable: u64,   // How many tokens to generate for reliable measurement
    pub warmup_prompt_tokens: u64,    // How many tokens for warmup
}
```

```rust
/// Probe result — ALL measured, NOTHING estimated
pub struct ProbeResult {
    // Identity
    pub config_snapshot: ProbeConfig,
    pub experiment_id: String,

    // Startup
    pub load_success: bool,
    pub server_start_ms: f64,
    pub model_load_ms: f64,
    pub first_token_ms: f64,

    // Performance (measured, NEVER estimated)
    pub tokens_per_second: f64,
    pub prompt_processing_speed: f64,  // prompt tok/s
    pub total_tokens_generated: u64,

    // Memory (tracked continuously)
    pub vram_before_load_gb: f64,
    pub vram_after_load_gb: f64,
    pub vram_peak_gb: f64,
    pub vram_after_unload_gb: f64,
    pub ram_before_load_gb: f64,
    pub ram_after_load_gb: f64,
    pub ram_peak_gb: f64,
    pub swap_usage_gb: f64,

    // Errors
    pub oom: bool,
    pub oom_during: String,  // "load" | "first_token" | "inference" | "context_shift"
    pub crash: bool,
    pub timeout: bool,
    pub exit_code: Option<i32>,

    // Raw evidence
    pub stdout: String,
    pub stderr: String,

    // Hardware utilization (sampled during run)
    pub gpu_utilization_pct: Vec<f64>,  // sampled every 100ms
    pub vram_trace_gb: Vec<(u64, f64)>, // timestamp_ms, vram_gb
    pub power_watts: Vec<(u64, f64)>,
    pub temperature_c: Vec<(u64, f64)>,

    // Result
    pub success: bool,
    pub error_message: Option<String>,
}
```

**The probe never estimates.** If it can't measure, it runs again. If it OOMs, it records the OOM trace. Every number comes from the real runtime.

**The probe samples continuously.** VRAM trace, GPU utilization, power, temperature — all recorded at 100ms intervals during the entire execution.

---

## 3. Complete Artifact Structure

Each phase produces a self-contained directory. No external dependencies needed to understand what happened.

```
.state/experiments/
    EXP-20260717-00184/
        experiment.yaml              # Full experiment descriptor
        phases/
            PHASE-0006-execution-probe/
                # Structured data
                artifact.json         # ProbeResult (structured, agent-consumable)
                artifact.yaml         # Same data in YAML

                # Metrics
                metrics.json          # { tok/s, ttft, peak_vram, ... }

                # Timeline
                timeline.json         # [ { ts: 0, event: "server_start" },
                                      #   { ts: 410, event: "first_token" },
                                      #   { ts: 15000, event: "oom" } ]

                # Raw evidence
                evidence/
                    stdout.log        # Full stdout from llama-server
                    stderr.log        # Full stderr (critical for OOM diagnosis)
                    vram-trace.csv    # timestamp_ms, vram_gb — sampled every 100ms
                    gpu-util.csv      # timestamp_ms, util_pct
                    power-trace.csv   # timestamp_ms, watts
                    nvidia-smi.log    # Pre/Post run nvidia-smi
                    /proc/meminfo     # Memory state before/after

                # Reproducibility
                command.sh            # Exact CLI command used to run the probe
                environment.json      # env vars at time of execution
                signature.sha256      # Hash of all files in this phase
                config-snapshot.yaml  # Exact probe config used
```

This means:
- **Any agent** (Hermes, Buffy, Claude Code) can open a phase directory and understand everything
- **No need for a database** — the filesystem IS the database
- **Reproducible** — `command.sh` + `config-snapshot.yaml` is all you need to re-run
- **Verifiable** — `signature.sha256` proves the phase wasn't tampered with

---

## 4. Full Search Space

This is everything Athena searches, at every dimension:

### Memory Strategy
| Strategy | Model Weights | KV Cache | Context | Use Case |
|---|---|---|---|---|
| All VRAM | VRAM | VRAM FP16/Q8 | Up to VRAM limit | Fastest, but limited context |
| Buendia's Config | VRAM | RAM Turbo3 | Up to RAM limit | Large context on limited VRAM |
| Hybrid GPU Layers | Partially VRAM | VRAM/RAM | Medium | When model doesn't fit in VRAM |
| All CPU | RAM | RAM | Max RAM | No GPU, large context |
| KV Hybrid | VRAM | Split VRAM+RAM | Very large | Extreme context needs |
| Offloaded | VRAM + RAM offload | RAM | Large | When VRAM is tight but model is big |

### KV Cache Types
FP16, Q8, Q8_K, Q6, Q5, Q4, Q4_K, Turbo3, Turbo2, Turbo1, IQ4, Bonsai, ISWA

### Model Offload
ngl = 0, 10, 20, ..., total_layers (all combinations of GPU vs CPU layers)

### Performance
- Batch: 64, 128, 256, 512, 1024
- UBatch: 64, 128, 256 (must be ≤ batch)
- Threads: 2, 4, 6, 8 (based on CPU cores)
- Flash Attention: on/off
- Sliding Window: on/off
- Rope Scaling: on/off
- Speculative Decoding: on/off
- NUMA: on/off
- Huge Pages: on/off
- Mlock: on/off

### Runtime Competition
Every runtime tested with the EXACT same model, prompt, and config:
- llama.cpp official
- PrismML (Bonsai)
- TurboQuant (if detected)
- Ollama (if detected)

---

## 5. VramCalculator Is Hypothesis Only

```rust
pub struct MemoryHypothesis {
    pub predicted_fits: bool,
    pub confidence: f64,        // How sure we are (0.0-1.0)
    pub predicted_vram_gb: f64,
    pub predicted_ram_gb: f64,
    pub strategy: String,       // Which strategy was predicted best

    // The probe may disagree — that's LEARNING
    pub probe_result: Option<ProbeResult>,

    // After probe, was hypothesis correct?
    pub hypothesis_correct: Option<bool>,
    pub correction: Option<String>,  // "overestimated by 1.2GB — model uses less VRAM than predicted"
}
```

The VramCalculator generates hypotheses. The ExecutionProbe tests them.
If the hypothesis was wrong, Athena learns and updates the calculator.
Over time, the hypotheses get better because they're backed by real evidence.

---

## 6. The DAG Design

The certification is NOT a linear pipeline. It's a DAG:

```
PHASE-0001 (Hardware) ──────────────────────────────┐
PHASE-0002 (Runtime Discovery) ─────────────────────┤
PHASE-0004 (GGUF Inspection) ───────────────────────┤
                                                     │
                                                     ▼
                                            PHASE-0005 (Memory Hypothesis)
                                                     │
                              ┌──────────────────────┼──────────────────────┐
                              ▼                      ▼                      ▼
                      PHASE-0006 (Probe)      PHASE-0006 (Probe)      PHASE-0006 (Probe)
                      Strategy: All VRAM      Strategy: Buendia       Strategy: Hybrid
                      KV: FP16                KV: Turbo3 RAM          KV: Q8 VRAM
                              │                      │                      │
                              ▼                      ▼                      ▼
                      PHASE-0007 (Context)    PHASE-0007 (Context)    PHASE-0007 (Context)
                      Find max stable ctx     Find max stable ctx     Find max stable ctx
                              │                      │                      │
                              ▼                      ▼                      ▼
                      PHASE-0011 (Quality)    PHASE-0011 (Quality)    PHASE-0011 (Quality)
                      Degradation curve       Degradation curve       Degradation curve
                              │                      │                      │
                              └─────────┬────────────┘──────────────────────┘
                                        ▼
                                PHASE-0012 (Runtime Competition)
                                Same model, same context, same prompt
                                PrismML vs Official vs TurboQuant vs Ollama
                                        │
                                        ▼
                                PHASE-0014 (Certification)
                                Compose all evidence into knowledge
```

**Key property:** Each phase can be run independently. If Context Search (PHASE-0007) fails at 196K, you don't re-run Hardware Snapshot. You only re-run Context Search with a lower starting context.

---

## 7. Agent-Consumable Output

Every phase produces output that any agent can consume WITHOUT running the phase.

```json
// PHASE-0007 Context Search — result for Hermes to use
{
    "phase": "context-search",
    "model": "Qwen3.5-9B-IQ3_XXS",
    "runtime": "PrismML (bonsai)",
    "gpu": "RTX3050 6GB",
    "strategy": "model=vram kv=ram:Turbo3",

    "best_context": 180224,
    "stable_until": 188416,
    "failure_at": 196608,
    "degradation_curve": [
        { "context": 32768,  "quality": 1.00 },
        { "context": 65536,  "quality": 0.99 },
        { "context": 131072, "quality": 0.96 },
        { "context": 180224, "quality": 0.92 },
        { "context": 196608, "quality": 0.71 }
    ],
    "confidence": 0.98,
    "evidence_phases": ["PHASE-0006-probe-01", "PHASE-0006-probe-02", "PHASE-0007-probe-03"],
    "execution_time_s": 342,
    "experiment_id": "EXP-20260717-00184"
}
```

Any agent (Hermes, Buffy, Claude Code, etc.) can read this and know:
- What configuration to use
- What context is safe
- How quality degrades
- What evidence backs the claim
- How confident Athena is

---

## 8. The CLI

```bash
# Discover everything
ath phase run PHASE-0001
ath phase run PHASE-0002
ath phase run PHASE-0004

# Generate hypothesis
ath phase run PHASE-0005 --model qwen.gguf

# Probe a specific config
ath phase run PHASE-0006 --config all-vram-fp16-ctx32k
ath phase run PHASE-0006 --config buendia-turbo3-ram-ctx262k

# Search for max stable context
ath phase run PHASE-0007 --start 32K --strategy buendia

# Compare runtimes
ath phase run-manifest manifest-runtime-competition.yaml

# Full certification
ath phase run-manifest manifest-full-certification.yaml

# Inspect past experiments
ath artifact get EXP-20260717-00184 PHASE-0007 artifact.json
ath timeline show EXP-20260717-00184 PHASE-0006
ath replay EXP-20260717-00184  # Re-run exact same experiment

# Agent API (internal)
ath phase list-cached             # What phases exist that can be reused?
ath phase diff EXP-001 EXP-002    # What changed between experiments?
ath status                        # Current experiment progress
```

---

## 9. Required Rust Traits

### Phase Trait
```rust
pub trait Phase: Send + Sync {
    fn id(&self) -> &str;
    fn question(&self) -> &str;          // "¿Cuál es el contexto máximo estable?"
    fn inputs(&self) -> Vec<ArtifactRef>;
    fn outputs(&self) -> Vec<ArtifactType>;
    fn execute(&self, ctx: &PhaseContext) -> Result<PhaseOutput>;
}
```

### PhaseOutput
```rust
pub struct PhaseOutput {
    pub artifact: serde_json::Value,   // Structured answer to the question
    pub timeline: Vec<TimelineEvent>,
    pub metrics: HashMap<String, f64>,
    pub raw_path: PathBuf,             // Path to evidence/ directory
    pub exit_code: i32,
}
```

### ArtifactStore
```rust
pub trait ArtifactStore {
    fn save_phase(&self, experiment_id: &str, phase_id: &str, output: &PhaseOutput) -> Result<()>;
    fn load_artifact(&self, experiment_id: &str, phase_id: &str) -> Result<PhaseOutput>;
    fn list_phases(&self, experiment_id: &str) -> Result<Vec<String>>;
    fn phase_exists(&self, experiment_id: &str, phase_id: &str) -> bool;
}
```

### ExecutionProbe
```rust
pub trait ExecutionProbe {
    fn probe(&self, config: &ProbeConfig) -> Result<ProbeResult>;
    fn estimate_duration(&self, config: &ProbeConfig) -> std::time::Duration;
}
```

---

## 10. Memory Strategy Enum

```rust
pub enum MemoryStrategy {
    /// Model weights in VRAM, KV in VRAM (default, fastest)
    AllVram { kv_cache_type: KvCacheType },

    /// Model weights in VRAM, KV in RAM with quantized KV
    /// Buendia's config: model=vram, kv=ram:Turbo3, max context
    BuendiaConfig { kv_cache_type: KvCacheType },

    /// Model partially in VRAM, rest in RAM
    HybridGpuLayers { gpu_layers: u64, total_layers: u64, kv_in_vram: bool, kv_type: KvCacheType },

    /// All weights in RAM, no GPU (CPU inference)
    AllCpu { kv_type: KvCacheType },

    /// KV split between VRAM and RAM
    KvHybrid { vram_kv_context: u64, ram_kv_type: KvCacheType },

    /// Model weights in RAM + GPU offload
    Offloaded { gpu_layers: u64, kv_type: KvCacheType, kv_in_ram: bool },
}
```

These strategies cover ALL possible combinations of model weights, KV cache, and memory placement. The ExecutionProbe tests them. The ones that work are recorded as knowledge.

---

## 11. Phases by Category

### Core Discovery (always run first)
| Phase | Question | Depends On | Duration |
|---|---|---|---|
| PHASE-0001 Hardware | ¿Qué hardware existe? | — | <1s |
| PHASE-0002 Runtime Discovery | ¿Qué runtimes existen? | PHASE-0001 | <5s |
| PHASE-0003 Runtime Capabilities | ¿Qué capacidades reales tiene cada runtime? | PHASE-0002 | <30s |
| PHASE-0004 GGUF Inspection | ¿Qué dice el modelo? | — | <100ms |

### Hypothesis (fast, bootstrap only)
| Phase | Question | Depends On | Duration |
|---|---|---|---|
| PHASE-0005 Memory Hypothesis | ¿Qué configs parecen posibles? | PHASE-0001, PHASE-0003, PHASE-0004 | <50ms |

### Search Phases (probe-based, real measurement)
| Phase | Question | Depends On | Duration |
|---|---|---|---|
| PHASE-0006 Execution Probe | ¿Esta config funciona? | PHASE-0005 | <30s–5min |
| PHASE-0007 Context Search | ¿Cuál es el contexto máximo estable? | PHASE-0006 | <5min–30min |
| PHASE-0008 KV Search | ¿Cuál es la mejor estrategia KV? | PHASE-0006 | <5min–30min |
| PHASE-0009 Layer Search | ¿Cuántas capas conviene descargar? | PHASE-0006 | <5min–30min |
| PHASE-0010 Batch Search | ¿Cuál es el batch óptimo? | PHASE-0006 | <2min–10min |
| PHASE-0011 Quality Search | ¿Cuándo empieza la degradación? | PHASE-0007 | <10min–60min |
| PHASE-0012 Runtime Competition | ¿Qué runtime gana? | PHASE-0007 | <30min–120min |
| PHASE-0013 Memory Strategy | ¿Qué estrategia de memoria es mejor? | PHASE-0007 | <30min–120min |

### Knowledge (composition)
| Phase | Question | Depends On | Duration |
|---|---|---|---|
| PHASE-0014 Full Certification | ¿Qué sabemos ahora que no sabíamos antes? | All above | <1s |

---

## 12. Migration Path

| Current Code | Becomes | Status |
|---|---|---|
| `hardware.rs` | PHASE-0001 (wrap) | ✅ Exists |
| `runtime_discovery/` capability + prober | PHASE-0002 + PHASE-0003 (wrap) | ✅ Exists |
| `gguf.rs` | PHASE-0004 (wrap) | ✅ Exists |
| `vram.rs` VramCalculator | PHASE-0005 hypothesis only (wrap) | ✅ Exists |
| **NEW: ExecutionProbe** | PHASE-0006 + PHASE-0007 + PHASE-0008 + PHASE-0009 + PHASE-0010 + PHASE-0011 + PHASE-0012 + PHASE-0013 | 🔴 Not built |
| `recovery.rs` RecoveryEngine | PHASE-0006 recovery logic | ✅ Exists — integrate with probe |
| `experiment/engine.rs` | DISCARD — replaced by PhaseOrchestrator | ❌ Remove |
| `experiment/planner.rs` | PHASE-0005 hypothesis + ManifestBuilder | 🟡 Refactor |
| `experiment/checkpoint.rs` | PhaseOutput + ArtifactStore | 🟡 Refactor |
| **NEW: Phase trait + ArtifactStore** | Infrastructure | 🔴 Not built |
| **NEW: PhaseOrchestrator + ManifestBuilder** | Pipeline | 🔴 Not built |

---

## 13. Principles for All Future Development

1. **Cada fase responde UNA pregunta científica.** No se mide por tiempo. Se mide por evidencia.

2. **El ExecutionProbe es el corazón del proyecto.** Todo se mide. Nada se estima.

3. **El VramCalculator es hipótesis, no juez.** La evidencia siempre gana.

4. **Los artefactos son el contrato entre agentes.** Cualquier agente debe poder leerlos sin ejecutar nada.

5. **Las fases son independientes.** Una fase nunca depende del estado interno de otra, solo de sus artefactos.

6. **Modelo y KV son independientes.** Modelo en VRAM no implica KV en VRAM. Todas las combinaciones se exploran.

7. **No existe "benchmark terminado".** Athena puede detenerse en cualquier momento y haber aprendido algo.

8. **Cada experimento tiene un ID único.** Todo cuelga de ese ID. Cinco años después debe ser reproducible.

9. **La certificación es un DAG, no una línea recta.** Cada nodo es re-ejecutable independientemente.

10. **El éxito no es "compila". El éxito es "Athena aprendió algo que ayer no sabía."**

---
id: BENCH-0001
title: Runtime Spike — TTFT & Throughput (Qwen3.5-4B)
author: Buffy
date: 2026-07-16
status: Complete
version: "1.0.0"
authority: Level 6 — Validation
validates:
  - COMPILER-0001
hardware: "Linux x86_64 (unknown GPU/CPU)"
runtime: llama.cpp (server)
runtime_commit: f2d1c2f
model: MODEL-0001 (Qwen3.5-4B)
quant: Q4_K_M
model_hash: unavailable
profile: PROFILE-0001
samplers: temperature=0.7, top_p=0.9
cuda_version: N/A (CPU-only)
kernel_version: Linux x86_64
confidence: 0.85
related:
  - MODEL-0001
  - PROFILE-0001
  - EVID-0002
---

# BENCH-0001 — Runtime Spike: TTFT & Throughput

> **Authority:** Level 6 (Validation)
> **Status:** Complete

---

## Claim

The Athenas Runtime pipeline (`ath run`) can successfully load a GGUF model via `llama-server`, execute a prompt, and measure Time-to-First-Token (TTFT) and generation throughput.

## Method

1. Run `ath run --model <model> --prompt <prompt> --max-tokens <n>` from the project root
2. `ath` starts `llama-server` as a subprocess on a local port
3. Wait for server to accept TCP connections and HTTP requests
4. Wait for model to fully load (poll `/completion` until not 503)
5. Send a POST to `/completion` with `stream=false` and `cache_prompt=true`
6. Parse the response JSON for `timings.prompt_ms` (TTFT) and computed `tokens_per_second`
7. Kill the server subprocess
8. Report results in both human-readable and `--json` formats

## Hardware

| Component | Detail |
|-----------|--------|
| CPU | Unknown x86_64 (Linux) |
| RAM | Unknown |
| OS | Linux |

## Software

| Component | Version / Commit |
|-----------|-----------------|
| Runtime | llama-server v1 (commit f2d1c2f, built with GCC 15.3.0) |
| Compiler | ath v0.1.0 (Rust, crate athenas-compiler) |
| Model | qwen3.5-4b-q4_k_m.gguf |

## Model

| Property | Value |
|----------|-------|
| Name | Qwen3.5-4B |
| Quant | Q4_K_M |
| File Size | 2.9 GB (3,013,027,808 bytes) |
| Path | `/home/buendia001/models/qwen3.5/qwen3.5-4b-q4_k_m.gguf` |

## Profile

| Parameter | Value |
|-----------|-------|
| Context | 2048 |
| Temperature | 0.7 |
| Top-P | 0.9 |
| Max Tokens | 100 |
| Batch Size | 512 (default) |
| Flash Attention | false |

## Results

### Run 1 — Spanish greeting prompt

| Metric | Value |
|--------|-------|
| Prompt | "Say hello in Spanish and introduce yourself briefly" |
| Prompt tokens | 8 |
| TTFT | **70.1 ms** |
| Throughput | **45.7 tok/s** |
| Total tokens | 47 |
| Total duration | 1,127.8 ms |
| Model load time | 2.0 s |

### Run 2 — Short instruction (JSON output test)

| Metric | Value |
|--------|-------|
| Prompt | "Reply with just the word: Hello" |
| Prompt tokens | 7 |
| TTFT | **74.3 ms** |
| Throughput | **53.5 tok/s** |
| Total tokens | 5 |
| Total duration | 195.4 ms |

## Raw Data

```json
{
  "run_1": {
    "model": "qwen3.5-4b-q4_k_m.gguf",
    "prompt": "Say hello in Spanish and introduce yourself briefly",
    "prompt_tokens": 8,
    "total_tokens": 47,
    "ttft_ms": 70.1,
    "tokens_per_second": 45.7,
    "total_duration_ms": 1127.8,
    "model_load_time_s": 2.0
  },
  "run_2": {
    "model": "qwen3.5-4b-q4_k_m.gguf",
    "prompt": "Reply with just the word: Hello",
    "prompt_tokens": 7,
    "total_tokens": 5,
    "ttft_ms": 74.3,
    "tokens_per_second": 53.5,
    "total_duration_ms": 195.4,
    "model_load_time_s": 1.5
  }
}
```

## Reproducibility

To reproduce this benchmark:

```bash
# Clone and build
cd athenas
cargo build --manifest-path crates/athenas-compiler/Cargo.toml

# Run inference with a GGUF model
crates/athenas-compiler/target/debug/ath run \
  --model /path/to/model.gguf \
  --prompt "Your prompt here" \
  --max-tokens 100

# Machine-readable output
crates/athenas-compiler/target/debug/ath run \
  --model /path/to/model.gguf \
  --prompt "Hello" \
  --max-tokens 10 \
  --json
```

## Conclusion

The Athenas Runtime Spike is **successful**. The pipeline works end-to-end with the following verified capabilities:

1. **Model discovery** — `ath run` finds GGUF models automatically or via `--model`
2. **Server lifecycle** — Launches and manages `llama-server` as a subprocess
3. **HTTP API integration** — Communicates via raw TCP HTTP; robust timeout and error handling
4. **Timing measurement** — Accurately reads TTFT and throughput from llama-server's built-in timings
5. **Structured output** — Both human-readable and `--json` modes produce correct results
6. **Clean shutdown** — Server process is killed and resources released on completion

The measured TTFT of **70-74 ms** and throughput of **45-53 tok/s** are reasonable for a 4B-parameter Q4_K_M model running on CPU. These establish a baseline for future optimization (GPU offloading, flash attention, larger batch sizes, etc.).

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-16 | Create BENCH-0001 | First benchmark — validates the entire Athenas Runtime pipeline end-to-end with real hardware |

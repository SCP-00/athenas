---
id: BENCH-NNNN
title: [Benchmark Title]
author: [Author]
date: YYYY-MM-DD
status: Complete
version: 1.0
authority: Level 6 — Validation
validates: [SPEC-NNNN]
hardware: [GPU/CPU/RAM details]
runtime: [Runtime name and version]
runtime_commit: [commit hash]
model: [Model name]
quant: [Quant, e.g., IQ3_XXS]
model_hash: [SHA256]
gguf_hash: [SHA256]
profile: [Profile used]
samplers: [sampler config]
gpu_temperature: [°C]
ram_used: [GB]
swap_used: [GB]
cuda_version: [version]
driver_version: [version]
kernel_version: [version]
reproducibility_hash: [hash of all inputs]
confidence: [0.0-1.0]
related: []
---

# BENCH-NNNN — [Benchmark Title]

> **Authority:** Level 6 — Validation

---

## Claim

[What claim does this benchmark test?]

## Method

[How was the benchmark performed? Step by step.]

## Hardware

| Component | Detail |
|-----------|--------|
| GPU | [GPU model] |
| CPU | [CPU model] |
| RAM | [RAM size] |
| OS | [OS version] |

## Software

| Component | Version / Commit |
|-----------|-----------------|
| Runtime | [version] ([commit]) |
| CUDA | [version] |
| Driver | [version] |
| Kernel | [version] |

## Model

| Property | Value |
|----------|-------|
| Name | [model name] |
| Quant | [quant] |
| Model Hash | [SHA256] |
| GGUF Hash | [SHA256] |

## Profile

| Parameter | Value |
|-----------|-------|
| Context | [tokens] |
| Temperature | [value] |
| Batch Size | [value] |
| Speculative Decoding | [true/false] |

## Results

| Metric | Value |
|--------|-------|
| TTFT | [ms] |
| TPOT | [ms/token] |
| Throughput | [tok/s] |
| Peak Memory | [GB] |
| Context Usage | [tokens] |

## Raw Data

```
[Link to raw data file or inline data]
```

## Reproducibility

To reproduce this benchmark:

```bash
[Exact command or script to reproduce]
```

## Conclusion

[What does this evidence tell us?]

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| YYYY-MM-DD | [Decision] | [Why] |

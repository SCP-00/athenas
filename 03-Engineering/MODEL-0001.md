---
id: MODEL-0001
title: Qwen3.5-4B
author: Buffy
date: 2026-07-16
status: Registered
version: "1.0.0"
authority: Level 5 — Implementation
architecture: Transformer
parameters: 4
license: Apache 2.0
related:
  - BENCH-0001
  - PROFILE-0001
  - EVID-0002
---

# MODEL-0001 — Qwen3.5-4B

> **Authority:** Level 5 (Implementation)
> **Status:** Registered

---

## Overview

| Property | Value |
|----------|-------|
| Name | Qwen3.5-4B |
| Architecture | Transformer (Attention: Qwen2, MLP: SwiGLU) |
| Parameters | 4.0B |
| Context Length | 32,768 tokens |
| License | Apache 2.0 |
| Source | https://huggingface.co/Qwen/Qwen3.5-4B |

## Available Quants

| Quant | File Size | BLAS | Source |
|-------|-----------|------|--------|
| Q4_K_M | 2.9 GB | Yes | Local (`/home/buendia001/models/qwen3.5/qwen3.5-4b-q4_k_m.gguf`) |
| Q4_0 | ~2.7 GB | Yes | HuggingFace |
| Q8_0 | ~4.5 GB | Yes | HuggingFace |

## Capabilities

| Capability | Score | BENCH Reference |
|------------|-------|-----------------|
| Text Generation | 45.7 tok/s | BENCH-0001 |
| TTFT (cold) | 70.1 ms | BENCH-0001 |
| TTFT (cached) | — | TBD |
| Multilingual (ES) | ✅ | BENCH-0001 |
| Instruction Following | ✅ | BENCH-0001 |

## Hardware Compatibility

| Hardware | Status | Notes |
|----------|--------|-------|
| CPU-only (Linux x86_64) | Compatible | Tested on AMD64, 2.9 GB RAM for Q4_K_M |
| GPU (CUDA) | Likely Compatible | Requires ~3.5 GB VRAM for Q4_K_M with ctx=32K |
| GPU (Metal) | Likely Compatible | Requires macOS with 4 GB+ unified memory |

## Profiles

| Profile | Purpose | Status |
|---------|---------|--------|
| PROFILE-0001 | Default spike profile | Active |

## Certification

| Decision | Date | Evidence |
|----------|------|----------|
| SPIKE-APPROVED | 2026-07-16 | BENCH-0001 — End-to-end pipeline validated |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-16 | Register MODEL-0001 | First model validated through the Athenas Runtime Spike pipeline |

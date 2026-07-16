---
id: PROFILE-0001
title: Default Spike Profile
author: Buffy
date: 2026-07-16
status: Active
version: "1.0.0"
authority: Level 5 — Implementation
runtime: llama.cpp (server v1, commit f2d1c2f)
model: MODEL-0001
related:
  - BENCH-0001
  - EVID-0002
---

# PROFILE-0001 — Default Spike Profile

> **Authority:** Level 5 (Implementation)
> **Status:** Active

---

## Purpose

General-purpose inference profile used for the first Runtime Spike. Optimized for quick validation of the Athenas → llama.cpp pipeline rather than maximum performance.

## Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| max_context | 2048 | Limited for spike; model supports 32K |
| temperature | 0.7 | Default creative temperature |
| top_p | 0.9 | Default nucleus sampling |
| n_predict | 512 | Max tokens per generation |
| batch_size | 512 | Default llama.cpp prompt processing batch |
| flash_attention | false | Not enabled for spike |

## Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 4 GB | 8 GB |
| Storage | 3 GB | 10 GB (for additional models) |

## Validation

- [x] Validated against BENCH-0001
- [x] TTFT acceptable (70.1 ms)
- [x] Generation speed acceptable (45.7 tok/s)
- [x] End-to-end pipeline: Markdown → Compiler → Runtime → Model → Response → Evidence

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-16 | Create PROFILE-0001 | First profile — validates the Runtime pipeline end-to-end |

---
id: PROFILE-NNNN
title: [Profile Name]
author: [Author]
date: YYYY-MM-DD
status: Draft
version: 1.0
authority: Level 5 — Implementation
runtime: [Runtime name]
model: [Target model]
related: []
---

# PROFILE-NNNN — [Profile Name]

> **Authority:** Level 5 (Implementation)

---

## Purpose

[What use case is this profile optimized for?]

## Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| max_context | [tokens] | |
| temperature | [value] | |
| top_p | [value] | |
| top_k | [value] | |
| repeat_penalty | [value] | |
| frequency_penalty | [value] | |
| presence_penalty | [value] | |
| batch_size | [value] | |
| threads | [value] | |
| speculative_decoding | [true/false] | |
| grammar | [path or inline] | |
| cache_type | [type] | |
| flash_attention | [true/false] | |
| kv_cache_quant | [type] | |

## Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| GPU Memory | [GB] | [GB] |
| RAM | [GB] | [GB] |
| VRAM | [GB] | [GB] |

## Validation

- [ ] Validated against BENCH-NNNN
- [ ] TTFT acceptable
- [ ] Memory within limits
- [ ] Context length verified

## Benchmarks

| BENCH | Metric | Result | Target |
|-------|--------|--------|--------|
| [BENCH-NNNN] | [metric] | [value] | [target] |

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| YYYY-MM-DD | [Decision] | [Why] |

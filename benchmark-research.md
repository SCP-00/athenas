# Athena Benchmark Research — Technical Comparison

> **For:** Chatty (Chief Architect)
> **From:** Buffy (Implementation Agent)
> **Date:** 2026-07-16
> **Objective:** Identify open, local, reproducible benchmarks for measuring real engineering capability — not token throughput. Classify each according to Athena's L0-L5 certification levels.

---

## Executive Summary

Chatty's directive reframes the entire certification architecture:

> *"Performance metrics (TTFT, tok/s, latency) are infrastructure metrics. Capability metrics are product metrics."*

The question is no longer "how fast?" but "what new capabilities become possible with Athena's environment?" A 4B model that solves a task previously requiring a 27B model — *because Athena supplied knowledge, workspace, tools, and agentic loops* — is infinitely more valuable than improving throughput by 8%.

This document surveys **13 benchmarks** and classifies each by the Athena certification levels they measure.

---

## Athena Certification Philosophy (Revised)

Chatty's L0-L5 levels, mapped to measurable capability:

| Level | Name | What It Measures | Athena's Role |
|-------|------|-----------------|---------------|
| **L0** | Raw model | Base coding/reasoning ability | None — model alone |
| **L1** | + Knowledge | Improvement from structured domain knowledge | `ath knowledge build` → generated pack |
| **L2** | + Workspace | Improvement from engineering environment | `ath workspace create` |
| **L3** | + Tools | Improvement from tool execution and feedback | `ath run` with tool access |
| **L4** | + Agent Loop | Improvement from iterative self-correction | Plan → Execute → Observe → Repair |
| **L5** | + Experience | Improvement from accumulated procedural memory | Cache of successful patterns |

Every existing benchmark maps to one or two of these levels. **No benchmark measures all six.** That is Athena's differentiation opportunity.

---

## Benchmark Comparison Matrix

| Benchmark | L0 | L1 | L2 | L3 | L4 | L5 | Offline | CI-Friendly | License | Est. Time | Tasks |
|-----------|:--:|:--:|:--:|:--:|:--:|:--:|:-------:|:-----------:|:--------|:----------:|:-----:|
| HumanEval | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Full | ✅ Perfect | MIT | ~2 min | 164 |
| MBPP | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Full | ✅ Perfect | CC-BY-4.0 | ~5 min | 974 |
| BigCodeBench | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Partial | ✅ Good | MIT | ~10 min | 1,140 |
| Aider Polyglot | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Full | ✅ Good | Apache-2.0 | ~15 min | 225 |
| RepoBench | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Full | ⚠️ Medium | MIT | ~30 min | Large |
| DevEval | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Full | ⚠️ Medium | Apache-2.0 | ~45 min | 1,874 |
| SWE-bench Lite | ✅ | ❌ | ❌ | ⚠️ Partial | ✅ | ❌ | ⚠️ Needs Docker | ⚠️ Heavy | MIT | ~2-4 hrs | 300 |
| SWE-bench Verified | ✅ | ❌ | ❌ | ⚠️ Partial | ✅ | ❌ | ⚠️ Needs Docker | ⚠️ Heavy | MIT | ~4-8 hrs | 500 |
| Terminal-Bench | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ Docker | ⚠️ Heavy | MIT | ~1-6 hrs | ~100 |
| GAIA | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ Needs Web | ❌ Needs Web | MIT | ~30 min/task | 466 |
| AgentBench | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ⚠️ Needs Docker | ⚠️ Heavy | MIT | ~2-4 hrs | 8 envs |
| OSWorld | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ⚠️ Needs VM | ❌ Very Heavy | MIT | ~1-8 hrs/task | ~100 |
| LiveBench | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ Needs API | ✅ Good | MIT | ~15 min | Dynamic |

---

## Detailed Analysis

---

### 1. HumanEval

**What it measures:** Functional correctness of Python code generated from natural language descriptions (docstrings) and function signatures. A model receives a function signature and docstring, and must generate the body.

**Created by:** OpenAI (2021)

**Dataset:** 164 hand-written Python problems

**Evaluation:** Execution-based — generated code is run against unit tests. Metric is `pass@k`.

**Runtime:** ~2 minutes for a full evaluation on modern hardware (164 problems × ~1s each)

**Hardware:** Minimal. CPU-only is sufficient. Any machine with Python 3.

**License:** MIT

**Reproducibility:** ✅ Perfect. Deterministic. Self-contained.

**CI Friendliness:** ✅ Excellent. Can run as a single `pytest` invocation.

**Offline Support:** ✅ Fully offline. No network required after download.

**Maintenance:** Stable — frozen. No updates expected or needed.

**Data Contamination Risk:** Very high. Dataset has been public since 2021 and is likely in every model's training data.

**Strengths:**
- Fastest possible baseline
- Deterministic evaluation
- Minimal dependencies
- Well-understood metric

**Weaknesses:**
- Extremely narrow scope (single-function Python only)
- Heavily contaminated
- No repository context
- No multi-step reasoning
- No tool use

**Athena Classification:** **L0 only.** Pure function generation. No environment needed.

**Recommendation for Athena:** Use as the fastest possible L0 smoke test. Run after every compiler change. < 2 minutes wall time. Catches regressions in raw model capability. Do NOT use as a primary capability metric.

---

### 2. MBPP (Mostly Basic Python Programming)

**What it measures:** Basic Python programming skills. Tasks are short function implementations from natural language descriptions.

**Created by:** Google Research (2021)

**Dataset:** ~974 crowd-sourced problems

**Evaluation:** Execution-based with unit tests. Same `pass@k` metric as HumanEval.

**Runtime:** ~5 minutes full evaluation

**Hardware:** Minimal. CPU-only.

**License:** CC-BY-4.0

**Reproducibility:** ✅ Excellent.

**CI Friendliness:** ✅ Excellent.

**Offline Support:** ✅ Fully offline.

**Data Contamination Risk:** Very high. Same vintage as HumanEval.

**Strengths:**
- Larger dataset than HumanEval (better statistical power)
- Slightly more diverse problems
- Same low overhead as HumanEval

**Weaknesses:**
- Same narrow scope as HumanEval
- Crowd-sourced quality is uneven
- Some problems are trivially simple
- No repository or multi-step reasoning

**Athena Classification:** **L0 only.**

**Recommendation for Athena:** Supplement HumanEval for a broader L0 baseline. The two together give ~1,100 L0 tasks in ~7 minutes.

---

### 3. BigCodeBench

**What it measures:** Function-level code generation with complex instructions requiring diverse library calls (139+ libraries). Has two splits: "Complete" (code completion) and "Instruct" (instruction following).

**Created by:** BigCode Project (2024)

**Dataset:** 1,140 tasks

**Evaluation:** Execution-based. Supports sandboxed execution via remote providers (e2b) or local Docker.

**Runtime:** ~10 minutes full evaluation

**Hardware:** Moderate. Python + Docker recommended, but runs without.

**License:** MIT/Apache

**Reproducibility:** ✅ Good. Repo contains exact evaluation code.

**CI Friendliness:** ✅ Good if using local execution.

**Offline Support:** ✅ Partial. Dataset is downloadable, sandbox execution can be local via Docker.

**Data Contamination Risk:** High. Public dataset, likely contaminated.

**Strengths:**
- More realistic than HumanEval (real library APIs)
- Good difficulty range
- Two splits allow targeted testing

**Weaknesses:**
- Still function-level only (no repository context)
- Library calls change over time (staleness risk)
- Docker dependency for sandbox execution

**Athena Classification:** **L0**, with potential L1 (the library knowledge IS domain knowledge that a Knowledge Pack could provide).

**Recommendation for Athena:** Better L0 baseline than HumanEval alone. The library-call nature aligns well with L1 knowledge pack testing — "does the model perform better on BigCodeBench tasks when given a Knowledge Pack for the required libraries?"

---

### 4. Aider's Polyglot Benchmark

**What it measures:** Code editing capability across 6 languages (C++, Go, Java, JavaScript, Python, Rust). The model receives a natural language request and must produce executable file edits.

**Created by:** Aider (Paul Gauthier) (2024)

**Dataset:** 225 most difficult Exercism exercises

**Evaluation:** Execution-based — tests whether the generated code passes the exercise's test suite.

**Runtime:** ~15 minutes full evaluation

**Hardware:** Minimal. Python only. No GPU needed.

**License:** Apache-2.0

**Reproducibility:** ✅ Excellent. The exact exercise set and evaluation code are in the Aider repo.

**CI Friendliness:** ✅ Good. Self-contained Python evaluation.

**Offline Support:** ✅ Fully offline.

**Data Contamination Risk:** High. Exercism exercises are public.

**Strengths:**
- Multi-language (not Python-only like most others)
- Tests code editing, not just generation
- Exercises are well-designed (from Exercism)
- Actively maintained

**Weaknesses:**
- Still isolated function/algorithm level
- No repository context
- No multi-file editing
- Exercism exercises may leak into training data

**Athena Classification:** **L0** (with L1 potential — language-specific Knowledge Packs could improve performance).

**Recommendation for Athena:** The best multi-language L0 baseline. Run after adding new language packs. If `ath pack show pack-go` shows tools, `aider polyglot go` should show improvement.

---

### 5. RepoBench

**What it measures:** Repository-level code auto-completion. Evaluates a model's ability to use cross-file context to complete code in real GitHub repositories.

**Created by:** Leolty (2023)

**Dataset:** Large-scale, derived from GitHub repos (Python and Java)

**Evaluation:** Masked line reconstruction. Metrics: Exact Match, Edit Similarity, CodeBLEU.

**Runtime:** ~30 minutes (varies with repository size)

**Hardware:** Moderate. Needs RAM for context processing.

**License:** MIT

**Reproducibility:** ⚠️ Medium. Results depend on exact context window and repository version.

**CI Friendliness:** ⚠️ Medium. Setup time is higher than HumanEval.

**Offline Support:** ✅ Fully offline.

**Maintenance:** Low. Last significant update 2023.

**Strengths:**
- Real repository context (not isolated functions)
- Cross-file reasoning
- Measures code understanding, not just generation

**Weaknesses:**
- Completion metrics (CodeBLEU) are less reliable than execution-based tests
- Repository staleness (repos may have changed)
- No functional correctness check (just edit similarity)
- Contamination risk (public GitHub repos)

**Athena Classification:** **L0** primarily, with **L1** potential (better context retrieval when Knowledge Packs provide repository understanding).

**Recommendation for Athena:** Useful for measuring workspace-level understanding, but completion metrics are weak evidence. Pair with functional benchmarks for stronger claims.

---

### 6. DevEval

**What it measures:** Repository-level code generation. Evaluates whether code aligns with real-world repository structures, dependencies, and complex software development requirements.

**Created by:** OpenCompass (2024)

**Dataset:** 1,874 samples from 117 real-world repositories

**Evaluation:** Execution-based where possible, supplemented by structural metrics.

**Runtime:** ~45 minutes full evaluation

**Hardware:** Moderate. Python + standard ML hardware.

**License:** Apache-2.0

**Reproducibility:** ✅ Good. Repo contains evaluation harness.

**CI Friendliness:** ⚠️ Medium. Longer runtime, more dependencies.

**Offline Support:** ✅ Full offline.

**Maintenance:** Moderate. Last updated 2024.

**Strengths:**
- Largest repository-level dataset
- Real-world repository structures
- Multiple domains (database, internet, etc.)

**Weaknesses:**
- Some evaluation metrics are structural, not functional
- Repository dependencies may break over time
- Public data → contamination risk

**Athena Classification:** **L0-L1.** Repository context aligns well with L2 workspace concept.

**Recommendation for Athena:** Good bridge between L0 (code generation) and L2 (workspace understanding). Measure whether workspace-generated context improves DevEval scores.

---

### 7. SWE-bench Lite (300 tasks)

**What it measures:** Whether an AI agent can repair real GitHub issues. Given a repository, an issue description, and tests, the agent must generate a patch that passes the tests.

**Created by:** Princeton NLP (2023)

**Dataset:** 300 GitHub issues from 12 Python repositories

**Evaluation:** Execution-based — the generated patch is applied, and the repository's test suite is run.

**Runtime:** ~2-4 hours full evaluation (each task requires setup, exploration, patching, testing)

**Hardware:** Heavy. x86_64 machine, 120GB+ free storage, 16GB+ RAM, 8+ CPU cores. Docker required.

**License:** MIT

**Reproducibility:** ✅ Good. Every task is containerized via Docker for deterministic execution.

**CI Friendliness:** ⚠️ Heavy. Docker-based, requires significant disk and compute. Not suitable for per-commit CI, but suitable for nightly.

**Offline Support:** ⚠️ Needs Docker images pulled once, then can run offline.

**Maintenance:** Active. Princeton NLP group maintains it.

**Data Contamination Risk:** Moderate (Lite subset is newer and less exposed than full SWE-bench).

**Strengths:**
- Measures real engineering ability (issue → diagnosis → patch)
- End-to-end agent evaluation
- Execution-based verification (tests pass or fail — no ambiguity)
- Industry standard for coding agent evaluation

**Weaknesses:**
- Heavy infrastructure requirements (Docker, storage)
- Long evaluation time
- Python-only repositories
- Setup requires significant disk (120GB+)
- Agent framework must be integrated (model alone cannot run SWE-bench)

**Athena Classification:** **L3-L4.** Requires tool execution (git, testing) and agentic loop (plan, edit, test, iterate). The closest existing benchmark to what Athena aims to measure.

---

### 8. SWE-bench Verified (500 tasks)

**What it measures:** Same as SWE-bench Lite, but with 500 human-validated tasks. Each task has been verified as solvable with the available information.

**Created by:** Princeton NLP + OpenAI (2024)

**Dataset:** 500 human-validated GitHub issues

**Evaluation:** Same execution-based Docker pipeline as Lite.

**Runtime:** ~4-8 hours full evaluation

**Hardware:** Same as SWE-bench Lite. Heavy.

**License:** MIT

**Reproducibility:** ✅ Good. High-quality human validation reduces ambiguity.

**CI Friendliness:** ⚠️ Heavy. Same infrastructure requirements.

**Offline Support:** ⚠️ Needs Docker (can run offline after image pulling).

**Maintenance:** Active. Considered the "gold standard" for coding agents.

**Data Contamination Risk:** Lower than Lite. Human-validated tasks are newer and less exposed.

**Strengths:**
- Gold standard for engineering agent evaluation
- Human-validated tasks (no ambiguous or unsolvable issues)
- Widely accepted leaderboard
- High task quality

**Weaknesses:**
- Same heavy infrastructure as Lite
- Python-only
- Long evaluation times
- Docker dependency

**Athena Classification:** **L3-L4.** The gold standard for measuring L3 (tools) and L4 (agent loop) capability. Highest-evidence benchmark for "can Athena make a smaller model solve harder problems?"

---

### 9. Terminal-Bench

**What it measures:** An agent's ability to perform complex, end-to-end technical tasks within command-line interfaces. Tasks include configuring legacy systems, reverse-engineering binaries, scientific computing, and engineering software.

**Created by:** Harbor Framework (2024)

**Dataset:** ~100 hard technical tasks

**Evaluation:** Outcome-based — verifies the final state of the container rather than the commands used.

**Runtime:** Highly variable. Simple tasks: minutes. Complex tasks: hours.

**Hardware:** Docker-based. Scales with parallel containers.

**License:** MIT

**Reproducibility:** ✅ Good. Containerized environments ensure reproducibility.

**CI Friendliness:** ⚠️ Heavy. Long-running tasks make per-commit CI impractical.

**Offline Support:** ✅ Docker-based, fully offline after image pull.

**Maintenance:** Active. Part of the Harbor evaluation framework.

**Strengths:**
- Measures real terminal fluency (the exact skill Athena aims to enable)
- Outcome-based verification (no command-spying)
- Tasks are genuinely hard
- Container isolation prevents system damage

**Weaknesses:**
- Small dataset (~100 tasks)
- Long execution times
- Docker dependency
- Less mature than SWE-bench

**Athena Classification:** **L2-L4.** Directly measures tool execution (L3) and agentic loops (L4) in the terminal environment. This is the benchmark that best matches Athena's intended use case.

**Recommendation:** This should be Athena's primary capability metric for L3 certification. It tests exactly what Athena provides: terminal tools, iterative debugging, and environment awareness.

---

### 10. GAIA (General AI Assistants)

**What it measures:** Multi-step reasoning with web browsing, tool use, and multi-modal inputs. Tasks are conceptually simple for humans but require multiple steps and tool orchestration.

**Created by:** Meta FAIR / Hugging Face (2023)

**Dataset:** 466 questions

**Evaluation:** Answer matching against ground truth.

**Runtime:** ~30 minutes per task (varies with search efficiency)

**Hardware:** Depends on tools used. Web search requires internet.

**License:** MIT

**Reproducibility:** ⚠️ Medium. Answers depend on web state which changes over time.

**CI Friendliness:** ❌ Poor. Requires internet, web browsing, and tool orchestration.

**Offline Support:** ❌ Requires internet. Many tasks need live web search.

**Maintenance:** Low. Originally released 2023, no significant updates.

**Strengths:**
- Tests real assistance capability (not just coding)
- Multi-step reasoning required
- Requires tool orchestration

**Weaknesses:**
- Web dependence makes deterministic evaluation impossible
- Changing web state breaks reproducibility
- Some tasks are trivial for humans
- Requires browser/tool infrastructure

**Athena Classification:** **L1-L3.** Tests knowledge use (L1), environment interaction (L2), and tool orchestration (L3). But only if Athena provides the search/tool infrastructure.

**Recommendation:** Not suitable for CI. Useful as a longer-duration capability benchmark for L3 tool evaluation. But the web dependency makes it a secondary option.

---

### 11. AgentBench

**What it measures:** General autonomous agent capability across 8 environments: OS interaction, database management, knowledge graph, card games, puzzles, web shopping, web browsing, household tasks.

**Created by:** THUDM (Tsinghua) (2023)

**Dataset:** 8 environments with multiple tasks each

**Evaluation:** Environment-specific metrics per scenario.

**Runtime:** ~2-4 hours full evaluation (across all 8 environments)

**Hardware:** Heavy. Docker required. WebShop environment needs ~15GB RAM.

**License:** MIT

**Reproducibility:** ✅ Good with Docker containers.

**CI Friendliness:** ⚠️ Heavy. Docker-based, resource-intensive.

**Offline Support:** ⚠️ Needs Docker, but can run offline after setup.

**Maintenance:** Low. Primary release 2023, minimal updates since.

**Strengths:**
- Broadest coverage of agent scenarios
- Standardized evaluation framework
- Tests non-coding agent capability
- Good for general agent evaluation

**Weaknesses:**
- Some environments are games/puzzles (less engineering relevance)
- Resource heavy
- Aging benchmark
- Contamination risk

**Athena Classification:** **L1-L4.** Different environments map to different levels. The OS environment maps to L3/L4; others are more general agent capability.

**Recommendation:** Useful as a broad agent capability check. Lower priority than Terminal-Bench and SWE-bench for engineering-specific certification.

---

### 12. OSWorld

**What it measures:** Multimodal agent ability to operate real desktop applications (Ubuntu). Tasks include file operations, application usage, multi-application workflows, and OS configuration.

**Created by:** xlang-ai (2024)

**Dataset:** ~100 desktop tasks

**Evaluation:** Execution-based — verifies task completion by checking desktop state.

**Runtime:** ~1-8 hours per task (very long)

**Hardware:** Very heavy. Requires virtualization (VMware, VirtualBox, or cloud VM). Docker-backed with KVM support recommended.

**License:** MIT

**Reproducibility:** ⚠️ Medium. VM state dependencies.

**CI Friendliness:** ❌ Very poor. Requires VM or cloud infrastructure.

**Offline Support:** ⚠️ Possible with local VM, but heavy.

**Maintenance:** Moderate. Active maintenance (2024).

**Strengths:**
- Most realistic desktop evaluation
- Measures true autonomous OS operation
- Multi-application workflows
- Vision + action required

**Weaknesses:**
- Extreme resource requirements
- Very long evaluation times
- VM management complexity
- Vision dependency (Athena currently text-only)
- Small task count

**Athena Classification:** **L2-L5.** The most comprehensive environment evaluation, but also the hardest to integrate.

**Recommendation:** Defer until Athena has a functioning multimodal agent pipeline. Not suitable for current architecture.

---

### 13. LiveBench (Bonus)

**What it measures:** Contamination-free LLM evaluation with dynamically refreshed tasks. Objective, verifiable questions across reasoning, coding, math, and language.

**Created by:** LiveBench team (2024)

**Dataset:** Dynamically generated, refreshed monthly

**Evaluation:** Execution-based and rule-based verification.

**Runtime:** ~15 minutes

**Hardware:** Minimal to moderate.

**License:** MIT

**Reproducibility:** ✅ Good for snapshot versions.

**CI Friendliness:** ✅ Good. Lightweight.

**Offline Support:** ❌ Requires API access for fresh tasks.

**Maintenance:** Active. Monthly refresh cycle.

**Strengths:**
- Contamination-free by design
- Regular refresh prevents score saturation
- Multi-domain coverage
- Lightweight execution

**Weaknesses:**
- Requires API access for new tasks
- Not engineering-specific (general capability)
- Snapshot versions become stale

**Athena Classification:** **L0.** General capability. Not engineering-specific.

**Recommendation:** Use as an independent L0 verification that avoids contamination concerns. Run monthly to supplement HumanEval/MBPP.

---

## Benchmark Recommendation for Athena

### Tier 1: Per-Commit CI (< 10 minutes)

| Benchmark | Level | Time | Purpose |
|-----------|:-----:|:----:|---------|
| **HumanEval** | L0 | ~2 min | Fast regression check |
| **MBPP** | L0 | ~5 min | Broader L0 baseline |
| **Aider Polyglot** | L0 | ~15 min | Multi-language check |

These run on every commit. If any regresses, the change is rejected. Total wall time: ~20 minutes.

### Tier 2: Nightly Certification (2-6 hours)

| Benchmark | Level | Time | Purpose |
|-----------|:-----:|:----:|---------|
| **SWE-bench Verified** | L3-L4 | ~4-8 hrs | Gold standard engineering capability |
| **Terminal-Bench** | L2-L4 | ~1-6 hrs | Terminal fluency + tool use |

These run nightly. They measure whether Athena actually improves a model's engineering capability. The output is the capability report Chatty described.

### Tier 3: Weekly Deep Certification (8+ hours)

| Benchmark | Level | Time | Purpose |
|-----------|:-----:|:----:|---------|
| **AgentBench** | L1-L4 | ~2-4 hrs | General agent capability |
| **DevEval** | L0-L2 | ~45 min | Repository-level generation |

These run weekly to measure broader capability trends.

### Tier 4: Deferred (Until Architecture Supports It)

| Benchmark | Reason for Deferral |
|-----------|--------------------|
| **OSWorld** | Requires VM infrastructure and multimodal pipeline |
| **GAIA** | Requires web browsing infrastructure |
| **LiveBench** | Requires API access for fresh tasks |

---

## What This Means for Athena's Architecture

### The Critical Gap

**No existing benchmark measures L1 (Knowledge Pack improvement) or L2 (Workspace improvement) independently.**

Every benchmark treats the model as a black box. None measure:
- "Does adding a Go Knowledge Pack improve the model's Go code?"
- "Does the workspace system prompt reduce errors?"
- "Does tool awareness improve iteration speed?"

This is Athena's measurement opportunity: **benchmark the layers, not just the model.**

### Proposed Architecture Change

```
ath certify \
    --model qwen-4b       \
    --benchmark swe-bench-verified \
    --level L0..L3
```

Should produce:

```
Capability: Engineering (SWE-bench Verified)
Model: Qwen 3.5-4B (Q4_K_M)

┌────────┬────────────────────────┬──────────┬──────────┬──────────┐
│ Level  │ Configuration          │ Pass@1   │ Δ from   │ Δ total  │
│        │                        │          │ previous │          │
├────────┼────────────────────────┼──────────┼──────────┼──────────┤
│ L0     │ Raw model              │ 12.3%    │ —        │ —        │
│ L1     │ + Knowledge Pack       │ 18.7%    │ +6.4%    │ +6.4%    │
│ L2     │ + Workspace            │ 22.1%    │ +3.4%    │ +9.8%    │
│ L3     │ + Tools                │ 31.5%    │ +9.4%    │ +19.2%   │
│ L4     │ + Agent Loop           │ 38.9%    │ +7.4%    │ +26.6%   │
└────────┴────────────────────────┴──────────┴──────────┴──────────┘

Hardware: RTX 3050 | llama.cpp | 16GB RAM
Total time: 4h 23m
```

This report answers exactly Chatty's question: *"What new capabilities become possible when Athena supplies everything the model was missing?"*

### Integration Requirements

To make this work, Athena needs:

1. **Benchmark harness abstraction** — A `BenchmarkRunner` trait similar to `KnowledgeProvider`:
   ```rust
   trait BenchmarkRunner {
       fn id(&self) -> &str;
       fn setup(&self) -> Result<BenchmarkEnv>;
       fn run(&self, model: &Model, env: &BenchmarkEnv, level: u8) -> Result<BenchmarkResult>;
       fn teardown(&self, env: &BenchmarkEnv) -> Result<()>;
   }
   ```

2. **HumanEvalRunner** — Simplest implementation. ~50 lines of Rust wrapping the Python harness.

3. **AiderPolyglotRunner** — Wraps Aider's evaluation scripts. Tests multi-language L0.

4. **SWEBenchRunner** — Manages Docker lifecycle for SWE-bench tasks. Most complex but highest value.

5. **TerminalBenchRunner** — Wraps Terminal-Bench's Harbor framework. Tests tool execution (L3).

### Self-Evaluation (This Agent)

As Chatty noted: *"Athena should be capable of certifying any compatible model — including the agent helping build Athena."*

I — this agent — am running on a proprietary model with internet access. I cannot benchmark myself with SWE-bench (no Docker, no local execution) or HumanEval (no Python execution environment). However, I can:

- **Write the benchmark integration code** (this document, the BenchmarkRunner trait, the harnesses)
- **Evaluate the architecture** (identify gaps, propose fixes)
- **Review the output** (this comparison document is the deliverable)

When Athena runs locally, it will certify the local model. This agent's role is to build the certification infrastructure, not to be certified by it.

---

## Summary: Priority Order for Benchmark Integration

| Priority | Benchmark | Level | Effort | Value | Rationale |
|:--------:|-----------|:-----:|:------:|:-----:|-----------|
| 1 | **HumanEval** | L0 | Low | High | Fastest L0 baseline. CI-ready. Documents regression immediately. |
| 2 | **Aider Polyglot** | L0 | Low | High | Multi-language L0. Measures effect of new language packs. |
| 3 | **Terminal-Bench** | L2-L4 | Medium | Very High | Best match for Athena's terminal-centric architecture. |
| 4 | **SWE-bench Verified** | L3-L4 | High | Very High | Gold standard. Industry credibility. |
| 5 | **MBPP** | L0 | Low | Medium | Supplements HumanEval. Broader dataset. |
| 6 | **AgentBench** | L1-L4 | High | Medium | Broad agent coverage, but aging. |
| 7 | **LiveBench** | L0 | Medium | Medium | Contamination-free validation. |

---

## Final Note

This research confirms Chatty's insight: **existing benchmarks measure only L0 (and partially L3-L4). None measure L1 (knowledge) or L2 (workspace) independently.**

That is both the problem and the opportunity.

The most valuable benchmark Athena could produce is not "can the model solve SWE-bench tasks?" — other frameworks already measure that. Athena's unique contribution would be:

**"How much does each architectural layer contribute to the model's engineering capability?"**

If Athena can answer that question with evidence, it differentiates from every existing inference framework.

The implementation path is:

1. Integrate HumanEval and Aider Polyglot (L0 baseline, ~1 day)
2. Integrate Terminal-Bench (L2-L4 core, ~3 days)
3. Instrument the existing Knowledge Pack + Workspace pipeline to measure L1-L2 deltas
4. Integrate SWE-bench Verified (L3-L4 gold standard, ~1 week)
5. Generate the capability-layer report from `ath certify`

The L1-L2 instrumentation is Athena-specific and cannot be replicated by any other framework. That is the competitive advantage.

— Buffy

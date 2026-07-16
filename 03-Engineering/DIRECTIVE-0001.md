---
id: DIRECTIVE-0001
title: How Athenas Must Be Developed
author: Chief Software Architect
date: 2026-07-16
status: Approved
version: 1.0
authority: Level 3 — Engineering
supersedes: []
---

# DIRECTIVE-0001 — How Athenas Must Be Developed

> **Authority:** Level 3 (Engineering)
> **Status:** Approved
> **Scope:** All agents and developers contributing to Athenas

---

## 1. GitHub Is the Source of Truth

From this moment forward, the repository is the authority. Not chats. Not memory. Not temporary files. Everything ends in Git.

---

## 2. Branching Model

Never work directly on `main`.

```
main
  └── develop
        ├── feature/constitution
        ├── feature/runtime-manager
        ├── feature/knowledge-engine
        ├── feature/model-registry
        ├── feature/dashboard
        ├── feature/pi-provider
        └── feature/benchmark-engine
```

No massive commits. Each commit is a single logical change.

---

## 3. Pull Requests Are Mandatory

Even when working alone:

```
feature → PR → Review → Merge
```

Why? Because in one year, agents will be able to review those PRs.

---

## 4. Issues First

Never implement something without reason. Every change starts with an Issue.

```
Issue: ATH-132 — Implement Runtime Registry
  ↓
SPEC → ARCH → Implementation
```

---

## 5. Every Change Answers a Question

Never write code. Always answer a question.

**NO:**
```
Implement benchmark.
```

**YES:**
```
How should Athenas determine whether a model is objectively better?
```

---

## 6. Engineering First

We do not want:

```
Idea → Code
```

We want:

```
Question → Research → Evidence → Decision → Specification → Architecture → Implementation → Validation
```

---

## 7. Everything Must Be Replaceable

If llama.cpp disappears tomorrow, nothing should break. Athenas depends on an interface, not an implementation.

```
Runtime
  └── Interface
        ├── LlamaCppProvider
        ├── VLLMProvider
        ├── MLXProvider
        ├── OllamaProvider
        ├── OpenAIProvider
        └── AnthropicProvider
```

Never do `if llama.cpp`.

---

## 8. Never Hardcode

**BAD:**
```python
ctx = 262144
port = 8088
```

**GOOD:**
```python
profile.max_context
runtime.port
```

Everything configurable.

---

## 9. Every Benchmark Must Be Reproducible

Running `BENCH-0007` tomorrow must yield the same result. Record:

- commit hash
- runtime version
- CUDA version
- driver version
- kernel version
- llama.cpp commit hash
- model hash
- GGUF hash
- profile
- samplers
- GPU temperature
- RAM
- swap
- everything

We demand scientific reproducibility.

---

## 10. Never Delete Information

If something becomes obsolete: archive it. Never delete.

```
99-Archive/
```

---

## 11. Every Document Has an Owner

```
ARCH-0008
Owner:      Runtime Team
Status:     Approved
Version:    1.2
Supersedes: ARCH-0004
```

---

## 12. Agents Must Never Assume

If something doesn't exist, don't invent it. Create:

```
TODO
QUESTION
UNKNOWN
BLOCKED
```

No hallucinating architecture.

---

## 13. Agents Must Explain Why

Never accept:

```
This is better.
```

Always:

```
Evidence:
Reason:
Tradeoffs:
Alternatives:
Decision:
```

---

## 14. All Decisions Are Reversible

Every ADR must answer:

- Why?
- When would we revert this?
- What evidence would make us change?

---

## 15. Never Optimize Before Measuring

Correct order:

```
Measure → Profile → Identify Bottleneck → Optimize → Measure Again
```

Never the reverse.

---

## 16. Every Component Must Be Able to Die

The favorite question:

> What happens if this module disappears?

If it breaks everything, the architecture is wrong.

---

## 17. Don't Trust Public Benchmarks

The official benchmark is:

> **YOUR LAPTOP**

Because that's the target hardware.

---

## 18. Everything Has an Interface

Even documentation:

```
Model Registry
  Inputs:
  Outputs:
  Contracts:
  Errors:
  Examples:
```

---

## 19. Everything Produces Artifacts

Not logs. Artifacts.

```
benchmark.json
dashboard.html
summary.md
timeline.md
metrics.csv
```

---

## 20. GitHub Actions From Day One

Every PR executes:

```
Lint → Validate YAML → Validate IDs → Check Links
→ Markdown Lint → Ontology Check → Reference Integrity
→ Graph Build → Documentation Build
```

No code yet? No problem. We automate documentation.

---

## 21. Build the Knowledge Graph

The final goal is not markdown. It's:

```
REQ → SPEC → ARCH → CODE → TEST → BENCH → MODEL → PROFILE
```

Everything connected.

---

## 22. Commits

**NEVER:**
```
Update docs
```

**ALWAYS:**
```
docs(CONST): Create first project constitution
arch(RUNTIME): Define runtime abstraction layer
feat(BENCH): Implement TTFT collector
refactor(REGISTRY): Separate quant metadata
```

---

## 23. Releases

Not `v0.2`. Phases that tell a story:

```
Genesis → Foundation → Alpha → Beta → Release Candidate → 1.0 Athena
```

---

## 24. Wiki

No GitHub Wiki. All documentation lives in the repository. Versioned. Reviewed. Audited.

---

## 25. Discussions

Yes. Use them heavily. Every new idea starts there, not in code.

---

## 26. Project Board

Simple columns matching the engineering method:

```
Inbox → Research → Question → Specification → Architecture → Implementation → Validation → Done
```

---

## 27. Milestones

```
M0 — Genesis:        Constitution, Bootstrap, Knowledge System
M1 — Runtime Core:   Registry, Profiles, Providers
M2 — Benchmark Engine
M3 — Certification
M4 — Dashboard
M5 — Agent Platform
M6 — Plugin Ecosystem
M7 — Public Release
```

---

## 28. The Most Important Rule

No agent may violate this rule:

> Athenas does not optimize for code.
> Athenas optimizes for engineering knowledge.
> Code is only one possible artifact of engineering.

This sentence appears literally in `CONST-0001`.

---

## Decision Log

| Date | Decision | Rationale | Alternatives | Evidence |
|------|----------|-----------|--------------|----------|
| 2026-07-16 | Adopt DIRECTIVE-0001 | Establish engineering foundations from day one | Informal development | Architect review |

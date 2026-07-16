---
id: BOOTSTRAP-0001
title: Athenas Agent Bootstrap
date: 2026-07-16
status: Active
purpose: "First file an agent reads when joining this project"
---

# BOOTSTRAP.md — Agent Bootstrap

> Welcome to **Athenas**, an engineering platform for local artificial intelligence.
> This document explains how to start contributing in under 5 minutes.

---

## 1. What Is Athenas?

Athenas is an engineering platform for local AI. It manages the complete lifecycle of local LLMs: downloading, profiling, benchmarking, certifying, and running models across multiple runtimes (llama.cpp, vLLM, MLX, Ollama, etc.).

Unlike a simple model runner, Athenas is built as an **engineering system** where:
- **Knowledge** is the primary artifact (not code)
- **Evidence** drives all decisions (not intuition)
- **Traceability** connects every requirement to its implementation and validation
- **Automation** allows agents and humans to collaborate on engineering

## 2. Which Documents Have the Highest Authority?

| Priority | Document | Location |
|----------|----------|----------|
| 1 (highest) | **CONST-0001** — The Athenas Constitution | `00-Constitution/CONST-0001.md` |
| 2 | **DIRECTIVE-0001** — Engineering Directives | `03-Engineering/DIRECTIVE-0001.md` |
| 3 | **knowledge.md** — Project Index (this directory) | `./knowledge.md` |
| 4 | **CURRENT_CONTEXT.md** — Daily Status | `./CURRENT_CONTEXT.md` |
| 5 | **ontology.yaml** — Official Vocabulary | `knowledge/ontology.yaml` |
| 6 | **.state/project.yaml** — Structured State | `.state/project.yaml` |

**Rule:** When documents conflict, lower authority level number wins (0 = Constitution, highest).

## 3. What Should I Read Next?

After this file:

1. **`CURRENT_CONTEXT.md`** — What's happening right now
2. **`knowledge.md`** — The project index (find any document)
3. **`.state/project.yaml`** — Structured project state (machine-readable)

## 4. What Is the Active Task?

Check `.state/project.yaml` → `next_task` field.
Check `CURRENT_CONTEXT.md` → Current Objective section.

## 5. What Rules Must I Never Violate?

From CONST-0001, Article I:

1. **Knowledge precedes implementation** — never write code without a specification
2. **Evidence outweighs intuition** — back every claim with data
3. **Traceability is absolute** — always reference by ID, never by name
4. **No optimization without measurement** — baseline first, then optimize
5. **Capabilities over implementations** — depend on interfaces, not tools
6. **Architecture must survive technology changes** — design for longevity
7. **Every experiment becomes knowledge** — never discard data
8. **Write for humans AND agents** — machine-readable metadata in every document

## 6. How Do I Propose Changes?

Athenas follows a strict engineering process:

```
Idea → Discussion → Engineering Review → Specification → ADR → Approved → Implementation
```

| If you want to… | Create a… | Template at |
|-----------------|-----------|-------------|
| Propose a new feature | `REQ-NNNN.md` | `04-Specifications/templates/REQ-template.md` |
| Specify a solution | `SPEC-NNNN.md` | `04-Specifications/templates/SPEC-template.md` |
| Design architecture | `ARCH-NNNN.md` | `05-Architecture/templates/ARCH-template.md` |
| Record a decision | `ADR-NNNN.md` | `12-ADR/ADR-template.md` |
| Run a benchmark | `BENCH-NNNN.md` | `09-Validation/templates/BENCH-template.md` |
| Define a profile | `PROFILE-NNNN.md` | `03-Engineering/templates/PROFILE-template.md` |
| Register a model | `MODEL-NNNN.md` | `03-Engineering/templates/MODEL-template.md` |

## 7. How Do I Validate an Implementation?

Every implementation must be validated against its specification:

1. Reference the SPEC and REQ it implements
2. Provide evidence (BENCH, EVID, or test results)
3. Record the validation in the document's YAML front-matter
4. Update the Knowledge Graph relationships

## 8. How Do I Record Decisions?

Use the Decision Log format:

```markdown
| Date | Decision | Rationale | Alternatives | Evidence |
|------|----------|-----------|--------------|----------|
| YYYY-MM-DD | What was decided | Why | What else was considered | Links to evidence |
```

Add this to the relevant document or to `12-ADR/`.

## 9. Quick Reference

| Concept | Meaning |
|---------|---------|
| CONST | Constitution — inviolable foundation |
| DIRECTIVE | Engineering directives — how to work |
| REQ | Requirement — what is needed |
| SPEC | Specification — how it must work |
| ARCH | Architecture — structural design |
| ADR | Architecture Decision Record — why |
| BENCH | Benchmark / Evidence Package — proof |
| MODEL | Model definition — weights + metadata |
| PROFILE | Runtime Profile — execution policy |
| EVID | Evidence item — raw data point |
| EXP | Experiment — method + results |
| CAP | Capability — measurable functionality |

## 10. Engineering Principles (Quick Reference)

```
╔═══════════════════════════════════════════════════════════╗
║  Question → Hypothesis → Evidence → Experiment           ║
║       → Observation → Decision → Specification           ║
║       → Architecture → Implementation → Validation       ║
║       → Knowledge → New Question                         ║
╚═══════════════════════════════════════════════════════════╝
```

---

*"Athenas does not optimize for code. Athenas optimizes for engineering knowledge."*

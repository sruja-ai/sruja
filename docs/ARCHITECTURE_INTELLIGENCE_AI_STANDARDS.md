# Architecture Intelligence: AI Standards & Skills Strategy

**Goal:** Make Sruja’s Architecture Intelligence tool **solid** and **AI-native** using current AI standards: skills as the integration surface, evidence-first answers, and deterministic intelligence with optional LLM enhancement.

---

## 1. Principles (Latest AI Standards)

| Principle | Meaning for Sruja |
|-----------|-------------------|
| **Skills as integration** | AI agents (Cursor, Copilot, etc.) integrate via **skills**, not a custom MCP server. The agent learns *when* and *how* to run `sruja` from skill instructions. |
| **Evidence-first** | Every answer is backed by **evidence** (file paths, graph nodes, violation lists). No “trust me” answers. |
| **Deterministic-first** | Value is delivered **without** an API key: scan → drift → analyze → why all work from code + optional baseline. LLM is an optional enrichment layer. |
| **Structured outputs** | CLI emits JSON for machine consumption (`--format json`). Skills tell the agent how to parse and use it. |
| **Single source of truth** | Architecture “reality” is the **scan graph**; “intent” is the baseline (e.g. `architecture.sruja`). Drift and health are derived from comparing the two. |

---

## 2. Architecture Intelligence Flow (Canonical)

```
intent (optional)  →  scan  →  drift  →  analyze  →  why / context
     .sruja              │        │         │           │
     ADRs                 │        │         │           │
                          ▼        ▼         ▼           ▼
                    sruja.graph   violations  report   evidence
                    (reality)     + health    (CTO)    + answers
```

**Commands that agents must know:**

| Command | When to use | Output to use |
|---------|-------------|---------------|
| `sruja quickstart -r .` | First run; “what’s the state of this repo?” | Inventory, health score, top findings, next steps |
| `sruja scan . --output sruja.graph.json` | Need raw graph for later steps or tooling | `sruja.graph.json` |
| `sruja drift -r .` or `sruja drift -r . -a architecture.sruja` | “Is code aligned with intent?” or “what’s broken?” | Violations, health score, (with baseline: layer/cycle/god/orphan) |
| `sruja drift-pr -r . --base main` | CI: “Did this PR make structure worse?” | New violations, health delta |
| `sruja analyze -r . --view cto` | Deeper report: structure + optional traces/intent | Comprehensive report (structural, semantic, intent, runtime) |
| `sruja why "question" -r .` | “Why do we use X?” / “What depends on Y?” | Answer + evidence (graph + file refs), confidence |
| `sruja context export -r . -f cursor-rules` | Feed AI editor with architecture context | Cursor rules / Copilot instructions / markdown |

---

## 3. Health Score & Insights (For Agents)

- **Health score** is **structural only** (cycles, layer violations, god modules, orphans). Use it for **trend and CI**, not as a single “quality” number. See [HEALTH_SCORE.md](HEALTH_SCORE.md).
- **Insights** are useful when they point at **your** code (not vendor/stories/tests). See [INSIGHTS_USEFULNESS.md](INSIGHTS_USEFULNESS.md).
- Agents should:
  - Prefer **violation lists + file paths** over the raw number.
  - For “why” answers, prefer **evidence** (from `sruja why`) over unsupported claims.
  - When suggesting fixes, use **tip IDs / docs** or optional LLM-backed suggestions (future).

---

## 4. Skills That Make It Solid

### 4.1 Existing skills (use them)

- **sruja-architecture** – When generating/editing `.sruja` files (rulebook, baseline). DSL rules, patterns, anti-patterns.
- **sruja-architecture-agent** – When discovering architecture from code (clone, read manifests, generate DSL). Use with `sruja lint` and `sruja export`.
- **sruja-architecture-collaboration** – Multi-agent sessions, review workflows, ADRs, CI integration.

### 4.2 New: sruja-architecture-intelligence

**Purpose:** Teach the agent the **Architecture Intelligence** workflow: when to run quickstart, drift, analyze, why, context export, and how to interpret results.

**Triggers:**

- User asks about “architecture health,” “drift,” “dependencies,” “why do we have X,” “what’s the state of this codebase.”
- User wants “context for the AI” or “rules for Cursor/Copilot.”
- User wants CI checks for structural regression.

**What the skill contains:**

- Table of commands and when to use each (Section 2 above).
- How to read health score and violation lists.
- How to chain: e.g. `quickstart` → then `drift -a architecture.sruja` if baseline exists.
- How to use `sruja why` for evidence-based answers and when to cite file refs.
- How to export context (`context export -f cursor-rules`) and where to put it (e.g. `.cursor/rules` or project rules).
- That no API key is required for core value; LLM is optional enhancement.

This makes the tool “solid” for AI: one place that defines **how** to use Architecture Intelligence correctly.

---

## 5. Implementation Checklist (Concise)

- [x] Deterministic quickstart, drift, analyze, why (existing).
- [x] Context export (cursor-rules, copilot-instructions, markdown, json) (existing).
- [x] Health score and violation breakdown (structural); docs for interpretation (HEALTH_SCORE, INSIGHTS_USEFULNESS).
- [ ] **Add skill:** `skills/sruja-architecture-intelligence/` with SKILL.md (and optional AGENTS.md / rules).
- [ ] **Document:** In README/skills README: “For AI agents: add sruja-architecture-intelligence to get the full Architecture Intelligence workflow.”
- [ ] Optional: LLM-backed “suggest fix” per violation behind a flag (see INSIGHTS_USEFULNESS Option A); keep deterministic path default.

---

## 6. Summary

- **Solid** = evidence-first, deterministic-first, clear commands, and a **dedicated skill** so agents know exactly when and how to use quickstart, drift, analyze, why, and context export.
- **Latest AI standards** = skills as the integration surface, structured JSON outputs, and optional LLM only where it adds value (e.g. tailored suggestions), without making it a prerequisite.
- **Skills** = sruja-architecture (DSL), sruja-architecture-agent (discovery), sruja-architecture-collaboration (multi-agent). *Planned:* sruja-architecture-intelligence (run and interpret the CLI workflow).

With this, the Architecture Intelligence tool is well-defined for both humans and AI agents, and remains maintainable and adoptable without mandating API keys or extra infra.

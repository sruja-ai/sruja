# Making Sruja Buyable for Anyone: Features for AI-Era Fast Development

**OSS-first (current target):** We are targeting **pure open source users**—maintainers, contributors, small teams, self-hosted. Success = adoption, contributor experience, and “runs in my repo” with minimal friction. Value = fast feedback in PR + IDE, evidence for maintainers, and machine-readable context for AI—**without** requiring a paid product. “Buyable” and paid/hosted tiers remain future positioning.

**Goal:** Define the real features that make Sruja valuable to a broad set of teams in a world where development is faster, more AI-assisted, and more chaotic.

**Principle:** In AI-era fast dev, structure degrades faster, context is scarce, and “did we just break the architecture?” must be answered in seconds. Sruja should be the default tool that keeps structure and intent alive at that speed.

---

## Current state (OSS roadmap)

| Priority | Feature | Status | Notes |
|----------|---------|--------|-------|
| P0 | PR / change-scoped drift | In progress | Base/head or changed-files; new violations only |
| P0 | IDE real-time feedback | Not started | LSP or extension; depends on scan/graph APIs |
| P1 | Machine-readable context for AI | In progress | `sruja context export` / for-ai JSON |
| P1 | Incremental / fast analysis | Not started | Cache by ref; re-parse changed files + dependents |
| P1 | “Why” and evidence as API | Partial | CLI JSON first; MCP/LSP later |
| P2 | One-click baseline from quickstart | Not started | |
| P2 | CI integration (GitHub Action) | Partial | Docs + example YAML |
| P2 | Bounded context / tags + rules | Not started | |
| P3 | Trends / dashboard | Future / optional | Out of scope for OSS-first phase |
| P3 | More languages | Future | |

**Incremental analysis:** Either treat as a **P0 dependency** with an MVP (drift on changed files + cache by git ref), or ship P0 with full-scan-on-PR first and add incremental when full scan is too slow for target repos.

**Multi-repo:** **Out of scope for OSS v1.** Cross-repo graph and “which service can call which?” can be a future feature; single-repo PR gates and IDE feedback are the focus first.

**Evidence API (minimal contract):** Structured “why” / evidence for tools and AI:

- `Evidence { kind, id, file?, excerpt?, confidence?, rule_id? }`
- Query types: “why this component?”, “what depends on X?”, “why is A→B allowed?” (e.g. ADR or rule).

**Quality attributes (targets for OSS users):** PR-scoped drift should complete in a predictable time (e.g. &lt; 30s for medium repos with cache; full-scan fallback acceptable for small repos). IDE check budget: aim for &lt; 2s for current file + immediate deps when incremental is available. Results should be deterministic for the same base/head and config.

---

## Why “AI era” changes what’s worth paying for

| Reality | Implication for Sruja |
|--------|-------------------------|
| **More code churn** | Structure (cycles, layering) degrades faster. One-time “run drift” isn’t enough; **every PR** should be checked. |
| **AI-generated code** | New code may ignore architecture. Teams need **“did this change violate our rules?”** at PR time, with clear blame (file/line). |
| **Context hunger** | AI agents and new devs need **“what can call what?”** and **“why does this exist?”** in machine-readable form so they can generate correct code. |
| **Docs go stale** | With high velocity, docs and code drift quickly. **Single source of truth** (e.g. `.sruja`) that drives both checks and docs is worth paying for. |
| **Multi-repo / services** | Many teams have 10+ repos. **Cross-repo boundaries** and “which service can call which?” matter. |
| **Compliance still exists** | Regulated and enterprise teams still need **evidence** that code matches design. AI era doesn’t remove that—it makes automated proof more valuable. |

So: **buyable** means Sruja is in the **fast feedback loop** (PR, IDE, AI context) and delivers **evidence and gates**, not just reports.

---

## Must-have features (make it buyable for “anyone”)

### 1. **PR / change-scoped drift (not just “full repo”)**

**Problem:** Today you run drift on the whole repo. In fast dev, the question is: **“Did *this* PR introduce a cycle or layer violation?”**

**Feature:**
- **Input:** Base ref (e.g. `main`) + head ref (e.g. PR branch). Or: list of changed files.
- **Behavior:** Compute graph at base and at head (or incrementally for changed files). Diff. Report only **new** violations introduced in this PR.
- **Output:** “This PR adds 1 circular dependency (A → B → C → A)” with file/line evidence. CI fails only when **this PR** regresses structure.
- **Where it lives:** CLI `sruja drift --base main --head HEAD` or `sruja drift --changed-files list.txt`; GitHub Action / GitLab job that comments on PR with “New violations” and blocks merge.

**Why buyable:** Teams run 50+ PRs/day. Full-repo drift is noisy. PR-scoped drift is the **gate** they’ll pay for.

---

### 2. **IDE / editor: real-time structure feedback**

**Problem:** Devs and AI write code in the editor. They find out about cycles and layer violations only when they run CLI or CI—too late.

**Feature:**
- On **save** (or on demand): run lightweight check for the **current file** and its immediate dependencies. “This import creates a cycle” or “This dependency violates layer X → Y.”
- **Diagnostics** in the editor: red squiggles or gutter markers with message and fix hint.
- Optional: **“Why can’t I import this?”** in hover or quick fix: “Importing X would create a cycle with Y, Z.”

**Why buyable:** Feedback in the **inner loop** (where code is written) prevents bad structure before it’s committed. This is table stakes for “AI-era” tooling that claims to protect architecture.

---

### 3. **Machine-readable context for AI agents (and humans)**

**Problem:** AI coding assistants (Cursor, Copilot, Claude) don’t know your architecture. They suggest imports and calls that can create cycles or break layers.

**Feature:**
- **Export “architecture context”** for a repo: allowed dependencies per layer/bounded context, list of “do not depend on” edges, and optional “why” snippets (from ADRs or .sruja descriptions).
- **Formats:** JSON or structured markdown that can be injected into AI context (e.g. Cursor rules, repo-level instructions, or MCP). Example: “Services in `billing/` must not import from `marketing/`. All DB access goes through `data/`.”
- **CLI:** e.g. `sruja context export -r . -f json` or `sruja context for-ai -r .`.

**Why buyable:** Teams already pay for AI tools. They will pay for **tooling that makes AI output respect their architecture**—fewer bad suggestions, fewer cycles introduced by AI.

---

### 4. **Faster and incremental analysis**

**Problem:** Large repos. Running full scan + drift on every PR or every save is too slow. Devs and CI skip it.

**Feature:**
- **Incremental scan:** Only re-parse changed files (and dependents). Merge with cached graph. Target: **&lt; 30s** for “drift on this PR” on repos with 500+ files.
- **Cache:** Persist graph (e.g. by git ref or content hash). CI: restore cache for base, compute only for head; or reuse last full scan and apply diff.
- **Config:** Max files per run, time budget, “quick mode” (e.g. only cycles + layer, no god modules).

**Why buyable:** If the check is slow, it won’t run. Speed is a **feature** that makes “run on every PR” and “run in IDE” viable.

---

### 5. **“Why” and evidence as an API (for AI and tools)**

**Problem:** “Why” today is CLI-only and human-oriented. AI and other tools can’t consume it.

**Feature:**
- **Structured API:** e.g. `sruja why "Why do we use Postgres?" -r . --format json` returns: `{ "answer": "...", "evidence": [ { "kind": "node", "id": "...", "file": "...", "excerpt": "..." } ], "confidence": 0.8 }`.
- **Query by component or dependency:** “What depends on X?” “Why is there an edge from A to B?” (e.g. ADR or rule that allows it.)
- **MCP / LSP:** Expose “get context for symbol/file” so IDEs and AI can request evidence without parsing CLI output.

**Why buyable:** AI and automation need **structured** context. “Why” as an API makes Sruja part of the **context pipeline** teams will pay for.

---

### 6. **Team/org visibility: trends and history (paid tier)**

**Problem:** “Are we getting better or worse?” In fast dev, one number (health score) today isn’t enough. Leads and managers want **trends**.

**Feature:**
- **Store** drift results per run (e.g. by ref, timestamp). Optional backend (cloud or self-hosted).
- **Dashboard or report:** Health score over time; “new cycles this week”; “which team/area introduced the most violations.”
- **Alerts:** “Health dropped below 70” or “New circular dependency in service X.”

**Why buyable:** This is **management visibility**. Teams that pay for “quality” tooling (SonarQube, etc.) pay for trends and dashboards. Sruja can own “structure and drift” trend.

---

### 7. **More languages (expand “anyone”)**

**Problem:** Today: JS/TS, Python, Go, Rust. Many codebases are Java, C#, C++, Kotlin, etc. AI generates all of them.

**Feature:**
- Add **Java**, **C#**, **Kotlin**, **C/C++** (at least manifest or key paths). Prioritize by user demand and parser availability (e.g. tree-sitter).
- **Fallback:** For unsupported languages, at least **manifest-based** graph (e.g. Maven, Gradle, csproj) so “we have N modules and M deps” and basic drift (e.g. new dep) is visible.

**Why buyable:** “Anyone” includes Java/C# shops and polyglot repos. Broader language support = larger addressable market.

---

### 8. **One-click “architecture from repo” (reduce bootstrapping)**

**Problem:** “I want to try Sruja but I don’t have a .sruja file.” Creating one by hand or via AI skill is friction.

**Feature:**
- **Quickstart → baseline:** After `sruja quickstart -r .`, offer: “Generate a draft `architecture.sruja` from this scan?” Heuristics: top-level dirs → systems; package names → containers; DB-like nodes → databases. Output is editable, then `sruja drift -a architecture.sruja`.
- **Incremental improvement:** “Update baseline from current scan” (merge new nodes/edges into .sruja with review).

**Why buyable:** Converts “I ran quickstart once” into “I have a baseline and I’m comparing.” That’s the path to **ongoing** value and willingness to pay.

---

### 9. **CI integration that’s obvious and robust**

**Problem:** “How do I run Sruja in CI?” is still a question. Exit codes and reporting should be bulletproof.

**Feature:**
- **First-class CI story:** Documented GitHub Action / GitLab job that: (1) runs PR-scoped drift (base vs head), (2) posts comment with “New violations” and links to files, (3) fails the job only on **new** errors (configurable). One YAML copy-paste.
- **Status checks:** Optional JSON/artifact for “Sruja: pass/fail” and “link to report.”
- **Config:** `sruja.toml` in repo: baseline path, which violations are errors vs warnings, exclude paths.

**Why buyable:** Teams that “run it in CI” are one step away from “pay for team features.” Making CI trivial increases adoption and conversion.

---

### 10. **Lightweight “bounded context” or tags (semantic hint)**

**Problem:** Pure structure (imports) doesn’t tell AI or humans “this is the billing context” or “this must not talk to that.”

**Feature:**
- **Tags or bounded contexts** in .sruja or in config: e.g. `billing`, `marketing`, `shared-kernel`. Map nodes/dirs to these (by path or name).
- **Rules:** “No edge from `billing` to `marketing`” or “Only `api` can depend on `db`.” Drift checks these rules.
- **Export for AI:** “Components in context `billing`: …; allowed external deps: ….” So AI knows boundaries without full domain modeling.

**Why buyable:** Gives “semantic-ish” value without building a full domain model. Enough for **“AI, stay within these boundaries.”**

---

## Prioritization (what to build first)

| Priority | Feature | Rationale | Effort (rough) |
|----------|---------|-----------|----------------|
| **P0** | PR / change-scoped drift | Core “gate” for fast dev; unblocks CI story | Medium |
| **P0** | IDE real-time feedback | Where devs and AI work; prevents bad code before commit | Medium |
| **P1** | Machine-readable context for AI | Differentiator; “make AI respect our architecture” | Small–Medium |
| **P1** | Incremental / fast analysis | Makes P0 and IDE feasible at scale | Medium |
| **P1** | “Why” and evidence as API | Enables AI and tools to consume Sruja | Small |
| **P2** | One-click baseline from quickstart | Converts try-once users into ongoing users | Small |
| **P2** | CI integration (docs + action) | Adoption and “run in CI” → path to paid | Small |
| **P2** | Bounded context / tags + rules | Semantic hint without full domain model | Medium |
| **P3** | Trends / dashboard (paid tier) | Management visibility; monetization | Large |
| **P3** | More languages | Expands market | Large (per language) |

---

## What “buyable for anyone” looks like after this

- **Developer:** Saves a file → sees immediately if they introduced a cycle or layer violation. Asks “why can’t I import this?” in the IDE. Gets AI suggestions that respect repo rules (because context is exported).
- **Tech lead:** Every PR shows “New structure violations: 0” or “1 new cycle (link).” Dashboard shows health trend. One-click baseline from repo so new repos get value fast.
- **Enterprise:** CI blocks PRs that add violations. Evidence (what’s allowed, what’s not) is exportable for audits. Optional paid tier: history, trends, support.

Sruja becomes **the** tool that keeps structure and intent alive in the AI-era fast loop—and that’s what makes it worth paying for, for almost any team that cares about structure at all.

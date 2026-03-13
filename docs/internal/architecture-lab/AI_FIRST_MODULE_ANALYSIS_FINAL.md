# AI-First Architecture Intelligence: Final Practical Direction

**Status:** Final  
**Date:** 2026-02-22  
**Goal:** Maximize adoption and perceived value immediately, while aggressively reducing long-term code maintenance risk.

**Update (post MCP/LLM removal):** The sruja-mcp crate and Sruja-owned LLM integrations have been removed. Editor integration is **skills + CLI** only (e.g. Sruja skill in Cursor/Copilot); no MCP server.

---

## 1. Product Positioning (What Users Understand in 10 Seconds)

Sruja should be positioned as:

> **Architecture memory + drift intelligence for real codebases.**

The first value must **not** require:
- model keys
- desktop app setup
- Slack/OAuth setup
- MCP client setup
- writing `.sruja` first

---

## 2. Adoption-First Principle

If users do not get value in the first minute, they will not adopt.

So the product entry point is:
1. `sruja quickstart -r .` (implemented)
2. `sruja why ...` and `sruja drift ...` as immediate follow-up
3. CI integration next
4. Chat/LLM/MCP channels after trust is established

---

## 3. Zero-Model-Key First Value (Deterministic Intelligence)

Without LLM, Sruja still provides architecture intelligence via code evidence:

### Inputs
- source code + manifests (`sruja-scan`)
- optional baseline graph (`sruja-diff`)

### Deterministic outputs
- architecture inventory (services/modules/databases/apis)
- dependency graph summary
- drift/risk report:
  - circular dependencies
  - orphan modules
  - layer violations
  - high-coupling/god modules
- evidence pointers (file/module/edge references)

### Why queries without LLM
`sruja why` should answer with evidence templates:
- "X depends on Y because imports/calls were detected in A, B, C"
- "Cycle A -> B -> C -> A detected from dependency edges ..."

This gives immediate trust because the answer is reproducible and inspectable.

---

## 4. LLM as Optional Enhancement (Not a Prerequisite)

LLM is additive for:
- conversational extraction from discussions
- decision draft generation
- richer natural-language synthesis

If key exists -> enrich results.  
If key missing -> deterministic mode remains fully useful.

---

## 5. Codebase Reality Check (Current State)

Validated from current code:
- `sruja-chat` is orchestration backend (sessions, extraction, graph merge, persistence), not just UI logic.
- `sruja-cli` already delivers key adoption paths (`why`, `drift`, `scan`, `lint`).
- `sruja-mcp` is currently HTTP API surface, not full MCP transport implementation.
- `sruja-diff` exists, but drift behavior is still partly duplicated in CLI logic.
- `sruja-diagnostics` is shared broadly; merging into language now increases coupling.
- Core checks/tests pass on workspace and architecture-intelligence E2E paths.

---

## 6. Target Architecture (Practical and Maintainable)

```text
                    +-----------------------------+
                    |         Interfaces          |
                    |  CLI | MCP/HTTP | Slack*   |
                    +-------------+---------------+
                                  |
                                  v
                  +-------------------------------+
                  |  Architecture Intelligence     |
                  |  (single orchestration layer)  |
                  |  scan + query + drift + merge  |
                  +------+------------+------------+
                         |            |
                         v            v
                +-------------+   +-------------+
                | Knowledge   |   | Extraction  |
                | Graph       |   | (optional)  |
                +------+------+   +-------------+
                       |
                       v
                 +-----------+
                 | Language  |
                 | + Rules   |
                 +-----------+
```

`*` Slack is a later channel adapter, not a core dependency.

---

## 7. Module Decisions (Aggressive but Safe)

## Keep as Core (P0)
- `sruja-cli` (primary adoption surface)
- `sruja-scan` (code reality ingestion)
- `sruja-graph` (knowledge/state)
- `sruja-diff` (drift/delta logic)
- `sruja-language` (DSL)
- `sruja-diagnostics` (shared diagnostics contracts)

## Keep as Core-Orchestration (P0/P1)
- `sruja-chat` (backend orchestration; should become channel-agnostic service core)
- `sruja-mcp` (interface layer; align naming/transport with actual protocol)

## Keep but Scope Down (P1)
- `sruja-engine` (reduce default rules; keep strict essentials on by default)
- `sruja-export` (human docs/CI output; not core runtime path)
- `sruja-extract` (optional enhancement path)

## Deprioritize (no deletion yet)
- `sruja-app`
- `sruja-lsp`
- `sruja-wasm`
- `skill-lint`
- `sruja-watch`

Deletion is only after usage evidence confirms inactivity.

---

## 8. Immediate Simplification Moves (Reduce Maintained Code Surface)

1. **One shared orchestration path**
- remove duplicate scan->graph merge logic currently spread across CLI and chat
- expose shared service functions used by CLI + MCP + future channels

2. **Unify drift implementation**
- move CLI drift heuristics to `sruja-diff` core functions
- keep one drift model and one severity mapping

3. **Default-members optimization**
- set workspace default development loop to core crates
- keep secondary crates opt-in to reduce routine CI/dev overhead

4. **Rule profile strategy**
- default profile: essential safety/correctness rules
- advanced/strict profiles: opinionated architectural rules

5. **MCP clarity**
- if HTTP-only, call it API server
- if native MCP target is required, implement proper MCP transport explicitly

---

## 9. First-Run Experience (Must Be Obvious)

Introduce:

```bash
sruja quickstart -r .
```

Output should always include:
1. architecture inventory
2. top 3 drift/risk findings
3. health score
4. three actionable fixes
5. evidence references

No API key prompts in this flow.

---

## 10. Execution Plan

### Phase 1 (2-3 weeks): Adoption Surface
1. ~~Implement `quickstart` command.~~ **Done.**
2. Improve deterministic `why` explanations with evidence templates.
3. Unify drift logic with `sruja-diff`.
4. Reorder README/docs around no-key first value.

### Phase 2 (3-5 weeks): Runtime Consolidation
1. Extract shared orchestration service used by CLI and MCP.
2. Expand MCP/API tools for read/write/query/drift operations.
3. Reduce engine default rule set and add profiles.

### Phase 3 (optional channels)
1. Add native MCP transport if required by target clients.
2. Add Slack adapter only after backend API is stable.
3. Keep `sruja-app` as optional until channel usage data is clear.

---

## 11. Adoption Metrics (Investment Lens)

Track these from day 1:
1. Time-to-first-value (install -> first useful output)
2. `% users running quickstart again within 7 days`
3. `% repos enabling drift in CI`
4. Median findings resolved after drift report
5. `% sessions using LLM enrichment after deterministic onboarding`

If these move up, investment case is strong.

---

## 12. Final Recommendation

1. Make **zero-key deterministic intelligence** the default product.
2. Make **CLI the hero** for immediate and obvious adoption.
3. Keep **chat as backend orchestration**, not as mandatory user channel.
4. Treat **LLM and Slack as acceleration layers**, not entry requirements.
5. Reduce maintenance by consolidating runtime paths, not by deleting crates blindly.

This gives immediate user value, clear adoption path, and a cleaner long-term architecture with reversible decisions.

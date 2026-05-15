# Feature Tightening Spec

Generated: 2026-05-15

## Context

From [PRODUCT_FEATURE_ALIGNMENT_REPORT.md](./PRODUCT_FEATURE_ALIGNMENT_REPORT.md), several feature families are marked **Tighten**: they are aligned with the product spine but need clearer user moments, overlap resolution, or simplified surface. This spec provides implementation guidance for each tighten area.

## 1. Onboarding Consolidation

### Problem

`quickstart`, `discover explain`, `onboard`, `ai`, and `init --prompt` all help a user understand or start. Users see them as a toolbox instead of one coherent path.

### Target State

Keep all commands but clarify the decision tree:

| User question | Command | Output |
|---------------|---------|--------|
| "What is this repo?" | `sruja quickstart -r .` | Structural overview, scan findings, optional baseline |
| "Give me a complete architecture brief" | `sruja onboard -r .` | Full repo brief, optional LLM enrichment |
| "I'm about to work on X" | `sruja ai -r . --task "..."` | Paste-ready AI coding brief |
| "Set me up" | `sruja init -r .` | `.sruja/`, `.srujaignore`, optional prompt |

### Implementation Actions

1. **Update CLI help text** for each command to state its job clearly:
   - `quickstart`: "First look: structural overview and optional baseline"
   - `onboard`: "Complete architecture brief for human or AI reader"
   - `ai`: "Paste-ready briefing for an AI coding assistant"
   - `init`: "Set up Sruja in a repo (not a briefing command)"

2. **Update [book/src/reference/cli.md](../../book/src/reference/cli.md)** to add a "Which command?" decision table.

3. **Deprecate implicit aliases** that blur boundaries:
   - `overview` alias for `quickstart` — keep but do not promote
   - `start` alias for `init` — keep but document as setup-only

## 2. Health Metric Boundaries

### Problem

`status`, `doctor`, `health`, `context-score`, `quickstart`, and `daily` all report some form of health. Users cannot tell which to use.

### Target State

Each command has one clear job:

| Command | Job | Answer |
|---------|-----|--------|
| `status` / `doctor` | Truth freshness + baseline state | "Is my `repo.sruja` current?" |
| `health` | Architecture graph health (violations) | "Are there structural problems?" |
| `context-score` | AI-readiness (0-100) | "Can AI work effectively here?" |
| `quickstart` | Discovery + overview | "What is in this repo?" |
| `daily` / `review` | Action list | "What should I do today?" |

### Implementation Actions

1. **Update `--help` descriptions** to be distinct:
   - `health`: "Architecture health score from violations (0-100)"
   - `status`: "Truth freshness and baseline state"
   - `context-score`: "AI-readiness score (0-100)"

2. **Add diagnostic output** to show which metric type each command produces:
   - `health` output: "Health score: structural violations"
   - `status` output: "Truth status: reviewed/drifted/unknown"
   - `context-score` output: "Context score: AI preparedness"

3. **Update [book/src/reference/cli.md](../../book/src/reference/cli.md)** with a "Which command?" table for health/status.

## 3. Retrieval Naming

### Problem

`why`, `query`, `ai-context`, `focus`, MCP retrieval, semantic/BM25/hybrid search feel like many ways to ask the same thing.

### Target State

Teach a retrieval ladder:

1. **`sruja focus`** → before starting a task (blast radius, decisions, AI instructions)
2. **`sruja ai`** → paste-ready brief for AI assistant
3. **MCP tools** → inside AI editor (Cursor, Copilot, etc.)
4. **`sruja why`** / **`sruja query`** → investigation ("Why is this like this?")

### Implementation Actions

1. **Add CLI help text** clarifying each retrieval command's job.
2. **Update [docs/MESSAGING.md](../../docs/MESSAGING.md)** if it exists, or create a "Retrieval Ladder" section in docs.
3. **MCP tools docs** ([docs/mcp_tools_reference.md](../../docs/mcp_tools_reference.md)) should explicitly say: "Use focus before a task, ai for a paste-ready brief, why/query for investigation."

## 4. Agent Loop Boundaries

### Problem

`agent run/plan/apply` risks becoming "Sruja is a coding agent." It must stay bounded.

### Target State

Agent loop is only valid when:
- Every run is bounded by Sruja evidence
- Plan artifacts are reviewable before apply
- Verification steps are included
- Learnings are recorded to `.sruja/context/`

### Implementation Actions

1. **Add guardrail documentation** in [docs/AGENTS.md](../../docs/AGENTS.md) (or create if missing):
   - What agent loop can do: architecture-bounded tasks, refactor verification, impact analysis
   - What it cannot do: general coding, web search, unverified changes
   - Required verification before apply

2. **Document kill rules** for agent features:
   - Remove if it cannot name one canonical workflow
   - Remove if it works without Sruja evidence

## 5. Advanced DSL Blocks

### Problem

Contracts, state machines, loops, incidents, policy rules, deployment, SLOs, and views are all defensible but can feel like "model everything."

### Target State

Progressive disclosure: introduce advanced blocks only as needed by workflow, not as a giant language checklist.

### Implementation Actions

1. **Update [docs/LANGUAGE_SPECIFICATION.md](../../docs/LANGUAGE_SPECIFICATION.md)** to have a clear "Start here" path:
   - Core: C4 elements, relationships, nesting, sources, ownership
   - Progressive: requirements, ADRs, scenarios/flows, SLOs
   - Advanced: contracts, state machines, policies, incidents

2. **Add workflow tags** to each feature:
   - "Define intent" features: first-class
   - "Understand context" features: second-class
   - "Review change" features: triggered by intent

## 6. Learn / Agent Memory

### Problem

`learn`, evidence graph, learned facts can confuse users into thinking learned = reviewed truth.

### Target State

Position as "hypotheses from evidence, never truth." Strong UX labels so users do not confuse learned facts with reviewed architecture.

### Implementation Actions

1. **Add warning labels** to learn output: "⚠️ Learned hypothesis — not reviewed architecture"
2. **Update help text** to clarify: "Records architectural learnings and hypotheses, not reviewed truth"
3. **Add UX copy** distinguishing:
   - "Reviewed architecture" = `repo.sruja`
   - "Learned hypothesis" = `.sruja/context/learned/`

## Implementation Priority

| Priority | Actions | Status |
|----------|---------|--------|
| **P0** | CLI help text for onboarding and health metrics | Done (see `crates/sruja-cli/src/cli.rs`, command output labels) |
| **P1** | Docs: retrieval ladder, decision tables, MESSAGING | Done (`book/src/reference/cli.md`, `docs/mcp_tools_reference.md`, `docs/MESSAGING.md`) |
| **P2** | Agent loop guardrails; learn UX labels | Done (`AGENTS.md`, `learn` CLI + JSON `artifact_kind`) |
| **P3** | Advanced DSL progressive disclosure | Done (`docs/LANGUAGE_SPECIFICATION.md` modeling paths) |
| **Follow-up** | JSON `metric_type` / `metric_description` on `status`, `health`, `context-score` | Done |

## Kill Rules

Apply to every feature:

| Question | If no |
|----------|-------|
| Can it name one canonical workflow? | Fold into another feature |
| Does it strengthen the spine? | Demote to hidden/support |
| Would removing it weaken a main workflow? | Keep as Core |
| Is its user moment clear and distinct? | Tighten or remove |

## Verification

After changes:
1. Run `sruja --help` and verify help text is distinct
2. Run `sruja <command> --help` for each tightened command and verify clarity
3. Update PRODUCT_FEATURE_ALIGNMENT_REPORT.md verdict column if commands are reclassified
4. JSON: `cargo test -p sruja-cli learn_json` and `json_includes` filters cover `metric_type` / `artifact_kind` fields
5. Book [CLI reference](../book/src/reference/cli.md) documents JSON metric hints for operators

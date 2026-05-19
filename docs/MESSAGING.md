# Sruja product messaging

**Canonical positioning (use for website, docs, and pitch):**

- **Tagline:** Context engineering for the AI era.
- **One-liner:** Sruja brings context engineering to the AI era—use AI to generate and maintain architecture as code, backed by deterministic evidence from your codebase.
- **Short:** Architecture-as-code + evidence: analyze code, generate and validate `.sruja` files, export diagrams and docs, and produce citable context for AI coding.

Do not use the old framing (e.g. "developer-friendly language for defining, visualizing, and validating" or "governance / best practices / standardize" as the primary pitch). Lead with **context engineering for the AI era** and AI-driven discovery/maintenance.

---

**Product index: AI skill first**

- **Index on the AI skill.** The primary product surface is the **sruja-architecture skill**. Docs and quick start should lead with: install the skill first, then use it; the skill will guide users to install the CLI when needed (CLI is not a prerequisite before adding the skill). The skill is the needle mover.
- **Structural analysis backs, verifies, and enhances the skill.** Scan, discover, sync, and drift remain in the product and power the skill (evidence, drift detection, validation). We do not remove or downplay the value of structural analysis—we simply don't offer it as a **standalone user-facing tool** in the main flow.
- **Do not offer static/structural analysis as a primary tool.** Positioning "run quickstart" or "run drift" as the first thing users do is a distraction for us and for users. Those commands stay available for the skill (which runs them), for CI, and for scripting—but we don't lead with them or list them as the main way to get value.
- **Draft vs reviewed truth:** `sruja quickstart -r . --generate-baseline` and `sruja init --auto` write **`repo.sruja.draft`** (capped workspace map from manifests—evidence only). **`repo.sruja`** is reviewed architecture (skill + human edit + lint/drift). Never say "generate baseline" means you already have architecture.
- **Synthesis evidence:** `sruja sync` writes **`.sruja/author_evidence.json`** (capped bundle for LLM synthesis). The skill/MCP synthesizes domain architecture; static scan does not auto-write `repo.sruja`. Prefer author evidence over dumping **`.sruja/graph.json`** into prompts.
- **Proposals:** Incremental changes go to **`.sruja/proposals/<id>.json`** → `sruja propose approve <id>` → lint/drift. Headless: `sruja author propose --enrich-cmd '…'`.
- **Skill as primary interface; tools behind the scenes.** The skill is the primary interface. It uses scan/sync/discover behind the scenes. When `.sruja/context.json` exists and is fresh, the skill uses it; when not, the skill runs discover/sync so the user gets an answer without running any command first.

---

## Retrieval ladder (CLI + MCP)

Same question can be asked many ways; teach one ladder:

| Step | When | CLI | MCP (in editor) |
|------|------|-----|-------------------|
| 1 | Before editing a specific area | `sruja focus -r . --file <path>` | `sruja_get_focus_briefing` |
| 2 | Paste-ready task brief for an AI assistant | `sruja ai -r . --task "…"` | `sruja_get_task_context` |
| 3 | Deep graph queries from the IDE | — | Other `sruja mcp` tools (prefer readonly when exploring) |
| 4 | “Why is this like this?” investigation | `sruja why` / `sruja query` | `sruja_query_graph`, `sruja_explain_element`, search tools |

Do not present `why`, `query`, `focus`, `ai`, and MCP as interchangeable—they differ by **moment** (before task vs investigation) and **surface** (CLI vs editor).

---

**Why Sruja when AI can give architecture?**

AI without Sruja can propose architecture or generate code, but it is often ungrounded (it may invent components, dependencies, or boundaries) and ephemeral (no single source of truth in the repo). Sruja gives the AI deterministic, repo-specific context (scan/graph, drift, violations with sources) and a persistent artifact (architecture as code: lint, drift, version control). So: AI proposes and edits; Sruja provides context, validation, and persistence. As models get smarter, we don't replace them—we give them better evidence and better checks so their output is accurate and maintainable.

**Three pillars:**

- **Grounding** — The skill feeds the model real data (discover, context, graph) so it reasons on what's actually in the codebase, not on guesses. Without Sruja, the model can hallucinate modules and edges.
- **Validation and sync** — The skill uses sruja lint (valid DSL), sruja drift (declared vs actual), and intent check. So you get architecture that is valid and stays in sync with code, not a one-off diagram.
- **Persistence and reuse** — Architecture lives as repo.sruja in the repo: versionable, exportable, comparable over time. The model without Sruja gives ad-hoc text or Mermaid; with Sruja, the output is a first-class artifact the whole team and CI can use.

**One-liner for pitches:**
"Sruja doesn't replace the AI—it gives the AI your real codebase structure and validates what it produces, so architecture stays accurate and in the repo."

**Strategy: Rely on models getting smarter; improve from there**

Do not compete with the model. We don't try to outdo the model at "being smart." We assume models will keep improving at reasoning and generation. Complement and improve from there: Sruja's job is to provide better inputs (evidence, graph, scoped discovery) and better checks (lint, drift, intent) so that whatever the model produces is grounded and maintainable. As models get smarter, they get better at using that evidence—so Sruja's value grows with model capability.

**Design principle:** Every feature (skill, discover, sync, drift, progressive discovery) should be answerable with: "This gives the model better evidence or better validation so its output is more accurate and stays in sync." If we can't say that, we should question the feature.

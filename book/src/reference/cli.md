# CLI reference

## Which command?

### Onboarding & setup

| User question | Command | Output |
|---------------|---------|--------|
| "What is this repo?" | `sruja quickstart -r .` | Structural overview, scan findings, optional baseline |
| "Give me a complete architecture brief" | `sruja onboard -r .` | Full repo brief with trust signals |
| "I'm about to work on X" | `sruja ai -r . --task "..."` | Paste-ready AI coding brief |
| "Set me up" | `sruja init -r .` | `.sruja/`, `.srujaignore`, optional prompt |

### Health & status

| User question | Command | Output |
|---------------|---------|--------|
| "Is my `repo.sruja` current?" | `sruja status -r .` | Truth freshness + baseline state |
| "Is my `repo.sruja` current?" (alias) | `sruja doctor -r .` | Same as `status` |
| "Are there structural problems?" | `sruja health -r .` | Architecture health score (0-100) from violations |
| "Can AI work effectively here?" | `sruja context-score -r .` | AI-readiness score (0-100) |
| "What should I do today?" | `sruja review -r .` | Action list from evidence refresh (`daily` alias) |

### Retrieval ladder

| Step | Command | When to use |
|------|---------|-------------|
| 1. Before starting a task | `sruja focus -r . --file <path>` | Blast radius, decisions, AI instructions |
| 2. Paste-ready AI brief | `sruja ai -r . --task "..."` | Share context with AI assistant |
| 3. Inside AI editor (MCP) | MCP tools | Cursor, Copilot, Claude integration |
| 4. Investigation | `sruja why "..."` / `sruja query "..."` | "Why is this like this?" |

### Discover vs onboarding

| Need | Command |
|------|---------|
| Scanner explain / repomap / question bank for AI or debugging | `sruja discover …` |
| First human read of the repo | `sruja quickstart -r .` |
| Full architecture brief | `sruja onboard -r .` |

## Stable commands

| Command | Description |
|---------|-------------|
| `sruja quickstart -r .` | First look: structural overview and baseline generation |
| `sruja onboard -r .` | Single onboarding brief (deterministic; optional LLM enrichment with `--enrich`) |
| `sruja lint <file>` | Validate `.sruja` file |
| `sruja sync -r .` | Refresh evidence and cached graph/context |
| `sruja status -r .` | Truth freshness and baseline state |
| `sruja review -r .` | Review workflow: refresh evidence, detect drift, summarize |
| `sruja drift -r . -a repo.sruja` | Declared vs actual structure |
| `sruja health -r .` | Architecture health score from violations |
| `sruja context-score -r .` | AI-readiness score (0-100) |
| `sruja publish -r .` | Export repo bundle for federation |
| `sruja compose -i <bundle.json> -o system.index.json` | Compose bundles into a system index |

## Other commands

| Command | Description |
|---------|-------------|
| `sruja export json <file>` | Export to JSON |
| `sruja export markdown <file>` | Export to Markdown |
| `sruja export mermaid <file>` | Export to Mermaid |
| `sruja fmt <file>` | Format DSL |
| `sruja tree <file>` | Print element tree |
| `sruja watch -r .` | Keep feedback live while coding |
| `sruja doctor -r .` | Alias for `status` |
| `sruja drift --ci -r .` | **Preferred** CI drift check (GitHub Actions annotations; replaces deprecated `check`) |
| `sruja check -r .` | Deprecated alias for drift-style CI output — use `drift --ci` |
| `sruja baseline -r .` | Snapshot violations for baseline ignore in CI |
| `sruja ai -r . --task "..."` | Paste-ready AI coding brief (optional `--enrich` / `--enrich-cmd`) |
| `sruja focus -r . --file <path>` | Context briefing before starting work |
| `sruja why "..."` | Investigate architecture decisions |
| `sruja query "..."` | Query registry for elements and relationships |
| `sruja ai-context -r .` | Structured context for AI editor integration |

**Repo root:** Prefer `-r` / `--repo` for the repository directory. Some commands also accept `--path` as an alias for backwards compatibility.

## JSON output: metric hints

When `-f json` (or the command’s JSON mode) is used, some commands add stable **`metric_type`** and **`metric_description`** fields so parsers and dashboards do not confuse different scores:

| Command | `metric_type` | Role |
|---------|---------------|------|
| `sruja status -f json` | `truth_freshness` | Baseline / sync / truth status (not structural health alone) |
| `sruja health -f json` | `structural_health` | Violations vs declared `repo.sruja` (0–100) |
| `sruja context-score -f json` | `ai_readiness` | AI preparedness (0–100) |
| `sruja learn -f json` | *(see `artifact_kind`)* | `artifact_kind` is `learned_hypothesis` — not reviewed DSL truth |

## Aliases

- `start` = `init` (setup only)
- `daily` = `review`
- `overview` = `quickstart`
- `doctor` = `status`

Run `sruja --help` for full options.

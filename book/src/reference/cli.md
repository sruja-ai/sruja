# CLI reference

Stable pilot surface:

| Command | Description |
|---------|-------------|
| `sruja quickstart -r .` | First look: structural overview and baseline generation |
| `sruja onboard -r .` | Single onboarding brief (deterministic; optional LLM enrichment with `--enrich`) |
| `sruja lint <file>` | Validate `.sruja` file |
| `sruja sync -r .` | Refresh evidence and cached graph/context |
| `sruja status -r .` | Repo health and truth status |
| `sruja review -r .` | Review workflow: refresh evidence, detect drift, summarize |
| `sruja drift -r . -a repo.sruja` | Declared vs actual structure |
| `sruja publish -r .` | Export repo bundle for federation |
| `sruja compose -i <bundle.json> -o system.index.json` | Compose bundles into a system index |

Other commands:

| Command | Description |
|---------|-------------|
| `sruja export json <file>` | Export to JSON |
| `sruja export markdown <file>` | Export to Markdown |
| `sruja export mermaid <file>` | Export to Mermaid |
| `sruja fmt <file>` | Format DSL |
| `sruja tree <file>` | Print element tree |
| `sruja watch -r .` | Keep feedback live while coding |
| `sruja doctor -r .` | Quick repo health and last evidence refresh |
| `sruja check -r .` | CI-oriented drift check (default format suited for GitHub Actions annotations) |
| `sruja baseline -r .` | Snapshot violations for baseline ignore in CI |
| `sruja ai -r . --task "..."` | Paste-ready AI coding brief (optional `--enrich` / `--enrich-cmd`) |

**Repo root:** Prefer `-r` / `--repo` for the repository directory. Some commands also accept `--path` as an alias for backwards compatibility.

Aliases:

- `start` = `init`
- `daily` = `review`
- `overview` = `quickstart`
- `doctor` = `status`

Run `sruja --help` for full options.

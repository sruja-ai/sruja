# CLI reference

Stable pilot surface:

| Command | Description |
|---------|-------------|
| `sruja quickstart -r .` | First look: structural overview and baseline generation |
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

Aliases:

- `start` = `init`
- `daily` = `review`
- `overview` = `quickstart`
- `doctor` = `status`

Run `sruja --help` for full options.

# CLI reference

Core commands:

| Command | Description |
|---------|-------------|
| `sruja start -r . --prompt` | Set up `.sruja/` and generate an AI-ready baseline prompt |
| `sruja daily -r .` | Daily review: refresh evidence and summarize drift/questions |
| `sruja doctor -r .` | Quick repo health, truth status, and last evidence refresh |
| `sruja watch -r .` | Keep architecture feedback live while coding |
| `sruja lint <file>` | Validate `.sruja` file |
| `sruja fmt <file>` | Format DSL |
| `sruja tree <file>` | Print element tree |
| `sruja sources -a <file>` | List architecture index source bindings |
| `sruja export json <file>` | Export to JSON |
| `sruja export markdown <file>` | Export to Markdown |
| `sruja export mermaid <file>` | Export to Mermaid |

Workflow aliases:

- `start` = `init`
- `daily` = `review`
- `overview` = `quickstart`
- `doctor` = `status`

Run `sruja --help` for full options.

# CLI refactoring plan

## Done

- **Command split**: Implementation moved from a single `_commands.rs` into domain modules:
  - `commands/error.rs` – `CliError` and conversions
  - `commands/version.rs` – version command
  - `commands/dsl.rs` – file-based DSL (lint, export, fmt, list, tree, diff, explain, import, lsp, validate, compile)
  - `commands/scan.rs` – scan, why, drift, quickstart and drift/quickstart printing
  - `commands/analyze.rs` – complexity, semantic, comprehensive analyze
  - `commands/intent.rs` – intent check and propose
- **Unified types**: `NodeKind`/`EdgeKind` live in `sruja-language` (`ast`); `sruja-graph` and `sruja-scan` use or re-export them where appropriate.
- **Dependency consistency**: `sruja-intent` uses workspace `thiserror`.

## Future (optional)

- Extract shared “parse file + enrich diagnostics” into a single helper used by dsl commands.
- Consider a small `print` or `format` module for “text vs json” output helpers to reduce duplication.
- Add unit tests per command module where useful.

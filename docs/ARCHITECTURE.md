# Sruja Architecture & Code Organization

## Overview

Sruja is a Go project composed of a CLI entrypoint, a language/AST and parser, a validation engine, exporters, LSP server, and developer experience helpers. The website lives under `apps/website/` and is built with Astro.

## Directory Layout

```
cmd/
  sruja/             # CLI entrypoint (Cobra) and subcommands
  wasm/              # WASM entrypoint for browser usage
pkg/
  language/          # DSL parser, AST, printer
  engine/            # Validation engine and rules
  export/            # Exporters (JSON, views)
    json/            # JSON exporter
    views/           # View generation
  import/            # Importers
    json/            # JSON importer
  lsp/               # Language Server Protocol implementation
  dx/                # Developer experience helpers (errors, formatting, explainer)
  diagnostics/       # Diagnostic system for errors and warnings
  config/            # Configuration management
internal/            # Internal utilities
  converter/         # Internal conversion utilities
  lister/            # Internal listing utilities
examples/            # Example .sruja models
apps/                # Frontend applications
  website/           # Astro-based website (docs, courses, tutorials, blog)
  designer/          # Interactive designer application
  vscode-extension/  # VS Code language support
  social-publish/    # Social media publishing tools
  storybook/         # Component documentation
packages/            # Shared TypeScript packages
  shared/            # Shared utilities and types
  ui/                # UI component library
  layout/            # Layout algorithms for diagrams
  diagram/           # Diagram rendering utilities
  eslint-config/     # Shared ESLint configuration
.github/workflows    # CI and release pipelines
scripts/             # Installer and tooling scripts
```

## CLI (cmd/sruja)

- Root command and subcommands are defined with Cobra in `cmd/sruja/cobra.go:1`.
- `main.go` initializes the CLI via `Execute()` and keeps existing command implementations as Go functions.
- Commands call `run*` functions:
  - `version`: `cmd/sruja/version.go:15`
  - `compile`: `cmd/sruja/compile.go:16`
  - `lint`: `cmd/sruja/lint.go:16`
  - `fmt`: `cmd/sruja/fmt.go:13`
  - `export`: `cmd/sruja/export.go:15`
  - `import`: `cmd/sruja/import.go:13`
  - `list`: `cmd/sruja/list.go:15`
  - `tree`: `cmd/sruja/tree.go:12`
  - `diff`: `cmd/sruja/diff.go:15`
  - `explain`: `cmd/sruja/explain.go:12`
  - `score`: `cmd/sruja/score.go:29`
  - `change`: `cmd/sruja/change.go:16` (with `create` and `validate` subcommands)
  - `init`: `cmd/sruja/init.go:20`
  - `lsp`: `cmd/sruja/lsp_cmd.go` (Language Server Protocol server)

### Adding a New Command

- Add a `*cobra.Command` in `cmd/sruja/cobra.go` and wire it to a `runXxx` function.
- Keep the run function explicit about inputs/outputs (`stdout`, `stderr`) and avoid panics.

### Shell Completion

- Cobra generates standard completions:
  - `sruja completion bash`
  - `sruja completion zsh`
  - `sruja completion fish`

## Language & AST (pkg/language)

- The AST types and parser are defined in `pkg/language/ast.go:1` and `pkg/language/parser.go:143`.
- Key types: `Architecture`, `System`, `Container`, `Component`, `Person`, `DataStore`, `Queue`, `Relation`, `Scenario`, `ADR`.
- Parser helpers convert top-level file items into architecture items: `pkg/language/parser.go:143`.
- Example tests cover metadata, journeys, getters:
  - Metadata parsing: `pkg/language/metadata_parsing_test.go:1`
  - Journeys parsing: `pkg/language/parser_journey_test.go:1`
  - AST getters: `pkg/language/ast_struct_test.go:128`

### Printing & Formatting

- Formatted output is produced by `language.Printer` in `pkg/language/printer.go` (see `runFmt` usage at `cmd/sruja/fmt.go:13`).

## Validation Engine (pkg/engine)

- Validation is orchestrated via `engine.NewValidator()` in `cmd/sruja/compile.go:51` and `cmd/sruja/lint.go:51`.
- Built-in rules registered for compilation and lint commands include unique IDs, valid references, cycle detection, and orphan detection.

## Exporters (pkg/export)

- JSON exporter lives in `pkg/export/json/` with comprehensive tests:
  - Architecture conversion: `pkg/export/json/architecture_conversion_test.go`
  - DSL to JSON integration: `pkg/export/json/dsl_to_json_integration_test.go`
  - View export: `pkg/export/json/view_export_test.go`
- View generation in `pkg/export/views/`:
  - Pattern includes: `pkg/export/views/pattern_include_test.go`
  - Tag mapping: `pkg/export/views/tag_map_test.go`
- CLI wiring for export: `cmd/sruja/export.go:15`.
- Currently supported formats: `json` (markdown and mermaid exports are disabled, use TypeScript exporters in frontend apps).

## Importers (pkg/import)

- JSON importer lives in `pkg/import/json/`:
  - JSON import: `pkg/import/json/json.go`
  - Integration tests: `pkg/import/json/json_integration_test.go`
- CLI wiring for import: `cmd/sruja/import.go:13`.

## Language Server Protocol (pkg/lsp)

- LSP implementation provides IDE support for Sruja files:
  - Server: `pkg/lsp/server.go`
  - Workspace management: `pkg/lsp/workspace.go`
  - Diagnostics: `pkg/lsp/diagnostics.go`
  - Completion: `pkg/lsp/completion.go`
  - Hover: `pkg/lsp/hover.go`
  - Definition: `pkg/lsp/definition.go`
  - References: `pkg/lsp/references.go`
  - Symbols: `pkg/lsp/symbols.go`
  - Formatting: `pkg/lsp/formatting.go`
  - Code actions: `pkg/lsp/code_actions.go`
- CLI entrypoint: `cmd/sruja/lsp_cmd.go`

## Diagnostics (pkg/diagnostics)

- Diagnostic system for structured error and warning reporting:
  - Diagnostic types: `pkg/diagnostics/diagnostics.go`
  - Diagnostic codes: `pkg/diagnostics/codes.go`
- Used by parser, validator, and LSP server.

## Developer Experience (pkg/dx)

- Enhanced error formatting and context: `pkg/dx/errors_test.go:95`, `pkg/dx/explainer.go:144`.
- The `ErrorEnhancer` augments parser/validator errors with file and line context.
- Explainer provides detailed explanations of architecture elements: `pkg/dx/explainer.go`.

## Frontend Applications (apps/)

### Website (apps/website/)

- Astro configuration: `apps/website/astro.config.mjs`.
- Content: Markdown/MD files in `apps/website/src/content/**` organized by type (docs, courses, tutorials, blog).
- Features: Playground with WASM support, viewer integration.
- Build: Astro static site generator with TypeScript/React components.

### Designer (apps/designer/)

- Interactive designer for testing Sruja code in the browser.
- Uses WASM for parsing and validation.
- Located at `apps/designer/`.

### VS Code Extension (apps/vscode-extension/)

- Language support for `.sruja` files in VS Code.
- Syntax highlighting, LSP integration, and preview features.
- Located at `apps/vscode-extension/`.

## CI, Releases, and Installer

- CI workflow: `/.github/workflows/ci.yml:1` (build, tests, lint, example compilation).
- Release workflow uses GoReleaser action v6: `/.github/workflows/release.yml:20`.
- GoReleaser config (v2): `/.goreleaser.yaml:1`.
  - Builds `linux/darwin` on `amd64/arm64`.
  - Archives named `sruja_${OS}_${ARCH}.tar.gz` (e.g., `sruja_Darwin_arm64.tar.gz`).
- Install script fetches matching archives: `scripts/install.sh:42`.

## Coding Conventions

- Keep packages cohesive and functions small; explicit error handling.
- Avoid panics in library code; return errors with context.
- Maintain tests adjacent to packages; prefer table-driven tests.
- Minimize dependencies; pin via `go.mod`.

## Local Development

- Build CLI: `make build` → outputs `bin/sruja`.
- Run tests: `make test`, coverage: `make test-coverage`.
- Lint/format: `make lint`, `make fmt`.
- Try examples: `./bin/sruja compile examples/example.sruja`.

## Extending Sruja

- New export format: implement under `pkg/export/<format>/` and add wiring in `runExport` (`cmd/sruja/export.go:15`).
- New import format: implement under `pkg/import/<format>/` and add wiring in `runImport` (`cmd/sruja/import.go:13`).
- New validation rule: define a type implementing a validation interface and register via `engine.NewValidator()` calls in `runCompile`/`runLint`.
- New CLI command: add a Cobra command in `cmd/sruja/cobra.go` and a corresponding `runXxx` function.
- LSP features: extend `pkg/lsp/` to add new language server capabilities.

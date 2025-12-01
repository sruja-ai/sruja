# Sruja Architecture & Code Organization

## Overview

Sruja is a Go project composed of a CLI entrypoint, a language/AST and parser, a validation engine, exporters, and developer experience helpers. The docs site lives under `learn/` and is built with Hugo.

## Directory Layout

```
cmd/sruja            # CLI entrypoint (Cobra) and subcommands
pkg/language         # DSL parser, AST, printer
pkg/engine           # Validation engine and rules
pkg/export           # Exporters (D2, …)
pkg/dx               # Developer experience helpers (errors, formatting)
examples             # Example .sruja models
learn                # Hugo-based docs/learning site
.github/workflows    # CI and release pipelines
scripts              # Installer and tooling scripts
```

## CLI (cmd/sruja)

- Root command and subcommands are defined with Cobra in `cmd/sruja/cobra.go:1`.
- `main.go` initializes the CLI via `Execute()` and keeps existing command implementations as Go functions.
- Commands call `run*` functions:
  - `version`: `cmd/sruja/version.go:13`
  - `compile`: `cmd/sruja/main.go:208`
  - `lint`: `cmd/sruja/main.go:105`
  - `fmt`: `cmd/sruja/main.go:148`
  - `export`: `cmd/sruja/main.go:173`
  - `list`: `cmd/sruja/list.go:15`
  - `tree`: `cmd/sruja/main.go:92`
  - `diff`: `cmd/sruja/main.go:94`

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
- Key types: `Architecture`, `System`, `Container`, `Component`, `Person`, `DataStore`, `Queue`, `Relation`, `Scenario`, `ADR`, domain modeling constructs (`Entity`, `ValueObject`, `DomainEvent`).
- Parser helpers convert top-level file items into architecture items: `pkg/language/parser.go:143`.
- Example tests cover metadata, journeys, getters:
  - Metadata parsing: `pkg/language/metadata_parsing_test.go:1`
  - Journeys parsing: `pkg/language/parser_journey_test.go:1`
  - AST getters: `pkg/language/ast_struct_test.go:128`

### Printing & Formatting

- Formatted output is produced by `language.Printer` in `pkg/language/ast.go` (see `runFmt` usage at `cmd/sruja/main.go:167`).

## Validation Engine (pkg/engine)

- Validation is orchestrated via `engine.NewValidator()` in `cmd/sruja/main.go:228`.
- Built-in rules registered for compilation and lint commands include unique IDs, valid references, cycle detection, and orphan detection.

## Exporters (pkg/export)

- D2 exporter lives in `pkg/export/d2/` with tests in:
  - Container/relations: `pkg/export/d2/d2_test.go:42`
  - Complete features: `pkg/export/d2/d2_complete_test.go:33`
  - Edge cases: `pkg/export/d2/d2_edge_cases_test.go:159`
- CLI wiring for export: `cmd/sruja/main.go:197`.

## Developer Experience (pkg/dx)

- Enhanced error formatting and context: `pkg/dx/errors_test.go:95`, `pkg/dx/explainer.go:144`.
- The `ErrorEnhancer` augments parser/validator errors with file and line context.

## Docs & Learn Site (learn/)

- Hugo configuration: `learn/hugo.toml:1`.
- JS entrypoint initializes navigation, code blocks, theme, and tracking: `learn/assets/js/main.ts:1`.
- Top navigation components: `learn/assets/js/components/TopNavigation.tsx:1` and utilities: `learn/assets/js/utils/navigation.ts:1`.
- Learn content: Markdown/MD files in `learn/content/**` (e.g., Getting Started: `learn/content/docs/getting-started.md:1`).

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

- New export format: implement under `pkg/export/<format>/` and add wiring in `runExport` (`cmd/sruja/main.go:173`).
- New validation rule: define a type implementing a validation interface and register via `engine.NewValidator()` calls in `runCompile`/`runLint`.
- New CLI command: add a Cobra command in `cmd/sruja/cobra.go` and a corresponding `runXxx` function.


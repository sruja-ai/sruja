# Contributing & Community

Central hub for Sruja contributions and community resources. Use this as the single source of truth for setup, workflow, policies, and ways to get involved.

## Project Overview

- Language and CLI in Go (Go 1.25 per `go.mod`)
- WebAssembly build for docs playground
- Examples, Hugo‑based docs/learning site
- Repo: https://github.com/sruja-ai/sruja

## Development Setup

### Prerequisites

- `Go >= 1.25` (CI uses `1.25.4`)
- Git

Optional:

- `golangci-lint` (installed automatically by `make lint`)
- `wasm-opt` (optimizes WASM artifacts)

### Clone the repository

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
```

### Install dependencies

```bash
make install
```

### Build the CLI

```bash
make build
./bin/sruja --help
```

### Run tests and coverage

```bash
make test
make test-coverage      # prints summary
make test-coverage-html # writes coverage.html
```

### Format and lint

```bash
make fmt
make lint
```

### Try examples

```bash
./bin/sruja compile examples/example.sruja
./bin/sruja lint examples/example.sruja
```

### Build WebAssembly for learn playground

```bash
make build-learn
```

Outputs go to `learn/static/` including compressed variants.

For additional context, see `README.md` and `Makefile`.

## Working With Examples

- Example `.sruja` models live under `examples/`
- After building the CLI:

```bash
./bin/sruja compile examples/example.sruja
./bin/sruja lint examples/example.sruja
```

CI compiles and lints selected examples to catch regressions.

## Contribution Workflow

Use this workflow to propose changes via pull requests.

### 1. Fork and branch

- Fork the repo on GitHub
- Create a topic branch from `main`, e.g. `feature/…`, `fix/…`, or `docs/…`

### 2. Implement and validate

- Build and test locally: `make build`, `make test`
- Run formatting and linting: `make fmt`, `make lint`
- Add or update tests for new behavior

### 3. Commit style

Follow Conventional Commits:

- `feat: add XYZ capability`
- `fix: prevent panic in parser`
- `docs: update contribution workflow`
- `refactor: simplify validator`
- `test: cover edge cases`

Optional scope: `feat(language): …`

### 4. Open a Pull Request

- Push your branch and open a PR against `main`
- Fill out `.github/pull_request_template.md`
- Ensure CI is green (build, tests, lint)

### 5. Review and iterate

- Address feedback with follow‑up commits
- Keep the PR focused; split large changes if needed

### 6. Merge

- Squash or rebase per maintainer guidance
- Post‑merge: clean up your branch if it’s no longer needed

## Branching Model

- Create topic branches off `main` using `feature/…`, `fix/…`, or `docs/…`
- Keep changes focused; split large PRs when possible

## Pull Request Checklist

- `make test`, `make fmt`, `make lint` run clean locally
- Add/update tests for new behavior
- Update docs/examples if usage changes
- CI passes (build, tests, lint)
- Use the PR template (`.github/pull_request_template.md`)

## Coding Guidelines

- Small, well‑named functions; cohesive packages
- Avoid breaking public APIs; document changes clearly when necessary
- Maintain meaningful test coverage
- Explicit error handling; avoid panics in library code
- Keep dependencies minimal and pinned via `go.mod`
- Follow the **[Design Philosophy](DESIGN_PHILOSOPHY.md)** when proposing language changes.

## Contributor Guide

Welcome! This section points you to key resources and principles.

### Find an issue

- Browse open issues on GitHub
- Look for `good first issue` labels suitable for newcomers

### Ask questions

- GitHub Discussions
- Discord community

### Principles

- Keep PRs small, focused, and well‑tested
- Prefer explicit error handling and minimal dependencies
- Document externally visible behavior

## Get in Touch

- GitHub Discussions
- Discord: https://discord.gg/QMCsquJq

## Contributing Content

To add content to the Sruja Learn site (courses, tutorials, blog posts, etc.), see:

**📖 [Content Contribution Guide](CONTENT_CONTRIBUTION_GUIDE.md)**

This guide covers:
- Creating courses, tutorials, blogs, and docs
- Content structure and best practices
- Validation and workflow
- Troubleshooting

## Documentation Notes

- User docs live under `learn/` (Hugo site). Changes under `docs/**` trigger the docs deploy workflow.
- Align learning materials and user‑facing docs across `learn/` when applicable.

## Reporting Issues and Requesting Features

- Use GitHub Issues with clear description, reproduction, and expected behavior
- Tag appropriately (bug, enhancement, docs)
- Consider draft PRs if you have a prototype

## Community Expectations

- Be respectful and constructive
- Provide actionable reviews and respond to feedback
- Prefer async, documented decisions (link ADRs or issues where relevant)

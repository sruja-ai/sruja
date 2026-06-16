# Contributing to Sruja

Welcome! This guide will help you get started contributing to Sruja. Whether you're fixing a typo or adding a major feature, your contributions are welcome.

## 🎯 New to Contributing?

**Start here:** [Getting Started](GETTING_STARTED_SKILL.md)

This step-by-step guide walks you through making your first contribution, even if you're new to the project.

## Quick Links

- 🐛 **Find Issues**: [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)
- 📖 **Development Guide**: [Development Practices](DEVELOPMENT.md)
- 🐕 **Internal Dogfooding**: [Dogfooding Playbook](internal/dogfooding-playbook.md)
- 📐 **Stack**: Rust (CLI, language, engine, export, scan/diff/graph/intent, WASM), mdBook (docs), VS Code extension (TypeScript, WASM-powered language features).
- 💬 **Get Help**: [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)
- **GitHub Pages deploy targets**: [Related repositories](RELATED_REPOSITORIES.md)

## Project Overview

Sruja is a Rust-focused repo containing:

- **Language and CLI**: Parser, validator, export; CLI via `crates/sruja-cli`
- **Book**: mdBook documentation in `book/`
- **Examples**: Canonical book-backed examples in `book/valid-examples/`
- **VS Code extension**: editor integration in `extension/`

The **VS Code extension** integrates directly with the WASM build for parsing/validation/preview (no standalone Rust LSP server crate).

### ⚠️ Important: Deployment Repositories

This repository (`sruja-ai/sruja`) is the **main development repository** where all contributions should be made.

**Do not contribute to these deployment-only repositories:**

- `sruja-ai/staging-website` - Staging deployment (path-filtered push to `main` and/or manual `workflow_dispatch`; see [Related repositories](RELATED_REPOSITORIES.md))
- `sruja-ai/prod-website` - Production deployment (manual promote from `main`; see [Deploy to Production](../.github/workflows/README.md#deploy-to-production))

These repositories are **automatically updated** by GitHub Actions workflows and contain only built static files for GitHub Pages hosting. They are **read-only** for contributors. See [Related repositories](RELATED_REPOSITORIES.md) for staging vs production triggers and where to file issues.

**All contributions should be made to this repository (`sruja-ai/sruja`):**

- ✅ Open issues here
- ✅ Submit pull requests here
- ✅ Report bugs here
- ✅ Contribute code, docs, and examples here

The deployment repositories will be automatically updated when your changes are merged to `main` or released.

## Scoped tasks (for contributors and agents)

Use a **single clear anchor** for non-trivial work: a GitHub issue URL, or a concrete path plus expected behavior. Write **short acceptance criteria** (what “done” means) before large diffs. Before opening a PR, run **`just check`** (or `make check`) (format, clippy, tests) and, when you touch architecture baselines, **`sruja drift -r .`** / **`sruja lint`** as described in [AGENTS.md](../AGENTS.md). For Dependabot and MSRV decisions, see [MSRV and dependencies](MSRV_AND_DEPENDENCIES.md).

## Development Setup

### Prerequisites

- **Rust** (stable)
- **Git**

Optional:

- `wasm-opt` (optimizes WASM artifacts)
- `wasm-pack` (for `just wasm` / `just wasm-nodejs`)

### Quick Setup

```bash
# 1. Clone the repository
git clone https://github.com/sruja-ai/sruja.git
cd sruja

# 2. Install dependencies
just install

# 3. Build the CLI
just build

# 4. Verify it works
./target/release/sruja --help
```

### Common Commands

```bash
# Run tests
just test

# Format code
just fmt

# Lint code
just lint

# Build everything
just build

# Try examples
./target/release/sruja compile book/valid-examples/getting-started.sruja
./target/release/sruja lint book/valid-examples/getting-started.sruja
```

For more details, see [Development Guide](DEVELOPMENT.md).

## Developer Tools

Sruja includes specialized developer tools located in the `dev-tools/` directory. These are **dev-only tools** not used in production.

### Diagram and export

Diagram export is handled by the Rust crate `sruja-export` (Mermaid, etc.). To test changes:

```bash
cargo build --release -p sruja-cli
./target/release/sruja export mermaid path/to/file.sruja
```

**Important:** Do not use `window.__*__` globals in production code. These are for E2E testing only.

## Working With Examples

Example `.sruja` models live under `book/valid-examples/`. After building the CLI:

```bash
# Compile an example
./target/release/sruja compile book/valid-examples/getting-started.sruja

# Lint an example
./target/release/sruja lint book/valid-examples/getting-started.sruja

# Export to different formats
./target/release/sruja export json book/valid-examples/getting-started.sruja
./target/release/sruja export markdown book/valid-examples/getting-started.sruja
```

CI compiles and lints selected examples to catch regressions.

## Contribution Workflow

Use this workflow to propose changes via pull requests.

### 1. Fork and branch

- Fork the repo on GitHub
- Create a topic branch from `main`, e.g. `feature/…`, `fix/…`, or `docs/…`

### 2. Implement and validate

- Build and test locally: `just build`, `just test`
- Run formatting and linting: `just fmt`, `just lint`
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

- `just test`, `just fmt`, `just lint` run clean locally
- Add/update tests for new behavior
- Update docs/examples if usage changes
- CI passes (build, tests, lint)
- Use the PR template (`.github/pull_request_template.md`)

## Coding Guidelines

- Small, well‑named functions; cohesive packages
- Avoid breaking public APIs; document changes clearly when necessary
- Maintain meaningful test coverage
- Explicit error handling; avoid panics in library code
- Keep dependencies minimal; pin where appropriate (Cargo.lock for Rust)
- Follow the **[Design Philosophy](DESIGN_PHILOSOPHY.md)** when proposing language changes.

## Ways to Contribute

### No Code Required

- 📝 **Documentation**: Fix typos, improve clarity, add examples
- 🐛 **Testing**: Test features and report bugs
- 💡 **Examples**: Add example architectures to `book/valid-examples/` (and the book gallery)
- ✍️ **Content**: Write tutorials, blog posts, or courses
- 🌐 **Translation**: Translate documentation

### Beginner-Friendly Code

- ✅ **Tests**: Add test cases for existing functionality
- 🐛 **Bugs**: Fix small bugs
- 📝 **Error Messages**: Improve error messages and help text
- 📚 **Examples**: Add more example architectures
- 🎨 **CLI**: Improve CLI help text and user experience

### More Advanced

- 🔧 **Features**: Implement new features
- 🚀 **Export Formats**: Add new export formats
- 🔍 **Validation**: Add validation rules
- 🛠️ **Tooling**: Improve development tooling

## Finding Work

### GitHub Issues

- Browse [open issues](https://github.com/sruja-ai/sruja/issues)
- Look for `good first issue` labels
- Filter by `help wanted` for areas needing assistance

### Contribution Ideas

If there aren't many issues yet, here are common tasks that don't need issues:

- Fix typos in documentation
- Add examples to `book/valid-examples/`
- Improve error messages
- Add test cases
- Write tutorials or blog posts

You can start working on these right away! Open a draft PR to show what you're working on.

## Getting Help
- 💬 **GitHub Discussions**: Ask questions and share ideas
- 📝 **GitHub Issues**: Report bugs or request features
- 💬 **PR Comments**: Ask for help on your pull request

## Principles

- **Keep PRs small and focused**: Easier to review and merge
- **Test your changes**: Run `just test` before submitting
- **Follow conventions**: Use Conventional Commits, run `just fmt`
- **Start small**: You can always contribute more later!
- **Ask questions**: We're here to help!

## Get in Touch

- GitHub Discussions

## Contributing Content

To add or edit docs, courses, and tutorials (mdBook in `book/`):

- **Book content**: Add/edit files in `book/src/`
- **Examples**: Add to `book/valid-examples/`
- **Validate**: Run `just book` to build and verify

## Reporting Issues

When reporting bugs or requesting features:

1. **Check existing issues**: Search to see if it's already reported
2. **Use clear titles**: Describe the issue or feature clearly
3. **Provide context**: Include steps to reproduce, expected vs actual behavior
4. **Tag appropriately**: Use labels like `bug`, `enhancement`, `docs`
5. **Consider draft PRs**: If you have a prototype, open a draft PR

## Community Guidelines

- **Be respectful**: Treat everyone with kindness and respect
- **Be constructive**: Provide actionable feedback
- **Be patient**: Reviews take time, especially for maintainers
- **Be collaborative**: Work together to improve Sruja

## Additional Resources

- **Architecture**: See [architecture/README.md](architecture/README.md) for architecture overview and [crates/sruja-engine/ARCHITECTURE.md](../crates/sruja-engine/ARCHITECTURE.md) for engine internals
- **Design Philosophy**: See [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) for language design principles
- **Language Spec**: See [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) for complete DSL reference

---

## Related GitHub Repositories

The [sruja-ai](https://github.com/sruja-ai) organization on GitHub hosts the product, generated documentation sites, and (optionally) additional private or downstream repositories used in a composed `system.index.json`. This section summarizes the **public** repos most contributors interact with.

### Public repositories

- **[sruja-ai/sruja](https://github.com/sruja-ai/sruja)** — Source of truth: Rust workspace, VS Code extension, mdBook sources (`book/`), CLI, WASM, skills, and CI. Issues and [Discussions](https://github.com/sruja-ai/sruja/discussions) live here.
- **[sruja-ai/staging-website](https://github.com/sruja-ai/staging-website)** — GitHub Pages deploy target for **staging** documentation: deploy runs when selected paths change on `main` (path-filtered push) or via manual `workflow_dispatch`. Issues may be disabled; use the main repo for bug reports.
- **[sruja-ai/prod-website](https://github.com/sruja-ai/prod-website)** — GitHub Pages deploy target for **production** documentation. Updated only via a **manual** promote workflow after staging is validated.

### How documentation is deployed

Workflows in **this** repository (`sruja-ai/sruja`) build the mdBook site (including WASM for diagrams), then push static output to the Pages repos using a GitHub App:

- **Staging** — [.github/workflows/deploy-staging.yml](../.github/workflows/deploy-staging.yml): push to `main` when changed paths include `book/`, selected crates, `scripts/`, or the workflow itself; also `workflow_dispatch`.
- **Production** — [.github/workflows/deploy-production.yml](../.github/workflows/deploy-production.yml): `workflow_dispatch` only (promote after validating staging).

### Where to file issues

Use **[sruja-ai/sruja/issues](https://github.com/sruja-ai/sruja/issues)** (or Discussions) for documentation site bugs, CLI, extension, and book content—even if you first noticed the problem on the staging or production URL. That keeps triage and fixes next to the source and deploy definitions.

---

## OSS Traction Metrics (Weekly)

Track **substance**, not extension installs or diagram exports.

### Weekly checklist

| Signal | What to record |
|--------|----------------|
| GitHub stars | Count + note any post/demo link that drove it |
| Issues / PRs | New contributors; drift/MCP questions vs "how to draw" |
| Drift screenshots | External posts or issues with **terminal/JSON drift output** |
| False positives | Issues labeled or tagged `false-positive` with **technical** repro (rule, file, why wrong) |
| Repeat scans | Same repo/user runs `drift` more than once (retention) |
| CI pilots | Teams that keep `drift --ci` or `verify-task` after trial |

### Do not optimize for

- VS Code marketplace install count alone
- Mermaid/diagram preview usage as primary KPI
- "Architecture intelligence" buzzword mentions without drift evidence

### Internal dogfood

Run on this repo:

```bash
sruja drift -r . --structural-only --advisory
```

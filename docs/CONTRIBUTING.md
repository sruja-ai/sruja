# Contributing to Sruja

Welcome! This guide will help you get started contributing to Sruja. Whether you're fixing a typo or adding a major feature, your contributions are welcome.

## 🎯 New to Contributing?

**Start here:** [First Contribution Guide](FIRST_CONTRIBUTION.md)

This step-by-step guide walks you through making your first contribution, even if you're new to the project.

## Quick Links

- 💡 **Contribution Ideas**: [What Can I Contribute?](CONTRIBUTION_IDEAS.md)
- 🐛 **Find Issues**: [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)
- 📖 **Development Guide**: [Development Practices](DEVELOPMENT.md)
- 📝 **Content Guide**: [Content Contribution Guide](CONTENT_CONTRIBUTION_GUIDE.md)
- 💬 **Get Help**: [Discord](https://discord.gg/VNrvHPV5) | [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)

## Project Overview

Sruja is a monorepo containing:
- **Language and CLI**: Go (Go 1.25+)
- **Website**: Astro-based with TypeScript/React
- **Designer**: Interactive diagram designer
- **VS Code Extension**: Language support
- **Examples**: Real-world architecture examples

## Development Setup

### Prerequisites

- **Go >= 1.25** (CI uses `1.25.4`)
- **Git**
- **Node.js 18+** (for website and TypeScript packages)

Optional:
- `golangci-lint` (installed automatically by `make lint`)
- `wasm-opt` (optimizes WASM artifacts)

### Quick Setup

```bash
# 1. Clone the repository
git clone https://github.com/sruja-ai/sruja.git
cd sruja

# 2. Install dependencies
make install

# 3. Build the CLI
make build

# 4. Verify it works
./bin/sruja --help
```

### Common Commands

```bash
# Run tests
make test

# Format code
make fmt

# Lint code
make lint

# Build everything
make build

# Try examples
./bin/sruja compile examples/example.sruja
./bin/sruja lint examples/example.sruja
```

For more details, see [Development Guide](DEVELOPMENT.md).

## Working With Examples

Example `.sruja` models live under `examples/`. After building the CLI:

```bash
# Compile an example
./bin/sruja compile examples/example.sruja

# Lint an example
./bin/sruja lint examples/example.sruja

# Export to different formats
./bin/sruja export json examples/example.sruja
./bin/sruja export markdown examples/example.sruja
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

## Ways to Contribute

### No Code Required

- 📝 **Documentation**: Fix typos, improve clarity, add examples
- 🐛 **Testing**: Test features and report bugs
- 💡 **Examples**: Add example architectures to `examples/` directory
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

If there aren't many issues yet, check **[Contribution Ideas](CONTRIBUTION_IDEAS.md)** for specific tasks you can work on. Common tasks that don't need issues:

- Fix typos in documentation
- Add examples to `examples/` directory
- Improve error messages
- Add test cases
- Write tutorials or blog posts

You can start working on these right away! Open a draft PR to show what you're working on.

## Getting Help

- 💬 **Discord**: https://discord.gg/VNrvHPV5
- 💬 **GitHub Discussions**: Ask questions and share ideas
- 📝 **GitHub Issues**: Report bugs or request features
- 💬 **PR Comments**: Ask for help on your pull request

## Principles

- **Keep PRs small and focused**: Easier to review and merge
- **Test your changes**: Run `make test` before submitting
- **Follow conventions**: Use Conventional Commits, run `make fmt`
- **Start small**: You can always contribute more later!
- **Ask questions**: We're here to help!

## Get in Touch

- GitHub Discussions
- Discord: https://discord.gg/VNrvHPV5

## Contributing Content

To add content to the Sruja website (courses, tutorials, blog posts, etc.), see:

**📖 [Content Contribution Guide](CONTENT_CONTRIBUTION_GUIDE.md)**

This guide covers:
- Creating courses, tutorials, blogs, and docs
- Content structure and best practices
- Validation and workflow
- Troubleshooting

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

- **Architecture**: See [ARCHITECTURE.md](ARCHITECTURE.md) for code organization
- **Design Philosophy**: See [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) for language design principles
- **Language Spec**: See [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) for complete DSL reference

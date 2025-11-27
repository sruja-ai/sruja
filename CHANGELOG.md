# Changelog

All notable changes to Sruja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added
- Initial release of Sruja language
- Core DSL: workspace, model, system, container, component, relations
- Requirements and ADRs as first-class language constructs
- Lexer and Parser implementation
- D2 Export support
- Validation engine with 4 core rules:
  - Unique ID validation
  - Valid reference checking
  - Cycle detection
  - Orphan detection
- CLI tools:
  - `sruja export d2` - Export to D2
  - `sruja lint` - Validate code
  - `sruja fmt` - Auto-format code
  - `sruja tree` - Visualize hierarchy
  - `sruja list` - List elements
  - `sruja explain` - Explain elements
- GitHub Actions CI/CD workflows
- Cross-platform release binaries (Linux, macOS, Windows)

### Documentation
- README with quickstart
- Roadmap to v1.0.0
- Example `.sruja` files

### Removed
- Legacy commands: `compile`, `notebook`, `mcp`, `install`, `update`
- Unused packages: `pkg/compiler`, `pkg/notebook`, `pkg/mcp`, `pkg/kernel`, `pkg/extensions`
- Node.js dependencies and VS Code extension (moved to separate repo)

[Unreleased]: https://github.com/sruja-ai/sruja/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sruja-ai/sruja/releases/tag/v0.1.0

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
- Mermaid C4 compiler backend
- Validation engine with 4 core rules:
  - Unique ID validation
  - Valid reference checking
  - Cycle detection
  - Orphan detection
- CLI tools:
  - `sruja compile` - Compile to Mermaid
  - `sruja lint` - Validate code
  - `sruja fmt` - Auto-format code
  - `sruja notebook` - Markdown integration
  - `sruja mcp` - MCP server for AI
  - `sruja install/update` - Package management
- Git-based extension system (manifest only, loader TBD)
- Model Context Protocol (MCP) v2025-06-18 integration
- VS Code extension with syntax highlighting and snippets
- GitHub Actions CI/CD workflows
- Cross-platform release binaries (Linux, macOS, Windows)

### Documentation
- README with quickstart
- Roadmap to v1.0.0
- Example `.sruja` files

[Unreleased]: https://github.com/sruja-ai/sruja/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sruja-ai/sruja/releases/tag/v0.1.0

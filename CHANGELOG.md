# Changelog

All notable changes to Sruja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.0.0 (2025-11-28)


### Features

* add docs site ([d6c9e89](https://github.com/sruja-ai/sruja/commit/d6c9e89832ef42649cc547ab7d14efbb849eb4bc))
* add google tag manager ([6d84ea7](https://github.com/sruja-ai/sruja/commit/6d84ea7bfcc1712f3f9842053f76fc64c62a5b7a))
* scratch work ([79c57d3](https://github.com/sruja-ai/sruja/commit/79c57d38e3e5cf1b9bc9a1c2fe99eda8b59697f1))
* sruja language code ([be3a0e7](https://github.com/sruja-ai/sruja/commit/be3a0e756cd24d7415de31decb02f8288b8b343a))


### Bug Fixes

* github pages ([4aa6c21](https://github.com/sruja-ai/sruja/commit/4aa6c217e241cb6146b7760e09fdf2138d2e2f94))
* github pages ([58e6095](https://github.com/sruja-ai/sruja/commit/58e6095136bf3f7acfa690e86292f595fec0858e))
* github pages ([a570af1](https://github.com/sruja-ai/sruja/commit/a570af18a2a2a204199944ff6b303cfbedd73e2a))
* github pages ([dbb619d](https://github.com/sruja-ai/sruja/commit/dbb619d4eb0be7e51d86bf3c1692f4c2529be703))
* go version in ci ([84e2407](https://github.com/sruja-ai/sruja/commit/84e2407eaf74d65927e54ca3ef1e7b4571e196f9))
* go version in ci ([ecd5f9b](https://github.com/sruja-ai/sruja/commit/ecd5f9b95d5572086a33fcebe428ac76be768393))
* sruja site ([223bf4e](https://github.com/sruja-ai/sruja/commit/223bf4eb11f98d9e139046092a09e404a50a8f1c))

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

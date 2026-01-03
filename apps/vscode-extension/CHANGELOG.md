# Changelog

All notable changes to the Sruja DSL Language Support extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/sruja-ai/sruja/compare/sruja-language-support-v0.1.1...sruja-language-support-v0.2.0) (2026-01-03)


### Features

* Add LikeC4Canvas implementation and supporting files ([7310084](https://github.com/sruja-ai/sruja/commit/73100849d304cca1e63eed26b53e4a5d660c0ef4))
* diagram improvements, theme fixes, and infrastructure upgrades ([182583a](https://github.com/sruja-ai/sruja/commit/182583ab360b02f04f325551aa7bec2c7cb7de16))
* **playground:** use shared UI components and inline examples list ([0654c2c](https://github.com/sruja-ai/sruja/commit/0654c2cdb3b1d20b8b57e2d3d1ceb97698160134))


### Bug Fixes

* add .vscodeignore to exclude parent directories from VSIX ([1201bf2](https://github.com/sruja-ai/sruja/commit/1201bf22b3f71f9449bc20322751c78a0f5fc5f5))
* build extension before running tests ([44bd3a7](https://github.com/sruja-ai/sruja/commit/44bd3a718f1c4593445dc65dd09c343d8c5d8ef1))
* correct extension development path in test runner ([a7361d2](https://github.com/sruja-ai/sruja/commit/a7361d21d95375074c806181e516e10ea8ef1146))
* correct publisher ID from sruja-ai to srujaai ([d6666c9](https://github.com/sruja-ai/sruja/commit/d6666c9f515fd43bfee1c27420ed0401d680cbf0))
* eslint ([62d228b](https://github.com/sruja-ai/sruja/commit/62d228b03456062e88b103badf2ff4692cd4df89))
* LikeC4 diagram rendering and interactivity improvements ([2af1b8e](https://github.com/sruja-ai/sruja/commit/2af1b8e56e48332529bcea70894e94b2138aa3c7))
* open workspace folder for extension tests ([1d85eed](https://github.com/sruja-ai/sruja/commit/1d85eedeaea8fe2780d9f3067b97aeb246e847cd))
* remove files property, use .vscodeignore only ([ba46050](https://github.com/sruja-ai/sruja/commit/ba46050ee461e8439d72de76db18a3a75b0647bb))
* strengthen .vscodeignore to prevent parent directory inclusion ([13b10c1](https://github.com/sruja-ai/sruja/commit/13b10c1a8cd8e7526e116fa22e96808b1822e02a))
* update extension ID from sruja-ai.sruja to srujaai.sruja ([adfcdca](https://github.com/sruja-ai/sruja/commit/adfcdca4a4e6e970d406bccd8bd95b96da03dfa9))
* use existing example file in semantic tokens test ([13cfb2b](https://github.com/sruja-ai/sruja/commit/13cfb2b5ad08bc37d2b282c99efdeddf92fe1e2d))
* **vscode-extension:** fix invalid property access in staging tests ([a1f504e](https://github.com/sruja-ai/sruja/commit/a1f504ef76a95751b565db292c63ea5fd91b95bd))
* **vscode-extension:** fix test paths and glob version mismatch ([bf1395c](https://github.com/sruja-ai/sruja/commit/bf1395cec66c6113e8448a30dd3e0b41bfe857ef))

## [Unreleased]

### Added
- **Performance Optimizations**
  - Debouncing for diagnostics (configurable, default 300ms delay)
  - Document caching system (configurable TTL, default 5 seconds) to reduce WASM calls
  - Cache invalidation on document close and configuration changes
  - Configurable performance settings in VS Code preferences
  
- **Advanced LSP Features**
  - Inlay Hints provider - shows element types inline (e.g., `: system`, `: container`)
  - Code Lenses provider - shows reference counts above symbols
  - Signature Help provider - parameter hints during typing
  
- **User Experience**
  - Welcome message on first activation with quick start options
  - Progress indicators for WASM loading
  - Enhanced status bar - shows error/warning counts instead of just "Ready"
  - Improved error messages with actionable guidance
  - Sample file creation on "Get Started"
  
- **Testing**
  - Comprehensive LSP provider tests
  - Performance tests for caching and debouncing
  - Integration tests for large files

### Changed
- Diagnostics now use debouncing to improve performance (configurable)
- Status bar shows diagnostic counts instead of just "Ready"
- Improved error handling and logging with better user feedback
- WASM initialization shows progress notification
- Cache settings are now configurable via VS Code settings

### Configuration
- New settings under `sruja.performance`:
  - `enableCaching` (default: true) - Enable/disable document caching
  - `cacheTTL` (default: 5000) - Cache time-to-live in milliseconds
  - `diagnosticsDebounce` (default: 300) - Diagnostics debounce delay in milliseconds

### Performance
- Reduced WASM API calls through intelligent caching
- Debounced diagnostics updates (300ms) to prevent excessive parsing
- Cache TTL of 5 seconds balances freshness with performance

## [0.1.1] - 2024-01-XX

### Added
- Initial release
- Syntax highlighting for `.sruja` files
- WASM-based Language Server Protocol (LSP) support
- Diagnostics (error detection and validation)
- Hover information
- Auto-completion
- Go-to-definition
- Find references
- Rename symbol
- Document formatting
- Document symbols (Outline view)
- Workspace symbols
- Code actions
- Document links
- Folding ranges
- Semantic tokens
- Architecture preview (Markdown export)
- Comprehensive snippets
- Debug command for WASM LSP troubleshooting

### Features
- No CLI dependency - works entirely with WASM
- Platform-agnostic (Windows, macOS, Linux)
- Real-time error detection
- Comprehensive IntelliSense support

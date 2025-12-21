# Changelog

All notable changes to the Sruja DSL Language Support extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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


# Sruja Roadmap

## Current Version: v0.1.0 (Alpha)

The initial release provides core functionality for architecture-as-code with basic validation and tooling.

## Path to v1.0.0

### Language Refinements
- [ ] **Strict C4 Model Alignment**: Ensure all C4 elements (Context, Container, Component, Code) are first-class citizens.
- [ ] **Deployment Views**: Add support for Deployment Nodes and Infrastructure.
- [ ] **Dynamic Views**: Add support for Dynamic diagrams (runtime flows).
- [ ] Formal grammar specification (EBNF)
- [ ] Import statement implementation
- [ ] Metadata/tags support for elements

### Compiler Enhancements
- [ ] PlantUML backend
- [ ] D2 backend
- [ ] Structurizr DSL export
- [ ] JSON/YAML export for programmatic access
- [ ] Diagram customization (themes, colors, layouts)

### Validation Engine
- [ ] Dependency depth analysis
- [ ] Naming convention rules
- [ ] Custom rule templates
- [ ] Severity levels (error, warning, info)
- [ ] Rule configuration via manifest

### Extension System
- [ ] Extension loader implementation (currently only manifest defined)
- [ ] Plugin API documentation
- [ ] Example extension packages
- [ ] Extension marketplace/registry
- [ ] Sandboxing for untrusted extensions

### Developer Experience
- [ ] Language Server Protocol (LSP)
- [ ] VS Code: Go to definition, find references
- [ ] VS Code: Inline diagnostics
- [ ] VS Code: Diagram preview
- [ ] CLI autocomplete (bash, zsh, fish)
- [ ] Better error messages with suggestions

### Documentation
- [ ] Comprehensive language reference
- [ ] Extension development guide
- [ ] Architecture patterns library
- [ ] Video tutorials
- [ ] API documentation

### Testing & Quality
- [ ] 80%+ test coverage
- [ ] Fuzz testing
- [ ] Performance benchmarks
- [ ] Example real-world architectures
- [ ] Migration guides

### Ecosystem
- [ ] Official extension repository
- [ ] Homebrew formula
- [ ] Docker image
- [ ] npm wrapper for JS/TS projects
- [ ] GitHub Action
- [ ] Pre-commit hooks

### MCP Enhancements
- [ ] Prompts for common architecture patterns
- [ ] Sampling support for large diagrams
- [ ] Extension tool exposure via MCP
- [ ] Collaborative editing support

## Version Strategy

- **v0.1.x**: Alpha releases, core functionality, breaking changes expected
- **v0.2.x**: Beta releases, stable language spec, extensions working
- **v0.9.x**: Release candidates, feature complete, polish
- **v1.0.0**: Production ready, stable API, comprehensive docs

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to help build Sruja.

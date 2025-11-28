# Sruja Roadmap

## Current Version: v0.1.0 (Alpha)

The initial release provides core functionality for architecture-as-code with basic validation and D2 export.

## Path to v1.0.0

### Language Refinements
- [ ] **Strict C4 Model Alignment**: Ensure all C4 elements (Context, Container, Component, Code) are first-class citizens.
- [ ] **Deployment Views**: Add support for Deployment Nodes and Infrastructure.
- [ ] Formal grammar specification (EBNF)
- [ ] Import statement implementation
- [ ] Metadata/tags support for elements

### Compiler Enhancements
- [ ] D2 backend improvements (themes, classes)
- [ ] JSON/YAML export for programmatic access

### Validation Engine
- [ ] Dependency depth analysis
- [ ] Naming convention rules
- [ ] Custom rule templates
- [ ] Severity levels (error, warning, info)

### Developer Experience
- [ ] CLI autocomplete (bash, zsh, fish)
- [ ] Better error messages with suggestions
- [ ] VS Code Extension (Syntax Highlighting)

### Documentation
- [ ] Comprehensive language reference
- [ ] Architecture patterns library
- [ ] API documentation

### Testing & Quality
- [ ] 80%+ test coverage
- [ ] Fuzz testing
- [ ] Performance benchmarks
- [ ] Example real-world architectures

## Version Strategy

- **v0.1.x**: Alpha releases, core functionality, breaking changes expected
- **v0.2.x**: Beta releases, stable language spec
- **v1.0.0**: Production ready, stable API, comprehensive docs

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to help build Sruja.

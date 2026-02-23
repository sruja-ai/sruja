# Sruja Architecture Documentation

This directory contains architecture-as-code files that describe Sruja's own architecture using the Sruja DSL.

## Purpose

These `.sruja` files serve as:

- **Living documentation** - Architecture that stays in sync with code
- **Visual diagrams** - Export to Mermaid/JSON for viewing
- **Reference models** - Examples of using Sruja to document real systems
- **Architecture decisions** - Document the structure and relationships

## Files

### `sruja-platform.sruja`

Complete architecture of the Sruja platform including:

- CLI and core engine
- Frontend applications
- Package structure
- WASM integration
- LSP server

### `sruja-development-workflow.sruja`

Development and build processes:

- CI/CD pipelines
- Testing infrastructure
- Build systems
- Deployment processes

### `sruja-data-flow.sruja`

Data flow and processing:

- DSL parsing pipeline
- Validation flow
- Export/import processes
- WASM communication

## Usage

### Export to JSON

```bash
sruja export json docs/architecture/sruja-platform.sruja
```

### Validate

```bash
sruja lint docs/architecture/sruja-platform.sruja
```

### Generate Views

```bash
sruja export json docs/architecture/sruja-platform.sruja --views
```

## Maintenance

- **Keep in sync**: Update these files when architecture changes
- **Validate regularly**: Run `sruja lint` on these files
- **Review in PRs**: Include architecture changes in PR reviews
- **Reference in docs**: Link to these files from `ARCHITECTURE.md`

## Relationship to Other Docs

- **ADRs**: Architecture Decision Records explain _why_ decisions were made
- **These files**: Show _what_ the architecture looks like

## Best Practices

1. **Keep it current**: Update when code structure changes
2. **Be comprehensive**: Include all major components
3. **Use relationships**: Show how components interact
4. **Add metadata**: Include descriptions, technologies, tags
5. **Create views**: Define views for different perspectives
6. **Use stdlib** (when available): Import standard elements from `sruja.ai/stdlib`

# Go Implementation: DSL ↔ JSON

## Responsibility

Go handles all DSL ↔ JSON transformations:

- ✅ DSL → AST (existing parser)
- ✅ AST → JSON (new exporter)
- ✅ JSON → AST (new converter)  
- ✅ AST → DSL (existing printer)
- ✅ CLI commands
- ✅ File operations

## Tasks

See individual task files for detailed implementation:

1. [JSON Exporter](task-1.1-json-exporter.md) - AST → JSON
2. [JSON to AST Converter](task-1.2-json-to-ast.md) - JSON → AST
3. [CLI Commands](task-1.3-cli-commands.md) - User-facing commands
4. [Modularization](task-1.4-modularization.md) - Split large files
5. [Change Commands](task-1.5-change-commands.md) - Change tracking, snapshots, diffs
6. ~~[Proposal Commands](task-1.6-proposal-commands.md)~~ - **REMOVED**: Use ADRs + external systems (GitHub/Cloud Studio) instead. See [Simplified Change Workflow](SIMPLIFIED_CHANGE_WORKFLOW.md)

## JSON Schema

The JSON format is the **contract** with TypeScript. See [JSON Schema](json-schema.md) for complete specification.

## Key Design Decisions

1. **Self-Contained JSON**: No imports in the structure. File information is preserved via metadata annotations. See [File Metadata Design](file-metadata-design.md) for details.

2. **Qualified Names**: All element IDs use qualified names (e.g., `SystemName.ContainerName.ComponentName`) for clarity and stability. Qualified names follow **scoping rules** in DSL (simple names in scope, qualified when outside) and are fully qualified in JSON. See [Qualified Names Specification](qualified-names.md) and [DSL Qualified Names](dsl-qualified-names.md) for complete naming rules.

3. **Relation Flattening**: Relations can be defined at multiple scopes (container, system, architecture) in DSL but are flattened into a single array in JSON. See [Relation Flattening](relation-flattening.md) for details.

4. **Architecture Model**: One architecture block per file (enforced). Each architecture file generates one JSON. Shared elements (from files without architecture blocks) are imported. Multiple architectures = multiple files (workspace concept). See [Architecture Model](architecture-model.md) and [Architecture Semantics](architecture-semantics.md) for details.

5. **Simplified Change Workflow**: Use ADRs for decision tracking, Studio for creating changes visually, and external systems (GitHub PRs, Cloud Studio) for discussions and reviews. See [Simplified Change Workflow](SIMPLIFIED_CHANGE_WORKFLOW.md) for details.

## Key Principles

1. **Round-trip Preservation**: DSL → JSON → DSL must preserve all information
2. **Import Preservation**: Imports must appear first in generated DSL
3. **Complete Coverage**: All AST types must be exportable to JSON
4. **Test Coverage**: 90%+ test coverage required

# Sruja 2.0 - Open Specifications Summary

## Overview

This document provides a comprehensive summary of the Open Specifications (OpenSpec) created for Sruja 2.0. OpenSpec is a lightweight spec-driven framework that aligns engineering work with documented requirements and enables reviewing intent, not just code.

## What Was Created

A complete OpenSpec repository structure with 7 capability specifications, covering all major components of Sruja 2.0:

```
openspec/
├── README.md                           # Complete guide to using OpenSpec for Sruja
├── SUMMARY.md                          # This file
└── specs/
    ├── dsl-parser/
    │   └── spec.md                     # DSL parsing requirements (152 lines)
    ├── validator/
    │   └── spec.md                     # Validation requirements (226 lines)
    ├── diff-engine/
    │   └── spec.md                     # Diff and comparison requirements (323 lines)
    ├── export-engine/
    │   └── spec.md                     # Export and diagramming requirements (376 lines)
    ├── traceability/
    │   └── spec.md                     # Dependency tracing requirements (108+ lines)
    ├── cli-interface/
    │   └── spec.md                     # CLI command requirements (484 lines)
    └── github-integration/
        └── spec.md                     # CI/CD integration requirements (388 lines)
```

**Total Lines of Specifications:** 2,057 lines across 7 specifications

## Specifications by Capability

### 1. DSL Parser (`specs/dsl-parser/spec.md`)

**Purpose:** Parse and convert Sruja's Domain Specific Language (DSL) into a structured model that can be validated, analyzed, and exported.

**Key Requirements:**
- Parse DSL file into Model objects
- Handle syntax errors with clear messages
- Support traceability links (ADRs, issues, PRs)
- Parse metadata (version, author, timestamps)
- Handle comments and whitespace
- Support quoted and unquoted strings
- Validate ID format

**Scenario Count:** 16 scenarios covering parsing, error handling, metadata, and string handling

**Use When:** Implementing the parser, adding new DSL syntax, improving error messages

---

### 2. Validator (`specs/validator/spec.md`)

**Purpose:** Validate architecture models parsed from the DSL to ensure structural integrity, reference consistency, and adherence to architectural best practices.

**Key Requirements:**
- Validate unique element IDs
- Check relationship references
- Detect orphaned elements
- Validate system structure
- Validate relationship direction
- Validate protocol consistency
- Validate container technology
- Support strict validation mode
- Collect validation statistics
- Provide error locations

**Scenario Count:** 21 scenarios covering ID validation, references, orphans, structure, and statistics

**Use When:** Adding validation rules, improving error reporting, implementing strict mode

---

### 3. Diff Engine (`specs/diff-engine/spec.md`)

**Purpose:** Compare two architecture versions to identify changes, track evolution of the system over time, and generate detailed diff reports.

**Key Requirements:**
- Compare architecture versions
- Detect added, deleted, and modified elements
- Detect added and removed relationships
- Detect added/removed links
- Detect breaking changes
- Generate diff in multiple formats (text, JSON, Markdown, HTML)
- Calculate diff statistics
- Preserve element and relationship details
- Handle renamed elements
- Support partial diffs

**Scenario Count:** 30 scenarios covering all change types, formats, and filtering options

**Use When:** Implementing diff algorithms, adding export formats, detecting breaking changes

---

### 4. Export Engine (`specs/export-engine/spec.md`)

**Purpose:** Generate architecture diagrams and documentation in various formats from parsed architecture models.

**Key Requirements:**
- Export to Mermaid format
- Export to Markdown format
- Export to JSON format
- Export to SVG format
- Export to PNG format
- Export to PlantUML format
- Support multiple architecture views (context, containers, components, deployed)
- Support theming (default, dark, forest, neutral)
- Filter elements by inclusion/exclusion lists
- Generate diagram metadata
- Handle large models
- Validate export output
- Write output to file

**Scenario Count:** 33 scenarios covering all export formats, views, themes, and filtering

**Use When:** Adding new export formats, implementing views, theming diagrams

---

### 5. Traceability (`specs/traceability/spec.md`)

**Purpose:** Analyze and trace dependencies between architectural elements to understand system connectivity, identify critical paths, and assess the impact of changes.

**Key Requirements:**
- Trace upstream dependencies
- Trace downstream dependencies
- Trace in both directions
- Detect circular dependencies (cycles)

**Scenario Count:** 11 scenarios covering tracing, depth limits, and cycle detection

**Use When:** Implementing dependency analysis, cycle detection, impact assessment

---

### 6. CLI Interface (`specs/cli-interface/spec.md`)

**Purpose:** Provide a command-line interface that enables developers to interact with Sruja's core functionality through terminal commands.

**Key Requirements:**
- Parse global flags (verbose, quiet, config, no-color)
- Initialize new Sruja project (`init`)
- Validate architecture files (`validate`)
- Compare architecture versions (`diff`)
- Export architecture to various formats (`export`)
- Check for breaking changes (`check`)
- Trace element dependencies (`trace`)
- Generate documentation (`docs`)
- Display version information (`version`)
- Provide help information (`help`)
- Handle errors gracefully
- Support exit codes
- Support auto-completion
- Respect environment variables

**Scenario Count:** 45 scenarios covering all commands, flags, error handling, and integration

**Use When:** Implementing CLI commands, adding flags, improving error handling

---

### 7. GitHub Integration (`specs/github-integration/spec.md`)

**Purpose:** Integrate Sruja with GitHub's CI/CD platform to automate architecture validation, detect breaking changes in pull requests, and provide architectural feedback directly in the review process.

**Key Requirements:**
- Validate architecture in GitHub Actions
- Check for breaking changes in PRs
- Generate and comment PR diffs
- Install Sruja via package managers (Homebrew, NPM, cargo)
- Support workflow configuration
- Handle workflow errors gracefully
- Support manual workflow dispatch
- Support workflow reuse
- Support matrix builds
- Generate workflow artifacts

**Scenario Count:** 36 scenarios covering CI/CD, PR integration, and workflow management

**Use When:** Setting up CI/CD, creating workflows, adding GitHub Actions features

## Relationship to SRUJA-2.0-STRATEGY.md

The OpenSpec specifications are directly derived from the strategy document:

| Strategy Section | OpenSpec Specification | Requirements Covered |
|-----------------|------------------------|---------------------|
| DSL Specification | `specs/dsl-parser/spec.md` | Grammar, example, key features |
| Validator Core | `specs/validator/spec.md` | Validation rules, error handling |
| Diff Engine | `specs/diff-engine/spec.md` | Comparison algorithms, change tracking |
| Exporters | `specs/export-engine/spec.md` | Mermaid, Markdown, JSON exporters |
| Traceability Engine | `specs/traceability/spec.md` | Dependency analysis, impact tracking |
| CLI Specification | `specs/cli-interface/spec.md` | Commands, flags, exit codes |
| GitHub Integration | `specs/github-integration/spec.md` | CI/CD workflows, PR checks |
| Core Principles | All specs | DSL as source of truth, fail fast, trace everything |

## How to Use These Specifications

### 1. Before Implementing

Read the relevant specification to understand requirements:

```bash
# Read a specific spec
cat openspec/specs/dsl-parser/spec.md

# Search for specific requirements
grep -A 10 "Parse DSL file" openspec/specs/dsl-parser/spec.md
```

### 2. Create Change Proposals

Use OpenSpec CLI to create proposals for significant changes:

```bash
# Install OpenSpec CLI
npm install -g @fission-ai/openspec@latest

# Create a proposal
openspec:proposal "Add JSON Schema export for DSL"

# This creates:
# openspec/changes/add-json-schema-export/
#   ├── proposal.md
#   ├── design.md
#   ├── tasks.md
#   └── specs/
#       └── export-engine/
#           └── spec.md
```

### 3. Implement and Test

Write tests based on specification scenarios:

```rust
// Scenario: Parse simple architecture file
#[test]
fn parse_simple_architecture_file() {
    // GIVEN a DSL file exists at "architecture.sruja"
    let content = r#"
        person customer {
            name: "Customer"
            description: "External customer"
        }
    "#;
    
    // WHEN the parser processes the file
    let result = parse(content);
    
    // THEN a Model object is created successfully
    assert!(result.is_ok());
    
    // AND the Model contains the Person element
    // AND the element has the correct ID, name, and description
    let model = result.unwrap();
    assert_eq!(model.elements.len(), 1);
    assert_eq!(model.elements[0].id, "customer");
    assert_eq!(model.elements[0].name, "Customer");
}
```

### 4. Review and Update

As you implement, update specifications with new requirements:

```markdown
### Requirement: Parse new DSL feature

The system SHALL parse the new feature added in v2.1.

#### Scenario: Parse feature with configuration
- GIVEN a DSL file contains the new feature
- WHEN the parser processes the file
- THEN the feature is parsed correctly
- AND configuration options are preserved
```

## Integration with AI Tools

### Tools with Native OpenSpec Support

These tools have native OpenSpec integration:
- Claude Code
- Cursor
- GitHub Copilot
- Windsurf
- And many more...

**Usage:**
```
Read openspec/specs/export-engine/spec.md and implement the SVG export feature
```

### Tools Without Native Support

When working with AI assistants that don't have native OpenSpec support:

```
I'm implementing Sruja 2.0. Here's the specification for the feature I'm working on:

[Paste the relevant spec section]

Please review this specification and help me implement it. Focus on the scenarios listed under each requirement.
```

## Specification Statistics

| Specification | Lines | Requirements | Scenarios | Complexity |
|---------------|--------|--------------|------------|-------------|
| DSL Parser | 152 | 7 | 16 | Medium |
| Validator | 226 | 10 | 21 | Medium |
| Diff Engine | 323 | 13 | 30 | High |
| Export Engine | 376 | 12 | 33 | High |
| Traceability | 108+ | 4 | 11 | Low |
| CLI Interface | 484 | 14 | 45 | High |
| GitHub Integration | 388 | 10 | 36 | Medium |
| **Total** | **2,057** | **70** | **192** | - |

## Next Steps

### For Developers

1. **Read the relevant spec** before implementing any feature
2. **Create tests** for each scenario in the specification
3. **Update specs** when adding new features or changing behavior
4. **Use change proposals** for significant modifications

### For Maintainers

1. **Keep specs in sync** with the strategy document
2. **Review and merge** spec updates with code changes
3. **Generate documentation** from specifications
4. **Track spec coverage** to ensure all scenarios are tested

### For AI/ML Integration

1. **Provide specs as context** when working with AI tools
2. **Use scenarios as test cases** for validation
3. **Create change proposals** with AI assistance
4. **Generate code** directly from specifications

## Best Practices

1. **Spec-First Development**: Always read specs before coding
2. **Test-Driven**: Write tests for each scenario
3. **Iterative**: Update specs as you learn
4. **Collaborative**: Review proposals before implementing
5. **Documentation-First**: Keep specs and docs in sync

## Troubleshooting

### Specification Outdated

If a specification doesn't match the code:
1. Check if a change proposal exists
2. Review recent commits
3. Update the specification
4. Create a change proposal if needed

### Missing Scenario

If you need a scenario that doesn't exist:
1. Add it to the relevant specification
2. Write a test for it
3. Implement the behavior
4. Update documentation

### Conflicting Requirements

If requirements conflict:
1. Check the strategy document for guidance
2. Create a proposal to resolve
3. Get team review
4. Update the specification

## Resources

- [OpenSpec Documentation](https://openspec.dev)
- [OpenSpec GitHub](https://github.com/Fission-AI/OpenSpec/)
- [Sruja 2.0 Strategy](../SRUJA-2.0-STRATEGY.md)
- [OpenSpec README](./README.md)

## License

These specifications are part of the Sruja project and follow the same MIT license as the main codebase.

---

**Last Updated:** 2024-01-20  
**Version:** 2.0.0  
**Total Specifications:** 7 capabilities  
**Total Requirements:** 70 requirements  
**Total Scenarios:** 192 testable scenarios
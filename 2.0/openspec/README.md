# Sruja 2.0 - Open Specifications

This directory contains Open Specifications (OpenSpec) for Sruja 2.0, organized by capability. OpenSpec is a lightweight spec-driven framework that aligns engineering work with documented requirements and enables reviewing intent, not just code.

## What is OpenSpec?

OpenSpec is a tool and format for managing technical specifications as living documentation. Unlike traditional requirements documents that become stale, OpenSpec specs:

- **Live alongside code** - Specs are checked into the repository
- **Drive development** - Requirements guide implementation
- **Support change reviews** - Proposals and diffs show how requirements change
- **Enable collaboration** - Teams review specs before writing code
- **Integrate with AI tools** - LLMs and agents use specs as context

## Directory Structure

```
openspec/
├── README.md                    # This file
├── specs/                       # Specifications organized by capability
│   ├── dsl-parser/
│   │   └── spec.md             # DSL parsing requirements
│   ├── validator/
│   │   └── spec.md             # Validation requirements
│   ├── diff-engine/
│   │   └── spec.md             # Diff and comparison requirements
│   ├── export-engine/
│   │   └── spec.md             # Export and diagramming requirements
│   ├── traceability/
│   │   └── spec.md             # Dependency tracing requirements
│   ├── cli-interface/
│   │   └── spec.md             # CLI command requirements
│   └── github-integration/
│       └── spec.md             # CI/CD integration requirements
└── changes/                     # Change proposals (created as needed)
    └── example-change/
        ├── proposal.md           # Description of the change
        ├── design.md             # Technical design decisions
        ├── tasks.md              # Implementation tasks
        └── specs/                # Spec deltas
            └── capability-name/
                └── spec.md       # Changed requirements
```

## Specifications

Each specification follows the OpenSpec format with **Purpose**, **Requirements**, and **Scenarios**.

### DSL Parser (`specs/dsl-parser/spec.md`)
Defines requirements for parsing Sruja's Domain Specific Language into structured models.

**Key Requirements:**
- Parse DSL files into Model objects
- Handle syntax errors with clear messages
- Support traceability links (ADRs, issues, PRs)
- Parse metadata (version, author, timestamps)
- Handle comments and whitespace
- Validate ID formats

**Use when:** Implementing the parser, adding new DSL syntax, improving error messages

### Validator (`specs/validator/spec.md`)
Defines requirements for validating architecture models.

**Key Requirements:**
- Validate unique element IDs
- Check relationship references
- Detect orphaned elements
- Validate system structure
- Support strict validation mode
- Provide error locations and statistics

**Use when:** Adding validation rules, improving error reporting, implementing strict mode

### Diff Engine (`specs/diff-engine/spec.md`)
Defines requirements for comparing architecture versions.

**Key Requirements:**
- Compare two architecture versions
- Detect added, deleted, and modified elements
- Detect added and removed relationships
- Identify breaking changes
- Generate diffs in multiple formats (text, JSON, Markdown, HTML)
- Support partial diffs and filtering

**Use when:** Implementing diff algorithms, adding export formats, detecting breaking changes

### Export Engine (`specs/export-engine/spec.md`)
Defines requirements for generating diagrams and documentation.

**Key Requirements:**
- Export to Mermaid, PlantUML, JSON, Markdown, SVG, PNG
- Support multiple architecture views (context, containers, components, deployed)
- Apply themes (default, dark, forest, neutral)
- Filter elements by inclusion/exclusion lists
- Handle large models efficiently

**Use when:** Adding new export formats, implementing views, theming diagrams

### Traceability (`specs/traceability/spec.md`)
Defines requirements for analyzing dependencies between elements.

**Key Requirements:**
- Trace upstream and downstream dependencies
- Trace in both directions
- Detect circular dependencies (cycles)
- Find all paths between elements
- Support depth limits

**Use when:** Implementing dependency analysis, cycle detection, impact assessment

### CLI Interface (`specs/cli-interface/spec.md`)
Defines requirements for the command-line interface.

**Key Requirements:**
- Support global flags (verbose, quiet, config, no-color)
- Commands: init, validate, diff, export, check, trace, docs, version, help
- Handle errors gracefully with clear messages
- Use appropriate exit codes
- Support shell completion (bash, zsh, fish)
- Respect environment variables

**Use when:** Implementing CLI commands, adding flags, improving error handling

### GitHub Integration (`specs/github-integration/spec.md`)
Defines requirements for CI/CD integration with GitHub.

**Key Requirements:**
- Validate architecture in GitHub Actions
- Check for breaking changes in PRs
- Generate and comment PR diffs
- Install via package managers (Homebrew, NPM, cargo)
- Support workflow configuration
- Handle workflow errors gracefully
- Support manual workflow dispatch

**Use when:** Setting up CI/CD, creating workflows, adding GitHub Actions features

## How to Use These Specifications

### 1. Read and Understand
Before implementing a feature, read the relevant specification(s) to understand requirements.

```bash
# Read the DSL Parser spec
cat openspec/specs/dsl-parser/spec.md

# Read multiple specs at once
cat openspec/specs/*/spec.md | less
```

### 2. Create a Change Proposal
When planning a significant change, create a proposal using OpenSpec format.

```bash
# Use OpenSpec CLI to create a proposal
npx openspec:proposal "Add JSON Schema export for DSL"
```

This creates:
```
openspec/changes/add-json-schema-export/
├── proposal.md    # What you're changing and why
├── design.md      # Technical approach
├── tasks.md       # Breakdown of implementation tasks
└── specs/
    └── export-engine/
        └── spec.md   # New requirements
```

### 3. Review and Refine
Share the proposal with your team for review. Iterate on the plan before writing code.

### 4. Implement
Use the proposal's tasks as your implementation guide. Update specs as you learn.

### 5. Update Specs
As you implement, update the relevant specification files with new requirements or scenarios.

### 6. Test Against Specs
Write tests that verify each scenario in the specification passes.

```rust
// Example test based on spec scenario
#[test]
fn parse_simple_architecture_file() {
    // GIVEN a DSL file exists at "architecture.sruja"
    // AND the file contains a valid Person element definition
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
    let model = result.unwrap();
    
    // AND the Model contains the Person element
    assert_eq!(model.elements.len(), 1);
    let element = &model.elements[0];
    
    // AND the element has the correct ID, name, and description
    assert_eq!(element.id, "customer");
    assert_eq!(element.name, "Customer");
}
```

## Relationship to SRUJA-2.0-STRATEGY.md

These OpenSpec specifications are derived from the [SRUJA-2.0-STRATEGY.md](../SRUJA-2.0-STRATEGY.md) document:

| Strategy Section | OpenSpec Specification |
|-----------------|------------------------|
| DSL Specification | `specs/dsl-parser/spec.md` |
| Validator Core | `specs/validator/spec.md` |
| Diff Engine | `specs/diff-engine/spec.md` |
| Exporters | `specs/export-engine/spec.md` |
| Traceability Engine | `specs/traceability/spec.md` |
| CLI Specification | `specs/cli-interface/spec.md` |
| GitHub Integration | `specs/github-integration/spec.md` |
| Core Principles (DSL is Source of Truth) | All specs |
| Fail Fast | `specs/validator/spec.md`, `specs/github-integration/spec.md` |

## Writing Specifications

When writing new specifications, follow this format:

```markdown
# [Capability Name] Specification

## Purpose
[Describe what this capability does and why it matters]

## Requirements

### Requirement: [Brief Title]

The system SHALL [shall statement - mandatory requirement].

#### Scenario: [Brief Description]
- GIVEN [preconditions]
- WHEN [action occurs]
- THEN [expected outcome]
```

**Guidelines:**
- Use SHALL for mandatory requirements
- Use SHOULD for recommendations
- Use scenarios as Given-When-Then tests
- Make scenarios specific and testable
- Include error scenarios and edge cases

## Integrating with AI Tools

These OpenSpec files can be used with AI-powered development tools:

### Claude Code / Cursor / GitHub Copilot
Many of these tools have native OpenSpec integration. Simply mention the spec file:

```
Read openspec/specs/export-engine/spec.md and implement the SVG export feature
```

### Custom AI Prompts
When working with AI assistants that don't have native OpenSpec support:

```
I'm working on Sruja 2.0. Here's the specification for the feature I'm implementing:

[ paste the relevant spec section ]

Please review this specification and help me implement it. 
Focus on the scenarios listed under each requirement.
```

## Workflow Examples

### Example 1: Adding a New Export Format

1. **Create proposal:**
   ```bash
   npx openspec:proposal "Add PlantUML export format"
   ```

2. **Update spec:**
   Edit `openspec/specs/export-engine/spec.md` to add PlantUML requirements

3. **Implement:**
   Write code to generate PlantUML syntax

4. **Add tests:**
   Write tests for each PlantUML scenario

5. **Update documentation:**
   Update CLI help and docs

### Example 2: Improving Error Messages

1. **Read validator spec:**
   Review error handling scenarios in `specs/validator/spec.md`

2. **Create proposal:**
   ```bash
   npx openspec:proposal "Improve parser error messages with suggestions"
   ```

3. **Update spec:**
   Add scenarios for improved error messages

4. **Implement:**
   Enhance error generation logic

5. **Test:**
   Verify all error scenarios produce helpful messages

## Version Control

Specifications are versioned with the main codebase:

- **Major version changes** - Update spec structure, rewrite sections
- **Minor version changes** - Add new requirements, add scenarios
- **Patch version changes** - Fix typos, clarify wording

When specs change:
1. Update the specification file
2. Create a change proposal documenting what changed
3. Update related tests
4. Review and merge the change

## Contributing

When contributing to specifications:

1. **Follow the format** - Purpose, Requirements, Scenarios
2. **Be specific** - Scenarios should be testable
3. **Cover edge cases** - Include error scenarios
4. **Keep it aligned** - Ensure specs match SRUJA-2.0-STRATEGY.md
5. **Test your changes** - Write tests for new scenarios

## Resources

- [OpenSpec Documentation](https://openspec.dev)
- [OpenSpec GitHub](https://github.com/Fission-AI/OpenSpec/)
- [Sruja 2.0 Strategy](../SRUJA-2.0-STRATEGY.md)
- [SRUJA-2.0-STRATEGY.md](../SRUJA-2.0-STRATEGY.md)

## License

These specifications are part of the Sruja project and follow the same MIT license as the main codebase.
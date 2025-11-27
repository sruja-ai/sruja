# LSP Autocomplete for Policy & Rule DSL

Complete design and implementation for intelligent, context-aware autocomplete in Policy and Rule DSLs.

## Overview

The Policy/Rule LSP completion provider delivers **world-class developer experience** by providing intelligent autocomplete for:

- Policy DSL structure and keywords
- Rule DSL structure and keywords
- Rule ID suggestions (from registry + MCP)
- Policy ID suggestions
- Metadata key completions
- Field path completions
- Expression DSL operators and values
- Severity level completions
- ADR and Standards ID completions
- Suggested fix commands

## Implementation Status

✅ **Complete**

- Policy/Rule completion provider
- Context-aware analysis
- MCP client interface
- Rule and Policy registries
- Expression DSL completions
- Field path completions
- Metadata key completions
- Snippet templates

## Features

### 1. Keyword Completions

**Policy DSL:**
- `policy` keyword with snippet template
- Policy block keywords: `description`, `applies_to`, `rules`, `controls`, `severity`, etc.

**Rule DSL:**
- `rule` keyword with snippet template
- Rule block keywords: `description`, `applies_to`, `when`, `ensure`, `severity`, `message`, `suggested_fix`

### 2. Rule ID Completion

When typing inside `rules { }` block:
- All registered rules from RuleRegistry
- Rules from MCP server (`listRules()`)
- Built-in rules: `noDirectDBAccess`, `require_rate_limit`, `noInternalImports`, `criticalityRequiresSLO`

### 3. Policy ID Completion

When typing policy IDs:
- All registered policies from PolicyRegistry
- Policies from MCP server (`listPolicies()`)

### 4. Applies To Completion

When typing `applies_to |`:
- Element types: `system`, `container`, `component`, `relation`, `person`, `datastore`, `queue`

### 5. Metadata Key Completion

When typing `metadata.|`:
- Metadata keys from MetadataRegistry
- Metadata keys from MCP server (`listMetadataKeys()`)
- Plugin-contributed metadata keys

### 6. Field Path Completion

When typing field paths like `system.|` or `container.|`:
- Common fields: `id`, `label`, `description`, `technology`, `metadata.*`
- Field paths from MCP server (`listFieldPaths(elementType)`)

### 7. Expression DSL Completion

When typing inside `when { }` or `ensure { }`:
- Operators: `==`, `!=`, `in`, `not in`, `contains`, `starts_with`, `ends_with`, `matches`, `exists`, `not exists`
- Field paths
- Metadata keys

### 8. Severity Completion

When typing `severity |`:
- `info`
- `warning`
- `error`
- `critical`

### 9. ADR ID Completion

When typing inside `related_adrs [ ]`:
- ADR IDs from SemanticIndex
- ADRs from MCP server (`listADRs()`)

### 10. Standards Completion

When typing inside `related_standards [ ]`:
- Standards from MCP server (`listStandards()`)

### 11. Suggested Fix Completion

When typing inside `suggested_fix { }`:
- `add_metadata`
- `remove_metadata`
- `replace_import`
- `add_relation`
- `remove_relation`
- `set`

## Architecture

```
PolicyRuleCompletionProvider
├── SemanticIndex (element references)
├── MetadataRegistry (metadata schemas)
├── RuleRegistry (rule definitions)
├── PolicyRegistry (policy definitions)
└── MCPClient (dynamic data from MCP)
```

## Context Analysis

The provider analyzes cursor position to determine completion context:

1. **PolicyContext** - Inside policy DSL
   - Keyword position
   - Policy block
   - Rules/Controls blocks
   - Field completions

2. **RuleContext** - Inside rule DSL
   - Keyword position
   - Rule block
   - When/Ensure blocks
   - Expression completions
   - Field path completions

## MCP Integration

The completion provider calls MCP server for dynamic data:

- `ListRules()` - Get all available rules
- `ListPolicies()` - Get all available policies
- `ListMetadataKeys()` - Get metadata keys
- `ListFieldPaths(elementType)` - Get field paths for element type
- `ListADRs()` - Get all ADRs
- `ListStandards()` - Get all standards
- `ListPluginFields()` - Get plugin-extended fields

## Usage

```go
// Create provider
provider := NewPolicyRuleCompletionProvider(
    semanticIndex,
    metadataRegistry,
    mcpClient,
)

// Register built-in rules
provider.ruleRegistry.RegisterRule(RuleInfo{
    ID: "noDirectDBAccess",
    Description: "Prevent direct database access",
    AppliesTo: []string{"component"},
})

// Provide completions for Policy DSL
completions, err := provider.ProvidePolicyCompletions(
    filePath,
    text,
    line,
    character,
)

// Provide completions for Rule DSL
completions, err := provider.ProvideRuleCompletions(
    filePath,
    text,
    line,
    character,
)
```

## Snippet Templates

### Policy Template

```sruja
policy ${1:policyId} {
  description "${2:Policy description}"
  applies_to ${3:elementType}
  
  rules {
    ${4:ruleId}
  }
  
  controls {
    ${5}
  }
  
  severity ${6:warning}
}
```

### Rule Template

```sruja
rule ${1:ruleId} {
  description "${2:Rule description}"
  
  applies_to ${3:elementType}
  
  when {
    ${4:condition}
  }
  
  ensure {
    ${5:assertion}
  }
  
  severity ${6:error}
  
  message "${7:Violation message}"
}
```

## Next Steps

1. ✅ **Complete** - Core completion provider
2. 🔄 **In Progress** - Integration with LSP handler
3. 📋 **Planned** - Code actions for suggested fixes
4. 📋 **Planned** - MCP client implementation
5. 📋 **Planned** - Plugin system integration

## Related Documentation

- [LSP Completion Architecture](./lsp-completion-architecture.md)
- [Metadata Model](./metadata-model.md)
- [DX Blueprint](./dx-blueprint.md)



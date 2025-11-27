# LSP Completion Architecture

## Overview

The Sruja DSL Language Server Protocol (LSP) provides intelligent autocomplete powered by:

1. **Metadata Schema Registry** - Core and plugin-contributed metadata keys
2. **Semantic Index** - Cross-file element references and qualified references
3. **Context-Aware Completion** - Understands where the user is typing
4. **Plugin Framework** - Extensible completion suggestions from plugins

This architecture enables auto-completion for:

- Core DSL keywords (`system`, `container`, `component`, etc.)
- Element IDs and references
- Qualified references (`Billing.API`)
- Metadata keys (from core and plugins)
- Metadata values (with enum support)
- Attributes (`technology`, `tags`, `metadata`, etc.)
- Relation targets (after `->`)

## Architecture Components

### 1. Metadata Registry (`pkg/lsp/metadata_registry.go`)

The metadata registry stores all metadata keys that can be autocompleted.

**Core Metadata Keys:**
- `team` - Team or organization
- `owner` - Owner or contact person
- `tier` - Criticality tier (gold, silver, bronze)
- `criticality` - Criticality level
- `tags` - Comma-separated tags
- `cost_center` - Cost center code
- `documentation` - Link to documentation

**Plugin Registration:**
Plugins can register additional metadata keys via `CompletionPlugin` interface.

**Usage:**
```go
registry := NewMetadataRegistry()

// Get metadata for a scope
descriptors := registry.ForScope("container")

// Get value suggestions
suggestions := registry.GetValueSuggestions("tier") // ["gold", "silver", "bronze"]
```

### 2. Semantic Index (`pkg/lsp/semantic_index.go`)

The semantic index maintains a cross-file index of all elements in the workspace.

**Indexed Elements:**
- Systems
- Containers
- Components
- Persons
- Data Stores
- Queues

**Features:**
- Element lookup by ID
- Type-based filtering
- Qualified reference resolution (`Billing.API`)
- Import alias tracking
- Cross-file references

**Usage:**
```go
index := NewSemanticIndex()
index.IndexFile(filePath, program)

// Get elements by type
systems := index.GetByType("system")

// Get elements by prefix (for autocomplete)
elements := index.GetByPrefix("Bill")

// Resolve qualified references
ref, ok := index.ResolveQualifiedID("Billing.API", filePath)
```

### 3. Completion Context (`pkg/lsp/context.go`)

Analyzes the document and cursor position to determine what completions to provide.

**Context Types:**
- `ContextTopLevel` - At architecture top level
- `ContextKeyword` - Typing a keyword
- `ContextElementID` - Typing an element ID
- `ContextElementBlock` - Inside an element block
- `ContextMetadata` - Inside metadata block
- `ContextMetadataKey` - Typing a metadata key
- `ContextMetadataValue` - Typing a metadata value
- `ContextRelation` - In a relation (after `->`)
- `ContextQualifiedReference` - Typing qualified reference (`Billing.`)

**Usage:**
```go
ctx := AnalyzeContext(lines, line, character)
switch ctx.ContextType {
case ContextMetadataKey:
    // Provide metadata key completions
case ContextRelation:
    // Provide element ID completions for relation targets
}
```

### 4. Completion Provider (`pkg/lsp/completion.go`)

Orchestrates all completion logic based on context.

**Methods:**
- `ProvideCompletions()` - Main entry point
- `provideKeywordCompletions()` - Keywords
- `provideElementIDCompletions()` - Element IDs
- `provideMetadataKeyCompletions()` - Metadata keys
- `provideMetadataValueCompletions()` - Metadata values
- `provideRelationCompletions()` - Relation targets
- `provideQualifiedReferenceCompletions()` - Qualified references

**Usage:**
```go
provider := NewCompletionProvider(index, registry)
items, err := provider.ProvideCompletions(filePath, text, line, character)
```

### 5. Plugin Interface (`pkg/lsp/plugin.go`)

Plugins can contribute to autocomplete via the `CompletionPlugin` interface.

**Interface:**
```go
type CompletionPlugin interface {
    MetadataDescriptors(scope string) []*MetadataDescriptor
    GetMetadataDescriptor(key string) (*MetadataDescriptor, bool)
    ProvideCompletions(ctx CompletionContext) []CompletionItem
}
```

**Example Plugin:**
```go
type RateLimitPlugin struct {
    *BasePlugin
}

func NewRateLimitPlugin() *RateLimitPlugin {
    p := NewBasePlugin("rate-limit")
    
    p.RegisterMetadata(&MetadataDescriptor{
        Key:         "rate_limit",
        Type:        "string",
        Description: "Requests per second",
        Scope:       []string{"container"},
    })
    
    return &RateLimitPlugin{BasePlugin: p}
}
```

## Completion Examples

### 1. Keyword Completion

**User types:** `sys|`

**Completions:**
- `system` - Define a system
- `system ${1:ID} "${2:Label}" {`

### 2. Metadata Key Completion

**User types inside metadata block:**
```sruja
metadata {
  |
}
```

**Completions:**
- `team:` - Team or organization
- `owner:` - Owner or contact person
- `tier:` - Criticality tier (gold, silver, bronze)
- `rate_limit:` - Requests per second (from plugin)

### 3. Metadata Value Completion

**User types:**
```sruja
metadata {
  tier: |
}
```

**Completions:**
- `"gold"`
- `"silver"`
- `"bronze"`

### 4. Element Reference Completion

**User types:** `Frontend -> |`

**Completions:**
- `BillingAPI` - System ID
- `BillingDB` - DataStore ID
- `Customer` - Person ID
- `NotificationService` - Container ID

### 5. Qualified Reference Completion

**User types:** `Frontend -> Billing.|`

**Completions:**
- `Billing.API` - From imported architecture
- `Billing.DB` - From imported architecture
- `Billing.Queue` - From imported architecture

### 6. Element Block Completion

**User types inside system block:**
```sruja
system API {
  |
}
```

**Completions:**
- `container` - Define a container
- `component` - Define a component
- `datastore` - Define a datastore
- `queue` - Define a queue
- `metadata` - Add metadata

## LSP Handler Integration

The LSP handler (`pkg/lsp/handler.go`) integrates all components:

```go
type Handler struct {
    semanticIndex    *SemanticIndex
    metadataRegistry *MetadataRegistry
    completionProvider *CompletionProvider
}

// On initialization
func (h *Handler) Handle(ctx context.Context, conn *jsonrpc2.Conn, req *jsonrpc2.Request) {
    switch req.Method {
    case "initialize":
        h.semanticIndex = NewSemanticIndex()
        h.metadataRegistry = NewMetadataRegistry()
        h.completionProvider = NewCompletionProvider(h.semanticIndex, h.metadataRegistry)
        // ... register plugins
    case "textDocument/completion":
        items, _ := h.completionProvider.ProvideCompletions(...)
        // ... return completions
    }
}
```

## File Lifecycle

1. **On File Open:**
   - Parse file
   - Index elements in semantic index
   - Publish diagnostics

2. **On File Change:**
   - Re-parse file (tolerant parsing for incomplete code)
   - Update semantic index
   - Re-publish diagnostics

3. **On Completion Request:**
   - Analyze context
   - Query semantic index
   - Query metadata registry
   - Query plugins
   - Return completion items

## Tolerant Parsing

The completion system uses **tolerant parsing** - it attempts to parse incomplete code and gracefully handles failures.

- If parsing succeeds, full semantic information is available
- If parsing fails, fallback to keyword and basic completions
- Context analysis works even with incomplete syntax

## Best Practices

1. **Keep metadata registry lightweight** - Only core keys, plugins add the rest
2. **Index incrementally** - Update index on file changes, don't rebuild entire workspace
3. **Context is king** - Always analyze context before providing completions
4. **Plugin-first** - Prefer plugin contributions over hardcoded completions
5. **Performance** - Cache completion results when possible

## Future Enhancements

1. **Nested Metadata (v2)** - Support for nested maps/arrays in metadata
2. **Snippet Support** - Rich snippets with tab stops
3. **Completion Ranking** - Score completions by relevance
4. **Incremental Indexing** - Only re-index changed parts
5. **Symbol Indexing** - Index all symbols, not just elements
6. **Cross-Language Support** - Completion for embedded code blocks

## References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Metadata Model](./metadata-model.md)
- [Plugin Framework](./plugin-framework.md) (future)


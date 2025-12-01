# Task 1.7: Language Server Protocol (LSP)

**Priority**: 🟢 Medium (Enhances IDE experience)
**Technology**: Go
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.1 (JSON Exporter), Task 1.2 (JSON to AST), Task 1.5 (Change Commands)

## Overview

Implement Language Server Protocol (LSP) for Sruja DSL to provide IDE features like syntax highlighting, error diagnostics, code completion, hover information, and quick fixes in VS Code, JetBrains, and other LSP-compatible editors.

## Features

### Core LSP Features

1. **Text Document Synchronization**
   - DidOpen, DidChange, DidClose notifications
   - Incremental document updates
   - Workspace-wide document management

2. **Diagnostics (Errors/Warnings)**
   - Parse errors (syntax)
   - Validation errors (semantic)
   - Real-time error reporting
   - Follows [Error Reporting Strategy](../ERROR_REPORTING_STRATEGY.md)

3. **Hover Information**
   - Element details on hover
   - Documentation from metadata
   - ADR information
   - Requirement/user story links

4. **Code Completion**
   - Keyword completion
   - Element ID completion
   - Relation target completion
   - Context-aware suggestions

5. **Go to Definition**
   - Jump to element definition
   - Jump to ADR definition
   - Jump to requirement definition

6. **Find References**
   - Find all references to element
   - Find all references to ADR
   - Find all references to requirement

7. **Code Actions (Quick Fixes)**
   - Auto-fix suggestions
   - Rename element
   - Add missing field
   - Fix common errors

8. **Document Symbols**
   - Outline view (all elements)
   - Hierarchical structure
   - Quick navigation

9. **Workspace Symbols**
   - Search across all files
   - Find elements by name
   - Find ADRs, requirements

10. **Formatting**
    - Auto-format document
    - Format on save
    - Consistent style

## Implementation

### LSP Server Structure

```
pkg/lsp/
  ├── server.go          # Main LSP server
  ├── handlers.go        # Request handlers
  ├── diagnostics.go    # Error diagnostics
  ├── completion.go     # Code completion
  ├── hover.go          # Hover information
  ├── definition.go     # Go to definition
  ├── references.go     # Find references
  ├── actions.go        # Code actions (quick fixes)
  ├── symbols.go        # Document/workspace symbols
  ├── formatting.go     # Formatting
  └── workspace.go      # Workspace management
```

### LSP Server Implementation

```go
// pkg/lsp/server.go
package lsp

import (
    "context"
    "github.com/sourcegraph/go-lsp"
    "github.com/sruja-ai/sruja/pkg/language"
    "github.com/sruja-ai/sruja/pkg/engine"
)

type Server struct {
    workspace *Workspace
    validator *engine.Validator
}

func NewServer() *Server {
    return &Server{
        workspace: NewWorkspace(),
        validator: engine.NewValidator(),
    }
}

func (s *Server) Initialize(ctx context.Context, params lsp.InitializeParams) (*lsp.InitializeResult, error) {
    return &lsp.InitializeResult{
        Capabilities: lsp.ServerCapabilities{
            TextDocumentSync: &lsp.TextDocumentSyncOptions{
                OpenClose: true,
                Change:    lsp.TDSKIncremental,
            },
            HoverProvider:        true,
            CompletionProvider:   &lsp.CompletionOptions{TriggerCharacters: []string{".", ":"}},
            DefinitionProvider:   true,
            ReferencesProvider:   true,
            CodeActionProvider:   true,
            DocumentSymbolProvider: true,
            WorkspaceSymbolProvider: true,
            DocumentFormattingProvider: true,
        },
    }, nil
}

func (s *Server) DidOpen(ctx context.Context, params lsp.DidOpenTextDocumentParams) error {
    doc := params.TextDocument
    s.workspace.AddDocument(doc.URI, doc.Text, doc.Version)
    return s.publishDiagnostics(doc.URI)
}

func (s *Server) DidChange(ctx context.Context, params lsp.DidChangeTextDocumentParams) error {
    doc := params.TextDocument
    changes := params.ContentChanges
    
    // Apply incremental changes
    for _, change := range changes {
        s.workspace.ApplyChange(doc.URI, change)
    }
    
    return s.publishDiagnostics(doc.URI)
}

func (s *Server) publishDiagnostics(uri lsp.DocumentURI) error {
    doc := s.workspace.GetDocument(uri)
    if doc == nil {
        return nil
    }
    
    // Parse document
    program, parseErrors := language.ParseString(doc.Text)
    
    // Validate
    var validationErrors []language.ValidationError
    if program != nil {
        validationErrors = s.validator.Validate(program)
    }
    
    // Convert to LSP diagnostics
    diagnostics := s.convertToDiagnostics(parseErrors, validationErrors)
    
    // Publish diagnostics
    return s.notifyDiagnostics(uri, diagnostics)
}
```

### Diagnostics

```go
// pkg/lsp/diagnostics.go
func (s *Server) convertToDiagnostics(
    parseErrors []language.ParseError,
    validationErrors []language.ValidationError,
) []lsp.Diagnostic {
    var diagnostics []lsp.Diagnostic
    
    // Parse errors
    for _, err := range parseErrors {
        diagnostics = append(diagnostics, lsp.Diagnostic{
            Range: lsp.Range{
                Start: lsp.Position{
                    Line:      err.Location.Line - 1,
                    Character: err.Location.Column - 1,
                },
                End: lsp.Position{
                    Line:      err.Location.Line - 1,
                    Character: err.Location.Column - 1 + len(err.Message),
                },
            },
            Severity: lsp.Error,
            Code:     err.Code,
            Source:   "sruja",
            Message:  err.Message,
            RelatedInformation: s.buildRelatedInfo(err),
            Data: map[string]interface{}{
                "suggestions": err.Suggestions,
                "quickFixes":  s.buildQuickFixes(err),
            },
        })
    }
    
    // Validation errors
    for _, err := range validationErrors {
        diagnostics = append(diagnostics, lsp.Diagnostic{
            Range: lsp.Range{
                Start: lsp.Position{
                    Line:      err.Location.Line - 1,
                    Character: err.Location.Column - 1,
                },
                End: lsp.Position{
                    Line:      err.Location.Line - 1,
                    Character: err.Location.Column - 1 + len(err.Message),
                },
            },
            Severity: s.severityToLSP(err.Severity),
            Code:     err.RuleID,
            Source:   "sruja",
            Message:  err.Message,
            RelatedInformation: s.buildRelatedInfo(err),
            Data: map[string]interface{}{
                "suggestions": err.Suggestions,
                "quickFixes":  s.buildQuickFixes(err),
            },
        })
    }
    
    return diagnostics
}
```

### Code Completion

```go
// pkg/lsp/completion.go
func (s *Server) Completion(ctx context.Context, params lsp.CompletionParams) (*lsp.CompletionList, error) {
    doc := s.workspace.GetDocument(params.TextDocument.URI)
    if doc == nil {
        return nil, nil
    }
    
    position := params.Position
    line := doc.GetLine(position.Line)
    beforeCursor := line[:position.Character]
    
    // Determine completion context
    context := s.analyzeCompletionContext(beforeCursor, doc, position)
    
    var items []lsp.CompletionItem
    
    switch context.Type {
    case CompletionTypeKeyword:
        items = s.completeKeywords(context)
    case CompletionTypeElementID:
        items = s.completeElementIDs(context)
    case CompletionTypeRelationTarget:
        items = s.completeRelationTargets(context)
    case CompletionTypeProperty:
        items = s.completeProperties(context)
    }
    
    return &lsp.CompletionList{
        IsIncomplete: false,
        Items:        items,
    }, nil
}
```

### Hover Information

```go
// pkg/lsp/hover.go
func (s *Server) Hover(ctx context.Context, params lsp.TextDocumentPositionParams) (*lsp.Hover, error) {
    doc := s.workspace.GetDocument(params.TextDocument.URI)
    if doc == nil {
        return nil, nil
    }
    
    position := params.Position
    element := s.findElementAtPosition(doc, position)
    
    if element == nil {
        return nil, nil
    }
    
    // Build hover content
    content := s.buildHoverContent(element)
    
    return &lsp.Hover{
        Contents: lsp.MarkupContent{
            Kind:  lsp.Markdown,
            Value: content,
        },
        Range: &lsp.Range{
            Start: element.Range.Start,
            End:   element.Range.End,
        },
    }, nil
}

func (s *Server) buildHoverContent(element *Element) string {
    var parts []string
    
    // Element type and ID
    parts = append(parts, fmt.Sprintf("**%s**: `%s`", element.Type, element.ID))
    
    // Label
    if element.Label != "" {
        parts = append(parts, fmt.Sprintf("**Label**: %s", element.Label))
    }
    
    // Description
    if element.Description != "" {
        parts = append(parts, fmt.Sprintf("\n%s", element.Description))
    }
    
    // Metadata
    if len(element.Metadata) > 0 {
        parts = append(parts, "\n**Metadata:**")
        for k, v := range element.Metadata {
            parts = append(parts, fmt.Sprintf("- %s: %v", k, v))
        }
    }
    
    // ADR links
    if len(element.ADRs) > 0 {
        parts = append(parts, "\n**ADRs:**")
        for _, adr := range element.ADRs {
            parts = append(parts, fmt.Sprintf("- [%s](%s)", adr.ID, adr.URI))
        }
    }
    
    return strings.Join(parts, "\n")
}
```

### Code Actions (Quick Fixes)

```go
// pkg/lsp/actions.go
func (s *Server) CodeAction(ctx context.Context, params lsp.CodeActionParams) ([]lsp.CodeAction, error) {
    var actions []lsp.CodeAction
    
    for _, diagnostic := range params.Context.Diagnostics {
        // Get quick fixes from diagnostic data
        if quickFixes, ok := diagnostic.Data["quickFixes"].([]QuickFix); ok {
            for _, fix := range quickFixes {
                actions = append(actions, lsp.CodeAction{
                    Title:       fix.Title,
                    Kind:        lsp.QuickFix,
                    Diagnostics: []lsp.Diagnostic{diagnostic},
                    Edit: &lsp.WorkspaceEdit{
                        Changes: map[string][]lsp.TextEdit{
                            string(params.TextDocument.URI): fix.Edits,
                        },
                    },
                })
            }
        }
    }
    
    return actions, nil
}
```

## CLI Command

```bash
# Start LSP server (stdin/stdout)
sruja lsp

# Or via extension (VS Code/JetBrains launches it)
```

## Integration

### VS Code Extension

**Extension launches LSP server**:
```json
// package.json
{
  "contributes": {
    "languages": [{
      "id": "sruja",
      "extensions": [".sruja"]
    }],
    "configuration": {
      "type": "object",
      "title": "Sruja",
      "properties": {
        "sruja.lsp.path": {
          "type": "string",
          "default": "sruja",
          "description": "Path to Sruja CLI"
        }
      }
    }
  }
}
```

### JetBrains Plugin

**Plugin launches LSP server**:
```kotlin
// Plugin registers LSP server
class SrujaLanguageServer : LanguageServerBase() {
    override fun createServer(): LanguageServer {
        val process = ProcessBuilder("sruja", "lsp").start()
        return LanguageServer(process.inputStream, process.outputStream)
    }
}
```

## Error Reporting

Follows [Error Reporting Strategy](../ERROR_REPORTING_STRATEGY.md):
- ✅ LSP Diagnostic format
- ✅ Error codes (PARSE_001, VALID_001, etc.)
- ✅ Location information (file, line, column)
- ✅ Suggestions and quick fixes
- ✅ Related error information

## Acceptance Criteria

- [ ] LSP server implements core protocol (initialize, text sync, diagnostics)
- [ ] Diagnostics show parse errors in real-time
- [ ] Diagnostics show validation errors in real-time
- [ ] Hover shows element information
- [ ] Code completion works (keywords, element IDs, properties)
- [ ] Go to definition works (elements, ADRs, requirements)
- [ ] Find references works (elements, ADRs, requirements)
- [ ] Code actions (quick fixes) work
- [ ] Document symbols work (outline view)
- [ ] Workspace symbols work (search)
- [ ] Formatting works (auto-format, format on save)
- [ ] Works with VS Code extension
- [ ] Works with JetBrains plugin
- [ ] Error reporting follows strategy document
- [ ] Performance: Diagnostics update in < 100ms for typical files

## Dependencies

- Task 1.1 (JSON Exporter) - For workspace analysis
- Task 1.2 (JSON to AST) - For parsing
- Task 1.5 (Change Commands) - For change-aware diagnostics
- Error Reporting Strategy - For error format

## Notes

- **LSP Library**: Use `github.com/sourcegraph/go-lsp` or similar
- **Performance**: Incremental parsing, caching, debouncing
- **Workspace**: Track all open files, handle imports
- **Error Recovery**: Continue parsing even with errors (partial AST)


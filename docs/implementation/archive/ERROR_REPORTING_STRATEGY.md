# Error Reporting Strategy

## Overview

Consistent, user-friendly error reporting across all Sruja interfaces: CLI, Public Studio, Self-Hosted Studio, CLI Studio, and LSP (VS Code/JetBrains).

## Error Types

### 1. Parse Errors
**When**: Invalid DSL syntax
**Examples**:
- Missing closing brace
- Invalid keyword
- Malformed relation syntax
- Unexpected token

### 2. Validation Errors
**When**: Architecture violates rules
**Examples**:
- Duplicate element IDs
- Invalid references (element doesn't exist)
- Circular dependencies
- Orphaned elements
- Invalid change states
- ADR state violations

### 3. Compilation Errors
**When**: Transformation failures
**Examples**:
- Missing required fields
- Invalid element type
- JSON schema violations
- Round-trip failures

### 4. File System Errors
**When**: File operations fail
**Examples**:
- File not found
- Permission denied
- Invalid file path
- Disk full

### 5. Network Errors
**When**: Network operations fail (Public Studio, Self-Hosted Studio)
**Examples**:
- Failed to fetch DSL from URL
- Google Drive API errors
- Timeout errors

## Error Structure

### Standard Error Format

```go
// pkg/language/errors.go
type SrujaError struct {
    Type      ErrorType      // parse, validation, compilation, filesystem, network
    Severity  ErrorSeverity  // error, warning, info
    Location  SourceLocation  // file, line, column
    Code      string         // Error code (e.g., "PARSE_001", "VALID_002")
    Message   string         // Human-readable message
    Details   string         // Additional context
    Suggestions []string     // Suggested fixes
    Context   map[string]interface{} // Additional context data
}

type SourceLocation struct {
    File   string
    Line   int
    Column int
}

type ErrorType string
const (
    ErrorTypeParse        ErrorType = "parse"
    ErrorTypeValidation   ErrorType = "validation"
    ErrorTypeCompilation  ErrorType = "compilation"
    ErrorTypeFilesystem   ErrorType = "filesystem"
    ErrorTypeNetwork      ErrorType = "network"
)

type ErrorSeverity string
const (
    ErrorSeverityError   ErrorSeverity = "error"
    ErrorSeverityWarning ErrorSeverity = "warning"
    ErrorSeverityInfo    ErrorSeverity = "info"
)
```

### Error Codes

**Format**: `{TYPE}_{NUMBER}`

**Parse Errors** (`PARSE_*`):
- `PARSE_001`: Missing closing brace
- `PARSE_002`: Unexpected token
- `PARSE_003`: Invalid keyword
- `PARSE_004`: Malformed relation syntax

**Validation Errors** (`VALID_*`):
- `VALID_001`: Duplicate element ID
- `VALID_002`: Invalid reference
- `VALID_003`: Circular dependency detected
- `VALID_004`: Orphaned element
- `VALID_005`: Change not in final state
- `VALID_006`: ADR not in final state
- `VALID_007`: Conflict between changes

**Compilation Errors** (`COMP_*`):
- `COMP_001`: Missing required field
- `COMP_002`: Invalid element type
- `COMP_003`: JSON schema violation
- `COMP_004`: Round-trip failure

**File System Errors** (`FILE_*`):
- `FILE_001`: File not found
- `FILE_002`: Permission denied
- `FILE_003`: Invalid file path
- `FILE_004`: Disk full

**Network Errors** (`NET_*`):
- `NET_001`: Failed to fetch URL
- `NET_002`: Google Drive API error
- `NET_003`: Timeout error

## Interface-Specific Error Reporting

### 1. CLI

**Format**: Structured, machine-parseable + human-readable

**Output Format**:
```bash
# JSON mode (--json flag)
$ sruja lint --json architecture.sruja
{
  "errors": [
    {
      "type": "validation",
      "severity": "error",
      "code": "VALID_001",
      "location": {
        "file": "architecture.sruja",
        "line": 10,
        "column": 5
      },
      "message": "Duplicate element ID 'API'",
      "details": "Element 'API' is already defined at line 5",
      "suggestions": [
        "Rename one of the elements",
        "Remove the duplicate definition"
      ]
    }
  ],
  "warnings": [],
  "summary": {
    "total": 1,
    "errors": 1,
    "warnings": 0
  }
}

# Human-readable mode (default)
$ sruja lint architecture.sruja
❌ Error: architecture.sruja:10:5 [VALID_001]
   Duplicate element ID 'API'
   
   Details: Element 'API' is already defined at line 5
   
   Suggestions:
   • Rename one of the elements
   • Remove the duplicate definition
```

**Features**:
- ✅ Color-coded output (red for errors, yellow for warnings)
- ✅ File:line:column location
- ✅ Error codes for reference
- ✅ Suggestions for fixes
- ✅ JSON output option for tooling
- ✅ Exit codes: 0 (success), 1 (errors), 2 (warnings only)

**Example**:
```bash
$ sruja change apply
❌ Error: Cannot apply changes - validation failed

Changes not in final state:
  • changes/003-add-analytics.sruja:10:5 [VALID_005]
    Change status is "in-progress" but must be "approved" or "deferred"
    
ADRs not in final state:
  • ADR-001 (referenced by change 003-add-analytics):15:3 [VALID_006]
    ADR status is "pending" but must be "decided" or "rejected"
    
All changes and their referenced ADRs must be in final state before applying.
```

### 2. Public Studio (Web App)

**Format**: Inline, visual, user-friendly

**UI Components**:
- Error banner at top
- Inline error markers on canvas
- Error panel (collapsible)
- Toast notifications for transient errors

**Error Display**:
```typescript
// Error Banner Component
<ErrorBanner>
  <ErrorIcon />
  <ErrorCount>3 errors, 2 warnings</ErrorCount>
  <ErrorDetails>
    {errors.map(error => (
      <ErrorItem key={error.code}>
        <ErrorLocation>{error.location.file}:{error.location.line}</ErrorLocation>
        <ErrorMessage>{error.message}</ErrorMessage>
        {error.suggestions && (
          <ErrorSuggestions>
            {error.suggestions.map(suggestion => (
              <SuggestionButton onClick={() => applyFix(suggestion)}>
                {suggestion}
              </SuggestionButton>
            ))}
          </ErrorSuggestions>
        )}
      </ErrorItem>
    ))}
  </ErrorDetails>
</ErrorBanner>

// Inline Error Markers on Canvas
<CytoscapeNode>
  {hasError && (
    <ErrorBadge>
      <ErrorIcon />
      <ErrorTooltip>
        {error.message}
        {error.suggestions && (
          <Suggestions>
            {error.suggestions.map(s => <Suggestion>{s}</Suggestion>)}
          </Suggestions>
        )}
      </ErrorTooltip>
    </ErrorBadge>
  )}
</CytoscapeNode>
```

**Features**:
- ✅ Visual error indicators on diagram elements
- ✅ Click to see details
- ✅ Quick fix suggestions (buttons)
- ✅ Error count badge
- ✅ Collapsible error panel
- ✅ Toast notifications for network errors
- ✅ Auto-dismiss for warnings

**Example**:
```
┌─────────────────────────────────────┐
│ ⚠️ 3 errors, 2 warnings  [Show All] │
├─────────────────────────────────────┤
│ architecture.sruja:10:5             │
│ Duplicate element ID 'API'          │
│ [Rename] [Remove duplicate]         │
└─────────────────────────────────────┘
```

### 3. Self-Hosted Studio

**Format**: Similar to Public Studio, with server-side validation

**Additional Features**:
- Server-side validation before saving
- Error persistence (save errors with preview)
- Error history (track errors over time)

**API Error Response**:
```json
{
  "success": false,
  "error": {
    "type": "validation",
    "code": "VALID_001",
    "message": "Duplicate element ID 'API'",
    "location": {
      "file": "architecture.sruja",
      "line": 10,
      "column": 5
    },
    "suggestions": [
      "Rename one of the elements",
      "Remove the duplicate definition"
    ]
  }
}
```

**Frontend Display**: Same as Public Studio

### 4. CLI Studio (Local)

**Format**: Hybrid - CLI errors + Web UI

**CLI Errors** (when starting/stopping):
```bash
$ sruja studio
✅ Studio started on http://localhost:5173

# If port in use:
❌ Error: Port 5173 is already in use
   Details: Another process is using port 5173
   Suggestions:
   • Use --port flag to specify different port
   • Stop the other process using port 5173
```

**Web UI Errors**: Same as Public Studio (inline, visual)

**File Operation Errors** (via API):
```json
// POST /api/save
{
  "success": false,
  "error": {
    "type": "filesystem",
    "code": "FILE_002",
    "message": "Permission denied",
    "location": {
      "file": "architecture.sruja"
    },
    "details": "Cannot write to architecture.sruja: permission denied"
  }
}
```

### 5. LSP (Language Server Protocol)

**Format**: LSP Diagnostic format

**LSP Diagnostic**:
```json
{
  "range": {
    "start": { "line": 9, "character": 4 },
    "end": { "line": 9, "character": 7 }
  },
  "severity": 1,  // Error = 1, Warning = 2, Info = 3
  "code": "VALID_001",
  "source": "sruja",
  "message": "Duplicate element ID 'API'",
  "relatedInformation": [
    {
      "location": {
        "uri": "file:///path/to/architecture.sruja",
        "range": {
          "start": { "line": 4, "character": 4 },
          "end": { "line": 4, "character": 7 }
        }
      },
      "message": "First definition of 'API'"
    }
  ],
  "data": {
    "suggestions": [
      "Rename one of the elements",
      "Remove the duplicate definition"
    ],
    "quickFixes": [
      {
        "title": "Rename to 'APIv2'",
        "edit": {
          "changes": {
            "file:///path/to/architecture.sruja": [
              {
                "range": {
                  "start": { "line": 9, "character": 4 },
                  "end": { "line": 9, "character": 7 }
                },
                "newText": "APIv2"
              }
            ]
          }
        }
      }
    ]
  }
}
```

**Features**:
- ✅ Underline errors in editor
- ✅ Hover for details
- ✅ Quick fixes (Code Actions)
- ✅ Related information (link to other occurrences)
- ✅ Error codes for filtering
- ✅ Workspace-wide diagnostics

**VS Code/JetBrains Integration**:
- Red squiggles for errors
- Yellow squiggles for warnings
- Hover tooltips with details
- Quick fix suggestions (lightbulb)
- Problems panel with all errors
- Go to definition/related errors

## Error Reporting Flow

### 1. Error Detection

```go
// pkg/language/validator.go
func Validate(program *Program) []SrujaError {
    var errors []SrujaError
    
    // Run validation rules
    errors = append(errors, checkDuplicateIDs(program)...)
    errors = append(errors, checkValidReferences(program)...)
    errors = append(errors, checkCycles(program)...)
    errors = append(errors, checkOrphans(program)...)
    
    return errors
}
```

### 2. Error Formatting

```go
// pkg/language/errors.go
func (e *SrujaError) FormatCLI() string {
    var sb strings.Builder
    
    // Severity icon
    switch e.Severity {
    case ErrorSeverityError:
        sb.WriteString("❌ Error: ")
    case ErrorSeverityWarning:
        sb.WriteString("⚠️  Warning: ")
    case ErrorSeverityInfo:
        sb.WriteString("ℹ️  Info: ")
    }
    
    // Location
    sb.WriteString(fmt.Sprintf("%s:%d:%d", e.Location.File, e.Location.Line, e.Location.Column))
    
    // Code
    sb.WriteString(fmt.Sprintf(" [%s]\n", e.Code))
    
    // Message
    sb.WriteString(fmt.Sprintf("   %s\n", e.Message))
    
    // Details
    if e.Details != "" {
        sb.WriteString(fmt.Sprintf("\n   Details: %s\n", e.Details))
    }
    
    // Suggestions
    if len(e.Suggestions) > 0 {
        sb.WriteString("\n   Suggestions:\n")
        for _, suggestion := range e.Suggestions {
            sb.WriteString(fmt.Sprintf("   • %s\n", suggestion))
        }
    }
    
    return sb.String()
}

func (e *SrujaError) FormatJSON() map[string]interface{} {
    return map[string]interface{}{
        "type":       e.Type,
        "severity":   e.Severity,
        "code":       e.Code,
        "location":   e.Location,
        "message":    e.Message,
        "details":    e.Details,
        "suggestions": e.Suggestions,
    }
}

func (e *SrujaError) FormatLSP() lsp.Diagnostic {
    return lsp.Diagnostic{
        Range: lsp.Range{
            Start: lsp.Position{Line: e.Location.Line - 1, Character: e.Location.Column - 1},
            End:   lsp.Position{Line: e.Location.Line - 1, Character: e.Location.Column - 1 + len(e.Message)},
        },
        Severity: e.severityToLSP(),
        Code:     e.Code,
        Source:   "sruja",
        Message:  e.Message,
        RelatedInformation: e.buildRelatedInfo(),
        Data: map[string]interface{}{
            "suggestions": e.Suggestions,
            "quickFixes":  e.buildQuickFixes(),
        },
    }
}
```

### 3. Error Aggregation

```go
// pkg/language/error_list.go
type ErrorList struct {
    Errors   []SrujaError
    Warnings []SrujaError
    Info     []SrujaError
}

func (el *ErrorList) Add(err SrujaError) {
    switch err.Severity {
    case ErrorSeverityError:
        el.Errors = append(el.Errors, err)
    case ErrorSeverityWarning:
        el.Warnings = append(el.Warnings, err)
    case ErrorSeverityInfo:
        el.Info = append(el.Info, err)
    }
}

func (el *ErrorList) FormatCLI() string {
    var sb strings.Builder
    
    // Errors
    for _, err := range el.Errors {
        sb.WriteString(err.FormatCLI())
        sb.WriteString("\n")
    }
    
    // Warnings
    for _, err := range el.Warnings {
        sb.WriteString(err.FormatCLI())
        sb.WriteString("\n")
    }
    
    // Summary
    sb.WriteString(fmt.Sprintf("\nSummary: %d errors, %d warnings\n", len(el.Errors), len(el.Warnings)))
    
    return sb.String()
}
```

## Error Recovery & Suggestions

### Auto-Fix Suggestions

**CLI**:
```bash
$ sruja lint --fix architecture.sruja
⚠️  Warning: architecture.sruja:10:5 [VALID_001]
   Duplicate element ID 'API'
   
   Auto-fix available:
   • Rename to 'APIv2' (recommended)
   • Remove duplicate definition
   
   Apply fix? [y/N]: y
✅ Fixed: Renamed 'API' to 'APIv2' at line 10
```

**LSP** (Code Actions):
- Quick fix: Rename element
- Quick fix: Remove duplicate
- Quick fix: Add missing field

**Studio** (UI):
- Button: "Rename to 'APIv2'"
- Button: "Remove duplicate"
- Button: "Show all occurrences"

### Error Context

**Related Errors**:
```go
// Group related errors
errors := []SrujaError{
    {
        Code: "VALID_002",
        Message: "Invalid reference 'NonExistent'",
        Related: []SrujaError{
            {
                Code: "VALID_001",
                Message: "Element 'NonExistent' not found",
                Location: SourceLocation{File: "other.sruja", Line: 5},
            },
        },
    },
}
```

**Error Chains**:
```bash
❌ Error: architecture.sruja:10:5 [COMP_001]
   Missing required field 'id'
   
   Caused by:
   • architecture.sruja:10:5 [PARSE_001]
     Invalid syntax: expected identifier
```

## Best Practices

### 1. Error Messages

✅ **Do**:
- Use clear, actionable language
- Include context (what, where, why)
- Provide suggestions
- Use consistent terminology

❌ **Don't**:
- Use technical jargon
- Blame the user
- Provide vague messages
- Include implementation details

### 2. Error Codes

✅ **Do**:
- Use consistent format (`TYPE_NUMBER`)
- Document all codes
- Make codes searchable
- Group related codes

### 3. Error Location

✅ **Do**:
- Always include file, line, column
- Use 1-based line numbers (human-friendly)
- Include character range for LSP
- Link related errors

### 4. Error Severity

✅ **Do**:
- Use error for blocking issues
- Use warning for non-blocking issues
- Use info for suggestions
- Be consistent across interfaces

### 5. Error Suggestions

✅ **Do**:
- Provide actionable suggestions
- Order by likelihood/impact
- Include quick fixes when possible
- Link to documentation

## Error Documentation

**Error Code Reference**:
```
docs/errors/
  ├── PARSE_*.md    # Parse error codes
  ├── VALID_*.md    # Validation error codes
  ├── COMP_*.md     # Compilation error codes
  ├── FILE_*.md     # File system error codes
  └── NET_*.md      # Network error codes
```

**Example** (`docs/errors/VALID_001.md`):
```markdown
# VALID_001: Duplicate Element ID

**Type**: Validation Error  
**Severity**: Error  
**Description**: An element ID is defined multiple times.

**Example**:
```sruja
system API {}
system API {}  // Error: Duplicate ID
```

**Fix**:
- Rename one of the elements
- Remove the duplicate definition

**Related Codes**:
- VALID_002: Invalid reference (may be caused by duplicate IDs)
```

## Summary

**Consistent Error Structure**:
- Standard error format across all interfaces
- Error codes for reference
- Location information (file, line, column)
- Suggestions for fixes

**Interface-Specific Formatting**:
- CLI: Human-readable + JSON option
- Studio: Visual, inline, interactive
- LSP: Diagnostic format with quick fixes

**Error Recovery**:
- Auto-fix suggestions
- Quick fixes (LSP, Studio)
- Related error linking

**Documentation**:
- Error code reference
- Examples and fixes
- Best practices


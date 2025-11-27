# DSL Parser Engine

**Status**: Core Engine  
**Pillars**: Core (Parsing)

[← Back to Engines](../README.md)

## Overview

The DSL Parser Engine parses DSL text into an Abstract Syntax Tree (AST) using participle (Go parser library), providing syntax validation and error reporting.

**This is the foundation of the architecture compiler pipeline.**

## Purpose

The DSL Parser Engine:

- ✅ Parses DSL text → AST
- ✅ Validates syntax
- ✅ Reports syntax errors
- ✅ Preserves source locations
- ✅ Handles comments and whitespace
- ✅ Supports incremental parsing
- ✅ Integrates with LSP

## Architecture

```
DSL Text
   ↓
Parser (participle)
   ↓
AST (Abstract Syntax Tree)
   ↓
AST Transformer
   ↓
IR (Intermediate Representation)
```

## Grammar Technology

Uses **participle** (Go parser library) for parsing:

- Struct-based parser combinators (Go-idiomatic)
- Built-in error reporting
- Source location tracking
- Extensible grammar (handles all extensions)
- Fast parsing performance
- Handles all DSL features: core, extensions, systems thinking

## DSL Grammar Structure

The grammar supports:

### Statements
- Import statements
- Node declarations
- Edge declarations
- Comments

### Node Declarations
```
identifier: type "label" { metadata }
```

### Edge Declarations
```
identifier -> identifier "label"
```

### Metadata Blocks
```
{
  key: value,
  key: value
}
```

## Parsing Process

### Step 1 — Lexical Analysis
- Tokenize input text
- Identify keywords, identifiers, literals
- Handle whitespace and comments

### Step 2 — Syntax Analysis
- Parse tokens into AST
- Validate grammar rules
- Build tree structure

### Step 3 — Error Reporting
- Collect syntax errors
- Report with source locations
- Provide helpful error messages

## AST Structure

The parser produces an AST with:

```ts
interface ASTNode {
  type: string;
  location: SourceLocation;
  children?: ASTNode[];
  value?: any;
}
```

Example AST nodes:

- `ImportStatement`
- `NodeDeclaration`
- `EdgeDeclaration`
- `MetadataBlock`
- `Comment`

## Source Location Tracking

Every AST node includes:

```ts
interface SourceLocation {
  file: string;
  line: number;
  column: number;
  offset: number;
  length: number;
}
```

Used for:
- Error reporting
- LSP diagnostics
- Code navigation

## Error Handling

The parser provides:

- Syntax error detection
- Error recovery
- Detailed error messages
- Source location in errors

Example error:

```
Syntax error at line 5, column 12:
  Expected ':' after identifier
  api service "API"
           ^
```

## Integration Points

### AST Transformer
- Consumes AST
- Transforms to IR

### Validation Engine
- Validates AST structure
- Checks semantic rules

### LSP Integration
- Provides diagnostics
- Enables code completion
- Supports go-to-definition

## Incremental Parsing

Supports:

- Partial re-parsing
- Change detection
- Minimal recompilation
- Fast feedback

## MCP API

```
parser.parse(text)
parser.parseFile(path)
parser.validate(text)
parser.getAST(text)
```

## Strategic Value

The DSL Parser Engine provides:

- ✅ Foundation for all DSL processing
- ✅ Syntax validation
- ✅ Error reporting
- ✅ LSP integration
- ✅ Fast parsing

**This is critical for DSL support and developer experience.**

## Implementation Status

✅ Architecture designed  
✅ Grammar specified (participle)  
✅ AST structure defined  
✅ Parser technology decided (participle for all features)  
📋 Implementation in progress

---

*The DSL Parser Engine parses DSL text into AST using participle. All DSL features (core, extensions, systems thinking) are parsed with participle.*


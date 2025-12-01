# DSL Syntax Consistency Check

## Overview

This document verifies DSL syntax consistency across:
- Actual parser implementation
- Existing examples in codebase
- Documentation examples
- Test cases

## Current DSL Syntax (From Parser)

### Basic Elements

**System**:
```sruja
system <ID> [<Name>] {
  // ... nested elements
}
```

**Container**:
```sruja
container <ID> [<Name>] {
  // ... nested elements
}
```

**Component**:
```sruja
component <ID> [<Name>] {
  // ... properties
}
```

**Relation**:
```sruja
<From> -> <To> [<Label>]
```

### Metadata Block

**Current Syntax** (from parser):
```sruja
metadata {
  <key>: <value>
  <key>: <value>
}
```

**Example from codebase** (`examples/metadata_showcase.sruja`):
```sruja
metadata {
  team: "Payments"
  tier: "critical"
  rate_limit_per_ip: "50/s"
}
```

**Parser Definition** (`pkg/language/ast.go`):
```go
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}
```

**Note**: Parser currently only supports string values. Arrays need to be added for `stakeholders`.

### ADR Syntax

**Current Syntax** (from parser):
```sruja
adr <ID> <Title> {
  tags [<refs>]
  status <status>
  context <context>
  decision <decision>
  consequences [<consequence1>, ...]
}
```

## Inconsistencies Found

### 1. Metadata Syntax in Change Blocks

**Issue**: Documentation uses different syntax for metadata in change blocks.

**Documentation** (DSL_CHANGES_REQUIRED.md):
```sruja
metadata {
  owner "<owner>"  // String value
  stakeholders ["<stakeholder1>", "<stakeholder2>", ...]  // Array
}
```

**Actual Parser** (ast.go):
```sruja
metadata {
  <key>: <value>  // Uses colon, not equals
}
```

**Fix Required**: 
1. Metadata in change blocks should use colon syntax
2. Parser needs to support array values for `stakeholders`

**Current Parser Limitation**: `MetaEntry` only supports string values. Need to extend to support arrays.

**Proposed Fix**:
```go
type MetaEntry struct {
    Key   string      `parser:"@Ident ':'"`
    Value interface{} `parser:"@@"`  // String or Array
}

// Or separate types:
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}

type MetaArrayEntry struct {
    Key   string   `parser:"@Ident ':'"`
    Value []string `parser:"'[' @String ( ',' @String )* ']'"`
}
```

**Corrected Syntax**:
```sruja
metadata {
  owner: "alice@example.com"
  stakeholders: ["bob@example.com", "charlie@example.com"]
}
```

### 2. Metadata Array Syntax

**Issue**: Arrays in metadata are NOT currently supported by parser.

**Current Parser**: `MetaEntry` only supports `string` values, not arrays.

**Documentation**: Uses `stakeholders: ["value1", "value2"]` (array of strings)

**Fix Required**: 
- Extend parser to support array values in metadata
- Add `MetaArrayEntry` type or extend `MetaEntry` to support both string and array values
- Update parser grammar to handle both cases

### 3. Change Block Fields

**Issue**: Field syntax in change blocks.

**Documentation**:
```sruja
change "001-add-api" {
  version "v1.1.0"
  requirement "REQ-001"
  adr "ADR-001"
  status "pending"
}
```

**Consistency Check**: These look correct (string values).

### 4. Qualified Names in Relations

**Issue**: Relation syntax with qualified names.

**Documentation Examples**:
```sruja
relation ShopSystem.WebApp -> ShopSystem.AnalyticsAPI "Sends metrics"
```

**Actual Examples**:
```sruja
User -> WebApp "Uses"
WebApp -> Database "Reads/Writes"
```

**Consistency**: Both should work. Qualified names are optional.

### 5. System/Container/Component Name Syntax

**Issue**: Optional name string syntax.

**Documentation**:
```sruja
system WebApp {}
container WebApp "Web Application" {}
```

**Actual Examples**:
```sruja
system API "API Service" {
container WebApp "Web Application" {
```

**Consistency**: ✅ Both are correct - name is optional.

## Corrected Syntax Examples

### Change Block (Corrected)

```sruja
change "001-add-api" {
  version "v1.1.0"
  requirement "REQ-001"
  adr "ADR-001"
  status "approved"
  
  metadata {
    owner: "alice@example.com"
    stakeholders: ["bob@example.com", "charlie@example.com", "Platform Team"]
  }
  
  add {
    system API {}
    relation WebApp -> API "Calls"
  }
}
```

### Metadata Block (Consistent)

```sruja
// In architecture/elements
metadata {
  team: "Payments"
  tier: "critical"
}

// In change blocks (same syntax)
metadata {
  owner: "alice@example.com"
  stakeholders: ["bob@example.com"]
}
```

### ADR (Consistent)

```sruja
adr "ADR-001" "Use REST API for analytics" {
  tags [
    container "ShopSystem.AnalyticsAPI"
  ]
  status "decided"
  context "Need to provide analytics data to external systems"
  decision "Use REST API with OAuth2 authentication"
  consequences [
    "Standard protocol - easy integration"
    "Requires authentication layer"
  ]
}
```

## Required Fixes

### 1. Parser Changes (CRITICAL)

**Issue**: Parser doesn't support array values in metadata.

**Current**:
```go
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}
```

**Required**:
```go
// Option 1: Union type
type MetaEntry struct {
    Key   string      `parser:"@Ident ':'"`
    Value interface{} `parser:"@@"`  // String or Array
}

// Option 2: Separate types (recommended)
type MetaEntry struct {
    Key   string `parser:"@Ident ':'"`
    Value string `parser:"@String"`
}

type MetaArrayEntry struct {
    Key   string   `parser:"@Ident ':'"`
    Value []string `parser:"'[' @String ( ',' @String )* ']'"`
}

// Update MetadataBlock to accept both
type MetadataBlock struct {
    LBrace  string       `parser:"'metadata' '{'"`
    Entries []MetaValue  `parser:"@@*"`  // Union of MetaEntry and MetaArrayEntry
    RBrace  string       `parser:"'}'"`
}
```

**Action**: Update parser before implementing change tracking.

### 2. Update DSL_CHANGES_REQUIRED.md

**Change**: Metadata syntax in change blocks should use colon (`:`) not assignment

**Before**:
```sruja
metadata {
  owner "<owner>"
  stakeholders ["<stakeholder1>", ...]
}
```

**After**:
```sruja
metadata {
  owner: "<owner>"
  stakeholders: ["<stakeholder1>", ...]
}
```

### 3. Update All Test Cases

**Files to Update**:
- `TEST_CASES.md` - All change block examples
- `task-1.5-change-commands.md` - Change examples
- `SIMPLIFIED_CHANGE_WORKFLOW.md` - Change examples
- `PARALLEL_CHANGES.md` - Change examples

**Change**: Use colon syntax for metadata:
```sruja
metadata {
  owner: "alice@example.com"
  stakeholders: ["bob@example.com"]
}
```

### 4. Update All Documentation Examples

**Files to Check**:
- All task documents with metadata examples
- All test case documents
- All workflow documents

**Change**: Ensure all metadata uses colon syntax consistently.

## Syntax Reference (Corrected)

### Change Block

```sruja
change "<change-id>" {
  version "<version>"
  requirement "<requirement-id>"  // Optional
  adr "<adr-id>"  // Optional
  status "<status>"  // pending, in-progress, approved, deferred
  
  metadata {
    owner: "<owner>"
    stakeholders: ["<stakeholder1>", "<stakeholder2>", ...]
  }
  
  add {
    // Architecture elements
  }
  
  modify {
    // Architecture elements
  }
  
  remove {
    // Architecture elements
  }
}
```

### Metadata Block

```sruja
metadata {
  <key>: <value>
  <key>: <value>
  <key>: [<value1>, <value2>, ...]  // Arrays
}
```

### ADR Block

```sruja
adr "<id>" "<title>" {
  tags [<refs>]
  status "<status>"  // pending, in-progress, decided, rejected
  context "<context>"
  decision "<decision>"
  consequences ["<consequence1>", ...]
}
```

### Snapshot Block

```sruja
snapshot "<snapshot-name>" {
  version "<version>"
  description "<description>"
  timestamp "<iso-timestamp>"
  preview true  // Optional, for preview snapshots
  changes ["<change-id1>", "<change-id2>", ...]  // Optional, for preview snapshots
  
  architecture "<name>" {
    // Full architecture definition
  }
}
```

## Verification Checklist

- [x] Metadata syntax uses colon (`:`) consistently - **FIXED**
- [ ] Array syntax support in parser - **REQUIRES PARSER UPDATE**
- [x] String values are quoted consistently - **VERIFIED**
- [ ] Change block syntax matches parser expectations - **REQUIRES PARSER UPDATE**
- [x] ADR syntax matches parser expectations - **VERIFIED**
- [ ] Snapshot block syntax is defined - **DOCUMENTED**
- [x] All test cases use correct syntax - **FIXED**
- [x] All documentation examples use correct syntax - **FIXED**

## Summary of Fixes Applied

1. ✅ Updated `DSL_CHANGES_REQUIRED.md` - All metadata examples use colon syntax
2. ✅ Updated `task-1.5-change-commands.md` - Metadata examples fixed
3. ✅ Updated `TEST_CASES.md` - All 13 metadata blocks fixed
4. ✅ Updated `SIMPLIFIED_CHANGE_WORKFLOW.md` - Metadata example fixed
5. ✅ Updated `PARALLEL_CHANGES.md` - Metadata example fixed
6. ✅ Created `DSL_SYNTAX_CONSISTENCY.md` - Comprehensive syntax reference

## Next Steps (Implementation)

1. **CRITICAL**: Extend parser to support array values in metadata (for `stakeholders`)
2. Implement change block parser
3. Implement snapshot block parser
4. Add syntax validation tests
5. Update printer to handle array metadata values


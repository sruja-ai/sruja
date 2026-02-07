# DSL Syntax Inconsistencies Analysis

## Summary

Analysis of systems thinking DSL syntax across examples, documentation, and parser implementation.

---

## 1. Element Definition Syntax

### Consistent Patterns ✓

```sruja
// Basic element definition
Name = kind "Title"

// With nested body
Name = kind "Title" {
  // body items
}

// With metadata
Name = kind "Title" {
  metadata {
    key "value"
    tags ["tag1", "tag2"]
  }
}

// With technology
Name = kind "Title" {
  technology "Technology"
}
```

### Issues Found

- **Nested components in tutorial**: `examples/tutorial/02-add-containers.sruja` shows component inside container but the element type is `component`, not a separate kind definition

---

## 2. Relation Syntax

### Supported Syntax ✓

```sruja
// Basic relation
From -> To

// With label
From -> To "Label"

// With label and technology
From -> To "Label" technology "Tech"

// With tags (array syntax)
From -> To "Label" [tag1, tag2]
```

### Implementation Details

- Parser: `crates/sruja-language/src/parser.rs:1592-1621`
- AST: `Relation` struct has `tags: Vec<String>` field
- Tag array parser: `parse_tag_array` handles `[tag1, tag2, ...]` syntax

### Issues Found

1. **Relation tags in examples**: `examples/demo_tags.sruja` uses `[tag1, tag2]` which IS supported
2. **Validation issue**: Validation rule warns about `app` not being referenced when relations use `app.api` (FQN)

---

## 3. Tags Syntax - TWO DIFFERENT WAYS

### A. Hash Prefix Tags (on element definition)

```sruja
// Not currently used in examples
Name = kind "Title" #tag1 #tag2 {
  // body
}
```

**Implementation**:

- Parser: `parse_tag_ref` handles `#Ident` format (line 1698)
- AST: `ElementAssignment.tag_refs: Vec<String>` stores these
- Stored with `#` prefix (line 1701: `format!("#{}", ident)`)

### B. Array Tags (in metadata block)

```sruja
// Widely used in examples
Name = kind "Title" {
  metadata {
    tags ["tag1", "tag2"]
  }
}
```

**Implementation**:

- Parser: `parse_metadata_block` and `parse_metadata_entry` handle this (lines 1669-1695)
- AST: `ElementDefBody.metadata: Vec<MetaEntry>` stores these
- Stored as key-value pairs with key="tags" and value=string array

### C. Array Tags (on relations)

```sruja
From -> To "Label" [tag1, tag2]
```

**Implementation**:

- Parser: `parse_tag_array` handles this (lines 1705-1711)
- AST: `Relation.tags: Vec<String>` stores these directly

### Inconsistency ⚠️

**Three different tag syntaxes exist for three different contexts:**

1. `#tag1 #tag2` - on element definition line (rarely used)
2. `tags ["tag1", "tag2"]` - in metadata block (common)
3. `[tag1, tag2]` - on relations (common in one example)

These are **functionally different** but serve similar purposes.

---

## 4. View Syntax

### Consistent Pattern ✓

```sruja
// Basic view
view index {
  include *
}

// With title and description
view api {
  title "API View"
  description "Shows API"
  include Shop.API Shop.DB
}

// With wildcard
view all {
  include Shop.*
}

// With exclude
view no_db {
  include Shop.*
  exclude Shop.DB
}

// Scoped view
view containers of Shop {
  include Shop.WebApp Shop.API Shop.DB
}
```

### Implementation

- Parser: `parse_view` (line 1357)
- AST: `ViewDef` struct with fields: id, title, description, view_of, tags, rules
- View body: `parse_view_body` (line 1433) currently **consumes but doesn't extract fields**

### Issues Found ⚠️

1. **View body parsing incomplete**: `parse_view_body` uses brace-counting to find matching closing brace but doesn't actually extract fields like title, include, exclude
2. **View field extraction**: The function extracts from `body_fields` but only parses title, include, exclude, description (lines 1378-1397)
3. **Array syntax**: Include/exclude can use:
   - Wildcard: `*`
   - Space-separated: `Shop.API Shop.DB`
   - Array: `[Shop.API, Shop.DB]` (in comments but not documented)

### Documentation vs Implementation Gap

**Documentation shows**: Space-separated includes without quotes
**Parser expects**: Either string value or parsed from body

---

## 5. Governance Syntax (Requirements, ADRs, Policies)

### Consistent Patterns ✓

#### Requirements

```sruja
// Assignment syntax (recommended in examples)
R1 = requirement functional "System must process orders"
R2 = requirement performance "API must respond in under 200ms"
R3 = requirement security "All data must be encrypted"

// Direct syntax
requirement R1 functional "Title"
```

#### ADRs

```sruja
// Assignment syntax (recommended)
ADR1 = adr "Use PostgreSQL" {
  status "Accepted"
  context "Need ACID transactions"
  decision "Use PostgreSQL"
  consequences "Strong consistency, SQL complexity"
}
```

#### Policies

```sruja
// Assignment syntax
P1 = policy "Security Policy" {
  category "security"
  enforcement "required"
  description "All API calls must use TLS 1.3"
}
```

### Issues Found

- No inconsistencies found between parser, AST, examples, and docs

---

## 6. Scenario/Flow Syntax

### Consistent Pattern ✓

```sruja
// Scenario
Checkout = scenario "Checkout Process" {
  step user -> shop.webApp "starts checkout"
  step shop.webApp -> shop.api "validates"
  step shop.api -> paymentGateway "charges"
}

// Flow
OrderFlow = flow "Order Processing" {
  step user -> shop.webApp "browses"
  step shop.webApp -> shop.api "calls"
  step shop.api -> shop.db "saves"
}
```

### Implementation

- Parser: `parse_scenario` and `parse_flow` (lines 1103-1217)
- AST: `Scenario` and `Flow` structs are identical
- Step keyword: Optional in both (line 1137: `opt(preceded(tag("step"), ws1))`)

### Issues Found

- No inconsistencies found

---

## 7. Overview Block Syntax

### Pattern

```sruja
overview {
  summary "System description"
  audience "Target audience"
  scope "System scope"
  goals ["goal1", "goal2"]
  nonGoals ["not in scope"]
  risks ["risk1", "risk2"]
}
```

### Issues Found ⚠️

1. **Parser consumes block only**: `parse_overview_block` uses brace-counting to consume entire block but doesn't extract any fields (lines 1629-1685)
2. **AST fields unused**: `OverviewBlock` struct has fields but they're never populated (lines 378-388)
3. **No field extraction**: Overview fields are not parsed from DSL text

---

## 8. Reference Errors in Examples

### Files with Issues

#### examples/tutorial/02-add-containers.sruja

```
ERROR: Reference 'app.backend' in relation does not exist
```

**Issue**: Element is named `backend` not `app.backend` (line 21)
**Actual name**: `app.backend` should work if backend is defined

#### examples/course/ecommerce.sruja

```
ERROR: Reference 'productController' in relation does not exist
```

**Issue**: `productController` IS defined (line 42) but validation says it doesn't exist
**Possible cause**: Scoping issue - it's nested inside `ecommerce.api` container

#### examples/demo_views_customization.sruja

```
ERROR: Reference 'analyticsPlatform.dataWarehouse' does not exist
ERROR: Reference 'analyticsPlatform.cache' does not exist
ERROR: Reference 'analyticsPlatform.eventStream' does not exist
ERROR: Reference 'analyticsPlatform.taskQueue' does not exist
```

**Issue**: These elements ARE defined in the file (lines 85-119)
**Possible cause**: File uses `analyticsPlatform` system with nested elements, but references use full FQNs

---

## 9. Metadata Syntax - Inconsistencies

### Pattern A: Simple Key-Value

```sruja
metadata {
  team "Engineering"
  tier "critical"
  owner "team@example.com"
}
```

### Pattern B: Array Values

```sruja
metadata {
  tags ["REQ001", "external"]
  goals ["goal1", "goal2"]
}
```

### Pattern C: String Quoted (same as A)

```sruja
metadata {
  description "Some description"
}
```

### Issues Found ⚠️

1. **Inconsistent value types**: Some keys have string values, some have arrays
2. **Parser limitation**: `parse_metadata_entry` only parses optional string values (line 1692)
3. **Array values not parsed**: Array syntax in metadata blocks appears in examples but parser doesn't handle it explicitly

---

## 10. Element Scoping - Validation Gap

### Expected Behavior

```sruja
app = system "App" {
  api = container "API"
  db = container "Database"
}

// These should all work:
app.api -> app.db "queries"      // Full FQN
api -> db "queries"               // Relative (in scope)
```

### Current Behavior

- Parser supports: Full FQN syntax `app.api`
- Validation reports: `app` not referenced when relations use FQNs
- Scoping logic: `resolve_relation_fqns` in traversal.rs tries to resolve scope (lines 172-218)

### Issues Found ⚠️

1. **Validation doesn't account for FQN usage**: When relations use `app.api`, validator should know this references `app` indirectly
2. **Scope resolution may be incomplete**: Nested element resolution has multiple fallback paths but may have edge cases

---

## Priority Fixes Needed

### HIGH PRIORITY

1. **Fix overview block parsing**: Extract overview fields instead of just consuming the block
2. **Fix view body parsing**: Extract include/exclude/title fields from view body
3. **Fix metadata array values**: Parse array syntax for metadata values

### MEDIUM PRIORITY

4. **Fix validation for FQN references**: Update orphan/warning rules to account for elements being referenced via FQNs
5. **Fix scoping validation**: Ensure nested elements resolve correctly in all contexts

### LOW PRIORITY

6. **Unify tag syntax**: Consider making tag syntax more consistent across contexts
7. **Update examples**: Fix files with actual reference errors (ecommerce.sruja, demo_views_customization.sruja)

---

## Files Requiring Updates

### Parser Changes

- `crates/sruja-language/src/parser.rs`:
  - Fix `parse_overview_block` to extract fields
  - Fix `parse_view_body` to extract include/exclude properly
  - Add array value parsing for metadata entries

### AST Changes (maybe)

- `crates/sruja-language/src/ast.rs`:
  - Verify `OverviewBlock` fields are used
  - Verify `ViewDef` rules are populated correctly

### Example Files to Fix

- `examples/course/ecommerce.sruja`
- `examples/demo_views_customization.sruja`
- `examples/tutorial/02-add-containers.sruja` (verify the issue)

### Validation Rule Updates

- `crates/sruja-engine/src/rules/orphan.rs`: Account for FQN references
- `crates/sruja-engine/src/rules/valid_ref.rs`: Better error messages for scoping issues

---

## Test Files to Update

- `crates/sruja-language/tests/example_files.rs`: Add tests for overview and view syntax
- `crates/sruja-engine/tests/`: Add tests for FQN reference validation

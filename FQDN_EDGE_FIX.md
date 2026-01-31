# FQDN Edge Fix - Summary

## Problem Statement

Rust parsers were not handling edges (relations) properly when using FQDN (Fully Qualified Domain Name) conventions. Specifically, when elements were defined inside parent bodies (nested elements) and relations used simple names, the edges were not appearing in diagrams.

### Example Scenario

```rust
Backend = system "Backend System" {
  API = container "API Service"
  DB = container "Database"
  
  API -> DB "reads/writes"  // Simple names inside body
}
```

In this example:
- Elements are stored with FQDNs: `Backend.API`, `Backend.DB`
- The relation uses simple names: `API` -> `DB`
- The edge exporters couldn't find elements with keys `API` and `DB`
- Result: Edge was not rendered in the diagram

## Root Causes

1. **Missing FQDN Resolution**: Relations defined inside element bodies used simple names, but elements were stored with FQDNs. The parser collected relations without resolving the endpoint names to their fully qualified identifiers.

2. **Incorrect View Projection**: In `sruja-export`, the `project_id` function for level 3 (show all) was incorrectly projecting nested elements to their root, causing same-cluster edges to be filtered out.

## Solution

### 1. Two-Pass Element Collection (sruja-language)

Modified `collect_elements` in `sruja/crates/sruja-language/src/traversal.rs` to use a two-pass approach:

- **Pass 1**: Collect all elements with their FQDNs
- **Pass 2**: Resolve relation endpoints to FQDNs based on their scope

Added `resolve_relation_fqns` function that:
- Checks if a name already exists as a fully qualified name
- Checks if the name exists in the elements map
- Prepends the scope if neither match, and checks if the scoped name exists
- Returns the original name if no match is found (for external references)

### 2. Fixed View Projection (sruja-export)

Modified `project_id` in `sruja/crates/sruja-export/src/dot/exporter.rs` to correctly handle level 3 (show all):

- For level 3 with no focus, return the FQN directly instead of projecting to root
- This preserves the full detail of nested elements and their relationships

### 3. Fixed Validation Tests (sruja-engine)

Updated validation tests in `sruja/crates/sruja-engine/src/validator.rs` to:
- Use correct error codes (E201, E202 instead of CODE_DUPLICATE_ID, CODE_UNDEFINED_REF)
- Fix test syntax to use assignment format (A = system vs system A)
- Simplified the validator module structure

## Files Modified

### sruja/crates/sruja-language/src/traversal.rs
- Modified `collect_elements` to use two-pass approach
- Added `resolve_relation_fqns` function for FQDN resolution
- Updated `collect_all_relations` to use resolved relations

### sruja/crates/sruja-language/src/lib.rs
- Added comprehensive test `test_fqdn_edge_resolution` to verify FQDN edge resolution

### sruja/crates/sruja-export/src/dot/exporter.rs
- Fixed `project_id` function for level 3 to return FQN when no focus
- Added integration test `test_fqdn_edge_resolution_nested_elements`

### sruja/crates/sruja-export/src/dot/edge_rendering_test.rs
- Added comprehensive integration test for FQDN edge resolution

### sruja/crates/sruja-engine/src/validator.rs
- Fixed UniqueIdRule to check for duplicates before deduplication
- Added helper function `check_nested_elements` for recursive duplicate checking
- Fixed test syntax and error codes

### sruja/crates/sruja-engine/src/rules/unique_id.rs
- Modified to check element definitions directly instead of using deduplicated FQDN map
- Added recursive checking for nested elements

### sruja/crates/sruja-engine/src/lib.rs
- Simplified exports and fixed module structure

## Test Coverage

### Unit Tests
- `test_fqdn_edge_resolution`: Verifies that relations with simple names are resolved to FQDNs
- `test_cross_cluster_edge_attributes`: Ensures cross-cluster edges have correct attributes
- `test_same_cluster_edge_no_attributes`: Ensures same-cluster edges don't have attributes
- `test_fqdn_edge_resolution_nested_elements`: Integration test for nested elements

### Validation Tests
- `test_unique_id_rule`: Verifies duplicate ID detection works with nested elements
- `test_valid_ref_rule`: Verifies invalid reference detection works

## Example Usage

After the fix, both patterns work correctly:

### Pattern 1: Simple Names (Inside Body)
```rust
Backend = system "Backend System" {
  API = container "API Service"
  DB = container "Database"
  
  API -> DB "reads/writes"  // Automatically resolved to Backend.API -> Backend.DB
}
```

### Pattern 2: FQDN Names (Cross-System)
```rust
Frontend = system "Frontend Application" {
  Web = container "Web App"
}

Frontend.Web -> Backend.API "calls API"  // Already uses FQDNs
```

## Backward Compatibility

The changes are fully backward compatible:
- Existing code using FQDN notation continues to work
- New code using simple names inside bodies now works correctly
- The resolution logic falls back to the original name if no match is found

## Performance Impact

Minimal performance impact:
- Two-pass approach adds a single iteration through the collected elements
- Resolution is O(1) per relation endpoint using HashMap lookups
- No impact on existing parsing performance

## Future Enhancements

Potential future improvements:
1. Add warnings for ambiguous references
2. Support relative path resolution (e.g., `..` for parent)
3. Add validation to catch unresolved references before export
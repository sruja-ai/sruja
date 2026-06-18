# Task: Rename LearningEntry fields for clarity

## Problem

The `LearningEntry` struct in `crates/sruja-graph/src/learning.rs` uses terse field names that are ambiguous without doc comments:
- `context` - unclear if this is the situation, the environment, or the scope
- `hypothesis` - could be confused with a test hypothesis
- `reason` - unclear if this is the failure reason or the rationale
- `guardrail_advice` - verbose and unclear

## Requirements

1. Rename the following fields:
   - `context` -> `situation` (clearer that it describes the situation when the learning was recorded)
   - `reason` -> `failure_reason` (clearer that it explains why the experiment failed)
   - `guardrail_advice` -> `suggested_behavior` (clearer that it describes what to do instead)
2. Add `#[serde(rename = "old_name")]` attributes for backward-compatible deserialization
3. Update all files that reference these fields
4. Run `cargo test --workspace` to verify

## Files to Modify

1. `crates/sruja-graph/src/learning.rs` - Update struct definition with renames
2. `crates/sruja-agent/src/memory/mod.rs` - Update field references
3. `crates/sruja-agent/src/cognition/mod.rs` - Update field references
4. `crates/sruja-agent/src/matts.rs` - Update field references
5. `crates/sruja-agent/src/memory/curation.rs` - Update field references

## Verification

The code should compile and all existing tests should pass. The serde rename attributes should allow existing JSON files to be deserialized correctly.

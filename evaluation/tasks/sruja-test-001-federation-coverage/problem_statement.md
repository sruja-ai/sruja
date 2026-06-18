# Task: Add unit tests for federation.rs

## Problem

`crates/sruja-cli/src/commands/federation.rs` is the largest CLI command file (983 lines) with zero test modules. This lack of test coverage makes it risky to refactor or extend.

## Requirements

1. Add unit tests for the following functions:
   - `infer_repo_id(repo_path: &Path) -> String` - Test with tempdir that has `.git` directory
   - `git_commit_short(repo_path: &Path) -> Option<String>` - Test with tempdir
   - `publish()` - Test with a minimal repo structure
2. Tests should use `tempfile::tempdir()` for isolation
3. Tests should follow the existing pattern in sibling files (e.g., `density.rs`, `learn.rs`)

## Files to Modify

1. `crates/sruja-cli/src/commands/federation.rs` - Add `#[cfg(test)] mod tests` block

## Verification

All new tests should pass. Existing functionality should not be affected.

# Task: Add --timeout flag to verify-task subcommand

## Problem

The `verify-task` subcommand runs verification steps with a default timeout of 60 seconds per step (defined in `VerifyOptions::default()`). Users cannot override this timeout from the CLI, which is problematic for:
- Large repositories where `cargo clippy` may take longer than 60 seconds
- CI environments with different performance characteristics
- Debugging timeout issues

## Requirements

1. Add a `--timeout` CLI flag to the `verify-task` subcommand that accepts milliseconds
2. Thread the timeout value through to `VerifyOptions::timeout_ms`
3. Default value should remain 60000ms (60 seconds) for backward compatibility

## Files to Modify

1. `crates/sruja-cli/src/cli/commands.rs` - Add `timeout` field to `VerifyTask` command variant
2. `crates/sruja-cli/src/cli/run.rs` - Pass timeout to `VerifyTaskOptions`
3. `crates/sruja-cli/src/commands/verify_task.rs` - Add timeout to `VerifyTaskOptions` struct and use it when creating `VerifyOptions`

## Verification

The flag should be accepted and the timeout value should be passed through to the verification engine. Existing tests should continue to pass.

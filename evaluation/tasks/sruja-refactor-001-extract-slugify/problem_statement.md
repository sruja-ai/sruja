# Task: Extract duplicated slugify function

## Problem

Three independent `slugify` functions exist in the codebase:
1. `crates/sruja-agent/src/cognition/decision.rs:191`
2. `crates/sruja-agent/src/cognition/runbook.rs:211`
3. `crates/sruja-cli/src/commands/agent_plan.rs:22`

The first two are identical implementations. The third is a slightly different implementation with different behavior (handles underscores/dots differently, takes 40 chars instead of 50).

This duplication creates maintenance risk and inconsistent behavior.

## Requirements

1. Identify which version is canonical (or write a unified one that covers all use cases)
2. Move the unified function to a shared location accessible by both crates
3. Update all three call sites to use the shared function
4. Add tests for the unified function covering edge cases from both variants

## Files to Modify

1. `crates/sruja-agent/src/cognition/decision.rs` - Remove local `slugify` function
2. `crates/sruja-agent/src/cognition/runbook.rs` - Remove local `slugify` function
3. `crates/sruja-cli/src/commands/agent_plan.rs` - Remove local `slugify` function
4. Add a shared utility module (likely in `sruja-agent/src/utils.rs` or similar)

## Verification

The unified function should handle all edge cases from both variants. Existing tests should continue to pass.

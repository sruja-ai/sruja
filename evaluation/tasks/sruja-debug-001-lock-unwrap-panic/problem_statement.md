# Task: Replace lock().unwrap() with proper error handling

## Problem

The agent crate uses `Mutex::lock().unwrap()` in production code. If a mutex is poisoned (e.g., due to a panic in another thread holding the lock), the `.unwrap()` call will panic and crash the entire agent loop mid-execution.

## Stacktrace

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: PoisonError { .. }'
crates/sruja-agent/src/cognition/mod.rs:986
```

## Requirements

1. Replace `.lock().unwrap()` with proper error handling in production code
2. Use `.lock().map_err(|e| ...)? ` or `.lock().unwrap_or_else(|e| e.into_inner())` depending on whether poisoning recovery is appropriate
3. Focus on the most critical sites first:
   - `cognition/mod.rs:986` (reflect method)
   - `llm/router.rs:149,177` (budget tracking)

## Files to Modify

1. `crates/sruja-agent/src/cognition/mod.rs` - Fix mutex handling in reflect method
2. `crates/sruja-agent/src/llm/router.rs` - Fix mutex handling in budget tracking

## Verification

The agent should not panic on mutex poisoning. Instead, it should either:
- Return an error (for critical operations)
- Recover gracefully (for non-critical operations like budget tracking)

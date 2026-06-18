# Bug: `all_passed` returns true when every verification step is Skipped

## Summary

The `all_passed()` function in `crates/sruja-agent/src/verify/mod.rs` returns `true` when every
verification step has status `Skipped`. This means a misconfigured allowlist (or an empty one)
silently passes verification, allowing the agent to converge without actually running any checks.

## Reproduction

```rust
let results = vec![
    VerifyResult { step_id: "a".into(), status: VerifyStatus::Skipped, .. },
    VerifyResult { step_id: "b".into(), status: VerifyStatus::Skipped, .. },
];
assert!(all_passed(&results)); // BUG: should be false
```

## Root cause

`all_passed()` delegates to `VerifyStatus::is_pass()` which treats both `Ok` and `Skipped`
as passing. This was an intentional design choice (Skipped = "not applicable") but creates a
security hole: the agent can bypass the deterministic grader by misconfiguring the allowlist.

## Impact

- **Severity**: Security boundary bypass
- **Affected code**: `crates/sruja-agent/src/verify/mod.rs:178-181`
- **Exploit**: If the agent's `shell_allowlist` doesn't include the verify step commands,
  all steps are Skipped, `all_passed` returns true, and the loop converges.

## Expected behavior

`all_passed()` should return `false` when every step is Skipped. Only `Ok` results should
count as passing. Mixed Skipped+Ok should also fail (the allowlist mismatch means at least
one step could not run).

## Fix direction

Change `all_passed()` to check for `VerifyStatus::Ok` directly instead of using `is_pass()`:

```rust
pub fn all_passed(results: &[VerifyResult]) -> bool {
    !results.is_empty()
        && results.iter().all(|r| matches!(r.status, VerifyStatus::Ok))
}
```

Add unit tests:
1. `verify_skipped_is_not_pass_when_allowlist_empty` — all Skipped → false
2. `verify_skipped_with_ok_passes` — mixed Skipped+Ok → false

# Enterprise policy: baselines and exceptions

This doc defines a pragmatic enterprise path for CI enforcement without “first-run bricks”.

## Problem: first-run CI fails on legacy drift

When you first enable drift checks in CI, existing violations can be numerous. Blocking all merges until you fix them is usually not viable.

## Approach: advisory → baseline → enforce

1. **Advisory rollout** (no merge blocking):
   - Use `sruja drift-pr` in CI (fails only on **new** violations introduced by the PR).
2. **Baseline legacy violations**:
   - Generate a baseline file once, commit it, and review it like code.
3. **Enforce**:
   - Switch CI to `sruja drift --ci --baseline .sruja/violations.baseline.json` (or use `verify-task` which calls drift/check).

## Baseline file format

Default path: `.sruja/violations.baseline.json`

Schema version 2 (current):

```json
{
  "schema_version": 2,
  "generated_at_unix": 0,
  "violations": [
    {
      "fingerprint": "layer-violation|Foo -> Bar|Layer violation: 'Foo' directly accesses 'Bar'",
      "reason": "legacy, tracked in ARCH-123",
      "expires": "2026-08-01"
    }
  ]
}
```

Notes:
- **`fingerprint`** is stable and derived from `(kind, location, message)`; it is the suppression key.
- **`reason`** is required so the baseline stays reviewable.
- **`expires`** is optional but recommended for time-bounded exceptions.

Backwards compatibility:
- Schema version 1 baselines (with only `fingerprints: []`) are still accepted.

## Commands

- **Write baseline**:
  - `sruja baseline -r . -o .sruja/violations.baseline.json`
- **Use baseline in CI drift**:
  - `sruja drift --ci -r . --baseline .sruja/violations.baseline.json`

## CI recommendations

- Prefer `drift-pr` during rollout; it enforces “no new debt” with minimal friction.
- When switching to full enforcement, use a baseline to suppress legacy violations while fixing them incrementally.


# Bug: `agent run` step differentiation collapses all intent violations to one command

## Problem

When `sruja agent run` is used with `--mode plan --format json`, all four distinct intent violation kinds (MissingComponent, TaxonomyMismatch, MissingRelationship, etc.) produce steps that all run `sruja intent check -r .` — even when `step_for_violation` had already produced a tailored step (`propose create`, `review`, `impact`, `compliance`).

## Root Cause

`remediation.rs` function `step_for_intent_violation` unconditionally overwrites `step.argv` to `sruja intent check` for every intent violation, even when `step_for_violation` had already produced a tailored step.

## Expected Behavior

- Specific steps (`propose`, `review`, `impact`, `compliance`) should preserve their tailored argv
- Only generic drift fallbacks (`step_drift_kind_unknown`, `step_drift_kind_*`) should be rewritten to `sruja intent check`

## Reproduction

```bash
sruja agent run --goal "Fix clippy warnings in eval.rs" --file crates/sruja-cli/src/commands/eval.rs --mode plan --format json
```

All 4 steps will show `sruja intent check -r .` instead of diverse commands.

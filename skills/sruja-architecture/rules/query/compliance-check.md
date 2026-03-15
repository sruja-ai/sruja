# compliance-check

## Why It Matters

Teams and regulators need to know whether the codebase and declared architecture meet architectural rules and policies. Compliance checks provide a deterministic, machine-readable status and remediation path without auto-applying changes.

## When to Apply

- User asks: "Are we within architectural rules?" or "What violations do we have?"
- CI/CD gate: fail the build or block merge when non-compliant
- Audit or governance: produce a report (status, violations, remediation)

## Correct Approach

1. **Run compliance.** Execute `sruja compliance -r . -a repo.sruja -f json` (or omit `-a` for structural-only). The output includes:
   - `status` (e.g. compliant / non-compliant)
   - `health_score`
   - `drift_entries`, `policy_violations`, `remediation_checklist`

2. **Optional: validate with policy files.** If the team uses policy files, run `sruja validate repo.sruja --policy <path>` and report any violations.

3. **Present findings.** Summarize for the user: status, key violations, and suggested remediation. Do not apply fixes automatically; let the user or team decide.

4. **CI usage.** In pipelines, use `sruja compliance -r . -f json` and check exit code (non-zero when non-compliant) or parse JSON to fail on specific violation types. Use `sruja drift-pr` for PR-scoped checks (new violations only).

## Incorrect Approach

- Auto-applying remediation without user or pipeline approval
- Ignoring exit codes in CI (compliance exits non-zero when non-compliant)
- Reporting compliance as "unknown" when the CLI can be run to get a definite status

## Summary

**Compliance check: run `sruja compliance` (and optionally `sruja validate --policy`); present status and remediation; do not auto-apply. Use exit code in CI for gates.**

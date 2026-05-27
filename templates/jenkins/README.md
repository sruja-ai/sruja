# Jenkins templates (Sruja)

This folder contains Jenkins pipeline examples for running Sruja as a deterministic CI gate.

## Recommended posture

- Use `sruja drift-pr` to enforce “no new violations” during rollout.
- Use `sruja verify-task --profile review` once the loop is stable.


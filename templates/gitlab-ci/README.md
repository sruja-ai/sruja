# GitLab CI templates (Sruja)

These templates mirror the GitHub Actions templates under `templates/github-actions/` but are designed for GitLab CI/CD.

## Recommended usage

- For low-friction adoption: use **MR-scoped drift** (`drift-pr`) to fail only on new violations.
- For stronger guarantees: add `verify-task` (review profile) and optionally enable evidence packs.

## Templates

- `sruja-verify-task-mr.yml` — run `sruja verify-task` on merge requests
- `sruja-drift-mr.yml` — run `sruja drift-pr` on merge requests


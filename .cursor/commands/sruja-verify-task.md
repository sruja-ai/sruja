# Sruja: Verify task

Run verification steps for a task profile. Use before marking a task as done.

**Related:** [plan-feature](./sruja-plan-feature.md) → [implement-from-plan](./sruja-implement-from-plan.md) → **verify-task** → [reflect-on-run](./sruja-reflect-on-run.md)

## Usage

```bash
# Default: coding profile
sruja verify-task --profile coding -r .

# Bugfix targeting a specific file
sruja verify-task --profile bugfix --file <path> -r .

# Pre-merge review
sruja verify-task --profile review -r .

# Architecture change
sruja verify-task --profile arch -r .

# JSON output for CI/MCP
sruja verify-task --profile coding -r . -f json
```

## Steps

1. **Choose the right profile** based on the task type:
   - `coding` — New features, refactors (default)
   - `bugfix` — Bug fixes (use `--file` to target)
   - `review` — Pre-merge review
   - `arch` — Architecture/DSL changes

2. **Run verification:**
   ```bash
   sruja verify-task --profile <profile> -r .
   ```

3. **If all passed:** Task is verified. Optionally record an affirmation learning.

4. **If any failed:**
   - Review the failed step output
   - Fix the issue
   - Re-run verify-task
   - Record a correction learning:
     ```bash
     sruja agent record -r . \
       -c "<task context>" \
       -H "<what was tried>" \
       -o failed \
       -g "<correction for next time>"
     ```

## Profiles

| Profile | Steps |
|---------|-------|
| `coding` | `lint repo.sruja` + `make check` + `drift` |
| `bugfix` | `focus --file` + `make check` + `intent check` |
| `review` | `review -f json` + `intent check` + `drift` |
| `arch` | `lint repo.sruja` + `drift` + `intent check` + `review` |

## References

- [docs/HOST_AGENT_INTEGRATION.md](../../docs/HOST_AGENT_INTEGRATION.md) — Integration contract
- [AGENTS.md](../../AGENTS.md#build-lint-and-test-commands) — Build commands

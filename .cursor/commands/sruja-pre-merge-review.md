# Sruja: Pre-Merge Review

Structured review workflow before merging changes. Produces actionable feedback with architecture grounding.

**Related:** [plan-feature](./sruja-plan-feature.md) → [implement-from-plan](./sruja-implement-from-plan.md) → [verify-task](./sruja-verify-task.md) → [reflect-on-run](./sruja-reflect-on-run.md)

## Steps

1. **Run review with JSON output**
   ```bash
   sruja review -r . -f json
   ```
   This produces structured suggestions, violations, and health score.

2. **Run intent check**
   ```bash
   sruja intent check -r . -f json
   ```
   Verify changes match declared architectural intent.

3. **Run drift check**
   ```bash
   sruja drift -r . -f json
   ```
   Check for new architectural drift.

4. **Or use the combined verify-task review profile**
   ```bash
   sruja verify-task --profile review -r . -f json
   ```

5. **Parse and categorize findings**

   Output the review results in this structured format:

   ```
   ## Blockers (must fix before merge)
   - [ ] <severity> <finding>: <description>

   ## Suggestions (should fix)
   - [ ] <finding>: <description>

   ## Nits (optional)
   - [ ] <finding>: <description>
   ```

   **Blockers include:**
   - Lint errors in `repo.sruja`
   - New drift violations
   - Intent mismatches
   - Failed tests

   **Suggestions include:**
   - Review recommendations from `sruja review`
   - Architecture improvements
   - Missing documentation

   **Nits include:**
   - Style issues
   - Minor refactoring opportunities

6. **If blockers exist:**
   - Do not merge
   - Fix blockers
   - Re-run review

7. **If no blockers:**
   - Review suggestions and nits
   - Decide which to address now vs defer
   - Proceed with merge

## Example Output

```
## Blockers
- [ ] drift: New violation in AuthContainer -> Database (missing relationship)

## Suggestions
- [ ] Consider splitting AuthContainer into AuthAPI and AuthWorker
- [ ] Add description to new PaymentService component

## Nits
- [ ] Format: run `sruja fmt repo.sruja`
```

## References

- [docs/HOST_AGENT_INTEGRATION.md](../../docs/HOST_AGENT_INTEGRATION.md) — Integration contract
- [AGENTS.md](../../AGENTS.md) — Build and test commands

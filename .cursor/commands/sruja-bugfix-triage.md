# Sruja: Bugfix Triage

Systematic workflow for triaging and fixing bugs with architecture grounding.

**Related:** [plan-feature](./sruja-plan-feature.md) → [implement-from-plan](./sruja-implement-from-plan.md) → [verify-task](./sruja-verify-task.md) → [reflect-on-run](./sruja-reflect-on-run.md)

## Steps

1. **Reproduce the bug**
   - Understand the failure mode
   - Identify the affected file(s) and component(s)

2. **Get focus briefing**
   ```bash
   sruja focus --file <buggy-file> -r . -f for-ai
   ```
   This gives you the blast radius, decisions, and AI instructions for the target.

3. **Search memory for similar issues**
   ```bash
   sruja agent history -r . -f json
   # Or MCP: sruja_search_memory with the error message
   ```
   Check if this bug (or similar) has been encountered before.

4. **Form a hypothesis**
   - What component is broken?
   - What change would fix it?
   - What's the blast radius of the fix?

5. **Create a minimal fix plan**
   - Scope the change to the smallest possible diff
   - Identify which verification steps will validate the fix

6. **Implement the fix**
   - Make the minimal change
   - Do not refactor unrelated code

7. **Verify with bugfix profile**
   ```bash
   sruja verify-task --profile bugfix --file <buggy-file> -r .
   ```

8. **If verify fails:**
   - Review the failed step
   - Fix and re-verify
   - Record a correction learning

9. **If verify passes:**
   - Optionally record an affirmation learning
   - Mark the bug as fixed

## Example

```bash
# Bug: auth token validation fails on edge cases
sruja focus --file src/auth/token.rs -r . -f for-ai
sruja agent history -r . -f json  # Check for similar auth bugs
# ... implement fix ...
sruja verify-task --profile bugfix --file src/auth/token.rs -r .
```

## References

- [docs/HOST_AGENT_INTEGRATION.md](../../docs/HOST_AGENT_INTEGRATION.md) — Run envelope
- [AGENTS.md](../../AGENTS.md) — Build and test commands

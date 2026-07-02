# validate-changes

## Why It Matters

Validation catches errors early and ensures changes conform to sruja standards and project rules.

## When to Apply

- After any change to `.sruja` files
- Before committing architecture changes
- After AI-generated edits
- Before creating a pull request

## Correct Approach

1. **Lint .sruja files**:
   ```bash
   sruja lint repo.sruja
   ```

2. **Fix errors**:
   - E201: Invalid kind
   - E204: Circular dependencies
   - E205: Orphan components
   - E206: Invalid references

3. **Check drift**:
   ```bash
   sruja sync -r .
   sruja drift -r .
   ```

4. **Verify classification**:
   ```bash
   sruja classify -r .
   sruja drift -r . -a repo.sruja
   ```

## Incorrect Approach

- Skipping validation after changes
- Leaving errors for "later"
- Assuming previous validation still applies

## Summary

**Validate: lint → fix errors → drift check → verify classification.**

# validate-after-every-iteration

## Why It Matters

AI code editors (e.g. Cursor, Copilot) apply changes in iterations. If validation is not run after each iteration, errors can accumulate and the final result may not conform to Sruja standards or pass `sruja lint`. Running validation after every iteration keeps the file valid and aligned with standards defined in .sruja (e.g. CodeStyle, NoCycles, governance).

## When to Apply

- After every AI-generated or AI-edited change to a `.sruja` file
- After applying an inline suggestion or a multi-edit from the AI
- Before starting the next iteration of edits

## Correct Approach

1. **In VS Code / Cursor**
   After each code iteration on a `.sruja` file:
   - Run the command **Sruja: Run validation (check after AI/edit)** (Command Palette or right-click → "Sruja: Run validation…"), **or**
   - Save the file (validation runs on save).

2. **Result**
   The editor runs `sruja lint` on the current document (including unsaved content), updates the Problems panel and inline diagnostics, and shows a brief status (e.g. "no issues" or "N errors, M warnings").

3. **Fix any reported issues** before the next iteration.

## Incorrect Approach

- Editing `.sruja` multiple times without running validation or saving between iterations.
- Assuming the last run of validation still applies after further edits.

## Summary

**After every code iteration on a .sruja file, run Sruja: Run validation (check after AI/edit) or save the file so the editor checks the file against Sruja standards.**
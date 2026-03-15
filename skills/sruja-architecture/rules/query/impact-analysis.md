# impact-analysis

## Why It Matters

Before changing or removing an element (e.g. a service or database), developers need to know what depends on it and what it depends on. Impact analysis reduces risk and speeds up refactoring decisions.

## When to Apply

- User asks: "What breaks if I change X?" or "What is affected if I remove Y?"
- Before refactoring, deprecating, or replacing a component
- When evaluating the cost of a change for planning or ADRs

## Correct Approach

1. **Identify the element.** Use the element ID as it appears in repo.sruja (e.g. `AuthService`, `Application.Database`).

2. **Run the CLI.** Execute `sruja explain <element_id> --file repo.sruja`. The output includes:
   - Element description
   - Incoming relations (who depends on this element)
   - Outgoing relations (what this element depends on)

3. **Optional: full hierarchy.** Run `sruja tree repo.sruja` to see the full architecture tree. Note: `tree` takes a **file path**, not an element ID; there is no `--depth` or element-scoped tree in the current CLI.

4. **Summarize for the user.** Report:
   - Dependents (incoming): components that would be affected if this element is removed or changed
   - Dependencies (outgoing): components this element relies on
   - Any structural concerns (e.g. if it is in a cycle, mention that)

Use only data from the CLI and DSL; do not invent dependencies.

## CLI Notes

- `sruja explain <id> --file repo.sruja` — correct. Use `--file` if the baseline is not architecture.sruja.
- `sruja tree repo.sruja` — correct (tree accepts a file path).
- `sruja tree <element> --depth 3` — **not supported**; do not document or use.

## Incorrect Approach

- Using unsupported flags or syntax (e.g. tree with element or depth)
- Fabricating dependency data not present in explain output or the DSL
- Applying changes to the architecture without the user asking for updates

## Summary

**Impact analysis: run `sruja explain <element_id> --file repo.sruja`, optionally `sruja tree repo.sruja`; summarize dependents and dependencies from evidence only.**

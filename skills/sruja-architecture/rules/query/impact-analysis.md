# impact-analysis

## Why It Matters

Before changing or removing an element (e.g. a service or database), developers need to know what depends on it and what it depends on. Impact analysis reduces risk and speeds up refactoring decisions.

## When to Apply

- User asks: "What breaks if I change X?" or "What is affected if I remove Y?"
- Before refactoring, deprecating, or replacing a component
- When evaluating the cost of a change for planning or ADRs

## Correct Approach

1. **Identify the element.** Use the element ID as it appears in repo.sruja (e.g. `AuthService`, `Application.Database`).

2. **Run architecture-level impact (DSL).** Execute `sruja human explain <element_id> --file repo.sruja`. The output includes:
   - Element description
   - Incoming relations (who depends on this element)
   - Outgoing relations (what this element depends on)

3. **Run code-level impact (scan graph) when refactoring code.** Execute:

   - `sruja impact <target> -r . --depth 3` (text)
   - `sruja impact <target> -r . --depth 3 -f json` (machine-readable)

   Interpretation:
   - `upstream` = dependents (callers/importers): what is likely to break if `<target>` changes
   - `downstream` = dependencies (callees/imports): what `<target>` relies on
   - output includes centrality metrics when available (use as a risk signal; not a proof)

4. **Optional: full hierarchy (DSL).** Run `sruja tree repo.sruja` to see the full architecture tree. Note: `tree` takes a **file path**, not an element ID.

5. **Summarize for the user.** Report:
   - Dependents (incoming): components that would be affected if this element is removed or changed
   - Dependencies (outgoing): components this element relies on
   - Any structural concerns (cycles, high fan-in/fan-out, high centrality)

Use only data from the CLI and DSL; do not invent dependencies.

## CLI Notes

- `sruja human explain <id> --file repo.sruja` — architecture (DSL) impact. Use `--file` if the baseline is not repo.sruja.
- `sruja tree repo.sruja` — correct (tree accepts a file path).
- `sruja impact <target> -r . --depth 3` — code (scan graph) impact. `<target>` can be an exact node id or a substring match against id/label/path.

## Incorrect Approach

- Using unsupported flags or syntax (e.g. tree with element or depth)
- Fabricating dependency data not present in explain output or the DSL
- Applying changes to the architecture without the user asking for updates

## Summary

**Impact analysis: for DSL use `sruja human explain <element_id> --file repo.sruja`; for code refactors use `sruja impact <target> -r . --depth 3` (or `-f json`). Summarize dependents and dependencies from evidence only.**

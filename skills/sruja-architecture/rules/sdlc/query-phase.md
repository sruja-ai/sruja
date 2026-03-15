# query-phase

## Why It Matters

Developers and architects need to answer questions about the architecture without editing it: impact of changes, requirement traceability, and compliance status. The query phase provides read-only, evidence-based answers using the CLI and the DSL.

## When to Apply

- "What breaks if I change or remove X?" (impact analysis)
- "Which components implement requirement R1?" (requirement traceability)
- "Are we within architectural rules?" or CI gate (compliance)

## Correct Approach

**Impact analysis:** Run `sruja explain <element_id> --file repo.sruja` to get incoming/outgoing relations and description. Optionally `sruja tree repo.sruja` for full hierarchy. Summarize dependents and dependencies; do not invent. Note: `sruja tree` takes a file path, not an element; there is no `--depth` flag.

**Requirement traceability:** Read repo.sruja for requirement definitions. Identify which elements are linked via tags, references, or narrative. If not linked, suggest how to add traceability. Optionally use `sruja export markdown repo.sruja` to see requirements in exported docs.

**Compliance:** Run `sruja compliance -r . -a repo.sruja -f json` (and optionally `sruja validate repo.sruja --policy`). Summarize status, health_score, violations, and remediation. Present findings; do not auto-apply fixes.

## Incorrect Approach

- Using `sruja tree <element> --depth N` (CLI does not support that)
- Fabricating dependency or traceability data not in the DSL or graph
- Applying compliance fixes automatically without user decision

## Summary

**Query phase: use explain/tree/compliance/validate to answer questions; present findings; do not edit the DSL unless the user explicitly asks for an update.**

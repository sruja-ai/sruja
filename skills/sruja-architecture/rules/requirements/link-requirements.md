# link-requirements

## Why It Matters

Linking requirements to architecture elements (systems, containers, components) answers "which parts implement requirement R1?" and supports compliance, onboarding, and impact analysis when requirements change.

## When to Apply

- After capturing requirements in repo.sruja (see capture-requirements.md)
- When preparing for compliance or audit (traceability matrix)
- When refining architecture so that each major element can be justified by requirements

## Correct Approach

**In the DSL:** Use tags, references, or narrative in element descriptions to associate elements with requirement IDs.

Example (tags or description):

```sruja
R1 = requirement functional "Users must be able to log in"

AuthService = container "Auth Service" {
  technology "Node.js"
  description "Implements R1: user login and session management"
}
```

Or use tags if the DSL supports requirement tags on elements (e.g. `tags ["R1", "R2"]`). Refer to the language spec for exact syntax.

**Querying traceability:** Read repo.sruja and list which elements reference or tag each requirement. Use `sruja export markdown repo.sruja` to see requirements and structure in exported docs. For "which components implement R1?", return the set of elements that reference R1.

## Incorrect Approach

- Claiming traceability without actual links in the DSL
- Linking every requirement to every element (no value)
- Inventing links not present in the file

## Summary

**Link requirements to elements via tags or descriptions; use the DSL and export as the source of truth for traceability queries.**

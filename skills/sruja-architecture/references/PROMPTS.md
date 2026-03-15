# Prompt Patterns

This file contains reusable prompt patterns for generating and refining Sruja architecture DSL.

## Discovery Prompts

### Initial Discovery

```
Use sruja-architecture skill. If .sruja/context.json exists and is recent, use it for evidence; otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`. Gather evidence from the repo (structure, technologies, modules, entry points, dependencies). Ask targeted questions only when scope or externals are unclear. Generate a minimal repo.sruja with evidence-based components and relationships using C4 context and container levels. Then run `sruja lint repo.sruja` and fix all errors until it passes. Do not guess about missing information; list open questions instead.
```

### Focused Scope Discovery

```
Use sruja-architecture skill. Run `sruja discover --context -r <path> --format json` to gather evidence. Focus on <specific area>: <additional context>. Generate repo.sruja covering only this scope with evidence-based components and relationships. Run `sruja lint repo.sruja` and fix all errors. List any open questions.
```

### Refinement Discovery

```
Use sruja-architecture skill. Analyze this existing repo.sruja:

[PASTE CONTENT]

Use .sruja/context.json if recent, or run `sruja sync -r .` / `sruja discover --context -r . --format json` to gather current evidence. Compare the architecture against the evidence. Identify:
1. Components that don't match code
2. Missing relationships
3. New components detected
4. External dependencies not documented

Propose updates to align architecture with current evidence. Run `sruja lint repo.sruja` and fix all errors. List open questions.
```

## Modeling Prompts

### Generate from Requirements

```
Use sruja-architecture skill. Generate repo.sruja for these requirements:

[PASTE REQUIREMENTS]

Follow these guidelines:
- Use C4 levels (person, system, container, component)
- Include technology tags for all containers
- Add clear descriptions for all elements
- Create specific, labeled relationships with protocols
- Apply architectural patterns appropriate to requirements
- Check for anti-patterns

Generate the DSL, then run `sruja lint repo.sruja` and fix all errors.
```

### Refactor Architecture

```
Use sruja-architecture skill. Review and refactor this repo.sruja:

[PASTE CONTENT]

Issues to address:
[PASTE ISSUES]

Refactor to:
- [GOALS]
- Apply best practices from rules/
- Remove anti-patterns
- Improve separation of concerns
- Validate trade-offs

Run `sruja lint repo.sruja` and fix all errors after each change.
```

### Add Feature

```
Use sruja-architecture skill. Add [feature] to this existing architecture:

[PASTE EXISTING DSL]

Feature requirements:
[PASTE REQUIREMENTS]

Update architecture appropriately. Consider:
- New components needed
- New relationships required
- Impact on existing structure
- Trade-offs

Run `sruja lint repo.sruja` and fix all errors.
```

## Validation Prompts

### Lint and Fix

```
Use sruja-architecture skill. Run `sruja lint <file>.sruja`. Fix all reported errors:
- E201: Invalid kind or type
- E204: Circular dependencies
- E205: Orphan elements
- E206: Invalid references

Iterate until lint passes with zero errors.
```

### Review for Anti-Patterns

```
Use sruja-architecture skill. Review this repo.sruja for anti-patterns:

[PASTE CONTENT]

Check for:
- God components
- Direct database access from multiple layers
- Circular dependencies
- Tight coupling
- Orphan components

Report any issues and suggest fixes. Apply fixes and run `sruja lint repo.sruja`.
```

### Validate Trade-offs

```
Use sruja-architecture skill. Review this repo.sruja for trade-off validation:

[PASTE CONTENT]

Evaluate:
- Monolith vs microservices - appropriate for team size and domain complexity?
- Sync vs async - appropriate for requirements?
- Security layers - properly layered?
- Scaling strategies - justified?

Report concerns or missing considerations. Update architecture with rationale comments if needed.
```

## SDLC Prompts

### Create phase (new baseline)

```
Use sruja-architecture skill. Create initial repo.sruja for this repo. If .sruja/context.json exists and is recent, use it; otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`. Ask 2-5 targeted questions only when evidence is ambiguous. Generate minimal repo.sruja with evidence-based components and relationships (C4 context + container). Run `sruja lint repo.sruja` and fix all errors. List open questions for anything unclear; do not guess.
```

### Update phase (drift → propose → human approves)

```
Use sruja-architecture skill. Run `sruja sync -r .` then `sruja drift -r . -a repo.sruja --format json`. Analyze drift output (cycles, orphans, layer violations, new/missing components). Propose concrete edits to repo.sruja to address drift. Show the diff or list changes; do not apply until the user approves. After approval, run `sruja lint repo.sruja` and fix any errors.
```

### Impact analysis ("What breaks if I change X?")

```
Use sruja-architecture skill. For impact of changing element <element_id>: run `sruja explain <element_id> --file repo.sruja` (and optionally `sruja tree repo.sruja` for full hierarchy). Summarize: what depends on this element (incoming), what it depends on (outgoing), and which components would be affected if it is removed or changed. Use evidence only; do not invent dependencies.
```

### Requirement traceability ("Which components implement R1?")

```
Use sruja-architecture skill. For requirement traceability: read repo.sruja (or architecture.sruja) and identify requirement definitions (e.g. R1 = requirement functional "..."). Then list which elements (systems, containers, components) are linked to that requirement via tags, references, or narrative. If the DSL does not yet link requirements to elements, suggest how to add tags or references for traceability. Optionally use `sruja export markdown repo.sruja` to see requirements in exported docs.
```

### Compliance check

```
Use sruja-architecture skill. Run `sruja compliance -r . -a repo.sruja -f json` (or `sruja validate repo.sruja --policy` if policy files exist). Summarize: status, health_score, structural violations, drift_entries, policy_violations, and remediation_checklist. If non-compliant, list concrete steps to fix. Do not apply changes automatically; present findings for user decision.
```

## Refinement Prompts

### Drift Detection

```
Use sruja-architecture skill. Run `sruja drift -r . -a repo.sruja --format json` to detect drift. Analyze the results:

- New cycles detected
- New orphan components
- New layer violations
- Suggested structural improvements

Propose updates to repo.sruja to address drift. Run `sruja lint repo.sruja` and fix all errors. List open questions.
```

### Baseline Update

```
Use sruja-architecture skill. Update this repo.sruja baseline:

[PASTE CONTENT]

Use .sruja/context.json if recent, or run `sruja sync -r .` / `sruja discover --context -r . --format json` for current evidence. Update repo.sruja to reflect:
- New components
- Changed technologies
- New dependencies
- Removed/renamed elements

Maintain backward compatibility where possible. Run `sruja lint repo.sruja` and fix all errors.
```

## Open Question Handling

### Surface Uncertainties

```
Use sruja-architecture skill. When generating repo.sruja, if evidence is insufficient for a decision:

1. Do not guess
2. Add a comment block with OPEN QUESTIONS
3. List what information is missing
4. Suggest how to gather the information

Example:

/*
OPEN QUESTIONS:
- Authentication mechanism unclear from code
- Message queue purpose not documented
- External API endpoints not fully discovered
*/
```

## Best Practices

### Evidence-First Modeling

```
Use sruja-architecture skill. When modeling architecture:

1. Use .sruja/context.json for evidence if present and recent; otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`
2. Base every component on evidence
3. Every relationship must be traceable to code
4. Technology tags must match what's actually used
5. If evidence is missing, list as open question

Do not fabricate components, relationships, or technologies not supported by evidence.
```

### Minimal Viable Architecture

```
Use sruja-architecture skill. Generate the minimal architecture that:
- Captures the main system boundaries
- Shows key containers
- Documents critical datastores
- Includes essential external dependencies

Start at C4 context level. Add container level when evidence supports it. Add component level only for complex subsystems. Avoid premature abstraction or over-modeling.
```

## Error Handling

### Lint Error Resolution

For each lint error type:

**E201 - Invalid kind or type:**
```
Use sruja-architecture skill. Fix E201 error in repo.sruja:
- Check that kind (person, system, container, database, queue) matches element purpose
- Verify syntax: `kind "Name" { }` format is correct
- Ensure all required fields are present
- Run `sruja lint` after fix to verify
```

**E204 - Circular dependencies:**
```
Use sruja-architecture skill. Fix E204 circular dependency:
- Identify the cycle: A -> B -> C -> A
- Extract common functionality to break the cycle
- Introduce intermediate components if needed
- Update relationships
- Run `sruja lint` after fix to verify
```

**E205 - Orphan elements:**
```
Use sruja-architecture skill. Fix E205 orphan element:
- Either add relationships to the orphan
- Or remove the orphan if it's not actually needed
- Check if it should be nested under a parent
- Run `sruja lint` after fix to verify
```

**E206 - Invalid references:**
```
Use sruja-architecture skill. Fix E206 invalid reference:
- Verify target element exists
- Check spelling and case sensitivity
- Update reference to point to valid element
- Run `sruja lint` after fix to verify
```

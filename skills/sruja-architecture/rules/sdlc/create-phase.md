# create-phase

## Why It Matters

The create phase establishes the first evidence-based architecture baseline for a repo. Doing it right avoids invented components, keeps the DSL minimal, and gives the team a single source of truth they can trust and update over time.

## When to Apply

- New repository with no existing repo.sruja or architecture.sruja
- New subsystem or product that needs its own baseline
- After a major pivot where the old architecture file is no longer valid

## Correct Approach

1. **Gather evidence first.** Use `.sruja/context.json` if present and recent (e.g. updated_at within the last hour). Otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`. Do not ask the user to run the command first; run it yourself.

2. **Ask targeted questions only when evidence is ambiguous.** Typical questions: main system boundaries, external services, datastores, deployment model, main data flows. Do not ask about things already clear from evidence.

3. **Generate minimal DSL.** Start with C4 context and container levels. Add component level only when evidence justifies it. Every component and relationship must be supported by evidence.

4. **Validate.** Run `sruja lint repo.sruja` and fix all errors (E201, E204, E205, E206) before considering complete.

5. **Surface open questions.** For anything unclear or missing from evidence, add an OPEN QUESTIONS comment block; do not guess.

## Incorrect Approach

- Generating architecture before gathering evidence
- Inventing components, technologies, or relationships not present in the code
- Skipping lint or leaving errors for "later"
- Asking many questions when evidence already answers them

## Summary

**Create phase: evidence → minimal DSL → lint → open questions. No guessing.**

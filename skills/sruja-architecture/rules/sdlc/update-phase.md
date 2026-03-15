# update-phase

## Why It Matters

Code and intent change over time. The update phase keeps the architecture document in sync with reality using drift detection and proposed edits, while keeping a human in the loop so high-stakes documentation changes are intentional and reviewable.

## When to Apply

- After significant code changes (new services, refactors, removed components)
- After intent or ADR updates that affect the declared architecture
- On a regular cadence (e.g. before releases or sprint boundaries) to catch drift

## Correct Approach

1. **Gather evidence and detect drift.** Run `sruja sync -r .` then `sruja drift -r . -a repo.sruja` (or use `--format json` for machine-readable output). Sync writes `.sruja/context.json` and drift compares the declared architecture to the current graph.

2. **Analyze drift output.** Identify: new cycles, new orphans, layer violations, new or missing components, relationship changes. Classify as intentional (architecture evolved), unintentional (technical debt), or false positive (scope/configuration).

3. **Propose changes.** Generate concrete DSL edits (add/remove/update elements and relationships). Show a diff or clear list of changes. Do not apply edits automatically.

4. **Human approval.** The user reviews and approves or rejects. Only after approval should edits be applied to repo.sruja.

5. **Apply and validate.** After approval, apply the changes, then run `sruja lint repo.sruja` and fix any errors. Optionally the user commits or opens a PR.

## Automation Level

- **Automatic:** Running sync and drift; analyzing results; proposing edits.
- **Manual:** Applying changes to the DSL; committing or creating a PR.

## Incorrect Approach

- Auto-applying DSL changes without user approval
- Inventing drift fixes not supported by the drift report
- Skipping lint after updating the file

## Summary

**Update phase: sync → drift → propose → human approves → apply → lint. Apply is always manual.**

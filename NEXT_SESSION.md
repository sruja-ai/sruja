# Sruja Development Context

**Strategy:** See [architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md) for the canonical Architecture Intelligence direction, module decisions, and execution plan.

## Current Focus (from FINAL §10 Execution Plan)

### Phase 1 (2-3 weeks): Adoption Surface
1. Implement `quickstart` command.
2. Improve deterministic `why` explanations with evidence templates.
3. Unify drift logic with `sruja-diff`.
4. Reorder README/docs around no-key first value.

### Tactical TODO
- Refactor `sruja drift` – move heuristics to `sruja-diff` core, require architecture baseline
- Improve `sruja why` – evidence templates, deterministic answers without LLM
- Add `sruja quickstart -r .` as primary entry point (no API key required)

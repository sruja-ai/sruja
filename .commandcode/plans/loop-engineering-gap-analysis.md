# Loop Engineering: Gap Analysis & Implementation Plan

## Thesis

Sruja's loop engineering principle: **"the actor never grades itself; the grader is independent and deterministic."** The current implementation covers the inner loop well but has gaps in verification completeness, comprehension grounding, and post-loop analysis that undermine this thesis.

---

## Top 5 Gaps (Ranked by Thesis Impact)

### #1 [CRITICAL] Default Grader Skips `sruja intent check`

**What's wrong:** The default in-loop grader (`loop_grader.rs`) only runs `sruja lint repo.sruja` + `sruja drift --structural-only`. It never runs `sruja intent check`, which is the richer verification — policy rules, boundary violations, undocumented components, missing relationships. Meanwhile, `verify-task` profiles (`bugfix`, `review`, `arch`) *do* include intent check. Two verification systems diverged.

**Fix:** Add a third `VerifyStep` to the default grader, gated on intent artifacts existing.

**Files to modify:**

- `crates/sruja-cli/src/commands/loop_grader.rs` — add step `"grader_intent"` that runs `sruja intent check -r . -f json`. Gate with `has_intent_artifacts(repo_path)` that checks for `.sruja/adr/` or `.sruja/decisions/` directories.
- `crates/sruja-cli/src/commands/loop_grader.rs` — update `LoopGraderOptions.default_grader_fail_on` to accept intent-related fail criteria (optional, default stays cycles,layer-violations).

**Existing pattern to reuse:** `loop_grader.rs:default_grader_steps()` already appends conditionally — `if contract_path.exists()`. Follow same pattern.

**Verification:** Unit test verifies that when intent artifacts exist, the grader produces 3 steps instead of 2. Integration test: `sruja agent loop` on a repo with `.sruja/adr/` and a boundary violation runs intent check and vetoes convergence.

---

### #2 [HIGH] Evidence Pipeline Not Wired Into Loop Comprehend Phase

**What's wrong:** `.sruja/author_evidence.json` is built by `sruja sync` and `sruja author evidence`, consumed by MCP and review prompts, but **never** injected into the agent loop's comprehension phase. The actor starts each loop with the goal text + agentic memory, but without structured architecture context (workspace units, communities, entrypoints, datastores).

**Fix:** Load evidence before `agent.run_loop()` and inject it into comprehension context.

**Files to modify:**

- `crates/sruja-cli/src/commands/agent_loop.rs` — between manifest resolution and `agent.run_loop()` call, read `.sruja/author_evidence.json` and pass key evidence as `system_hints` or preloaded context.
- `crates/sruja-agent/src/goal.rs` — optionally add `evidence_brief: Option<String>` to `GoalSpec` so it's a first-class field rather than a system hint string hack.
- `crates/sruja-agent/src/cognition/mod.rs` — in `comprehend()`, if `GoalSpec.evidence_brief` is `Some`, inject it alongside preloaded files and memory context.

**Capping strategy:** Truncate at 8K tokens or ~400 lines, matching `comprehend()`'s existing tool-result truncation.

**Verification:** Unit test on `GoalSpec` serialization with evidence field. Integration test: agent loop with evidence present prints "Architecture Evidence" in the comprehension prompt dump.

---

### #3 [HIGH] Post-Loop Retrospective Is Shallow

**What's wrong:** `sruja agent reflect` only checks verification results (pass/fail) and suggests a guardrail memory entry. It doesn't analyze *why* the loop terminated — was it wrong assumptions, tool failures, budget exhaustion, oscillation? It doesn't generate proposals to adjust `loop.toml` or surface actionable patterns.

**Fix:** Extend reflect to read the full `LoopResult` trajectory and produce termination-specific analysis + config patch proposals.

**Files to modify:**

- `crates/sruja-cli/src/commands/agent_reflect.rs` — rewrite `suggest_from_bundle()` to:
  1. Load the `LoopResult` from `loop.json` (not just `facts_bundle.json`)
  2. Match on `LoopTermination` variant and generate targeted analysis:
     - `Oscillation` → suggest adding critique personas, increasing max_iterations
     - `SpendCapExceeded` → suggest raising cap or switching to cheaper model tier
     - `MaxIterations` → check if verifier kept failing (deterministic blocker) vs critic rejecting
  3. If verifier failed consistently across iterations, emit a `LoopConfigPatch` proposal
  4. Still persist guardrails to memory for the next run (existing `--write` behavior)

**Existing pattern to reuse:** `agent_reflect.rs` already deserializes `facts_bundle` JSON. `LoopResult` and `LoopTermination` types already exist in `cognition/mod.rs`. The data is in `.sruja/runs/<run_id>/loop.json`.

**Verification:** Unit tests for each `LoopTermination` variant produce the correct `ReflectSuggestion`. Snapshot test for the generated `LoopConfigPatch` text.

---

### #4 [MEDIUM] Default Grader Self-Health Check

**What's wrong:** The verifier is the last line of defense for "actor never grades itself". If `sruja drift` stops detecting violations due to regression, or the allowlist is accidentally emptied, the loop silently converges on broken code. There is no pre-flight check that the grader itself works.

**Fix:** Add a pre-loop smoke test: verify the sruja binary works, `repo.sruja` is parseable, and drift can produce a structural report.

**Files to modify:**

- `crates/sruja-cli/src/commands/loop_grader.rs` — add `verify_grader_health(repo_path, sruja_bin) -> Result<(), Vec<String>>`:
  1. `sruja --version` — binary exists and responds
  2. If `repo.sruja` exists: `sruja lint repo.sruja --nocolor` — contract is parseable
  3. `sruja drift --structural-only -r . -f json` — drift tool works
- `crates/sruja-cli/src/commands/agent_loop.rs` — call `verify_grader_health()` after resolving sruja binary, before building the agent. Print warnings (don't abort — the loop can still run without a grader if `--no-default-grader` is set, but warn loudly if the grader is misconfigured).

**Verification:** Unit test verifies health check passes on real repo. Unit test with corrupted binary or malformed repo.sruja produces expected errors.

---

### #5 [MEDIUM] Cross-Run Pattern Detection

**What's wrong:** Each loop run starts from scratch. Memory consolidates/prunes per-run, but there's no aggregation across runs. "The last 3 loops all failed on boundary X in the database layer" is invisible — the actor has to learn this de novo each time.

**Fix:** After each loop, compare the current run's failure patterns against previous runs. Detect recurring failures and surface a cross-run pattern.

**Files to modify:**

- New file: `crates/sruja-cli/src/commands/cross_run.rs`:
  ```rust
  pub struct CrossRunPattern {
      pub pattern_type: CrossRunPatternType,  // RecurringVerifyFailure | RecurringCritiqueIssue | RecurringBoundaryViolation
      pub count: u32,
      pub element_id: Option<String>,
      pub description: String,
  }

  pub fn detect_cross_run_patterns(
      repo_path: &Path,
      current_run: &serde_json::Value,
      window: usize,  // how many past runs to consider
  ) -> Vec<CrossRunPattern>
  ```

  Logic: load `.sruja/runs/*/loop.json`, compare `verify_failed` entries by verifier step ID, compare critique issues by element ID. If 3+ consecutive runs have the same verifier step failing on the same element → emit pattern.
- `crates/sruja-cli/src/commands/agent_loop.rs` — call `detect_cross_run_patterns()` after trajectory persistence and print results:
  ```
  Pattern detected: 3 consecutive runs failed on 'layer-violations'
  in the Database element. Review the architecture boundary before
  proceeding with more changes in this area.
  ```
- Wire into `sruja agent history` output so users can see patterns across their entire run history without running a new loop.

**Existing pattern to reuse:** `.sruja/runs/<run_id>/loop.json` already stores full `LoopResult`. `crates/sruja-agent/src/cognition/mod.rs` has `LoopResult` and `LoopIteration` types. Just deserialize + compare.

**Verification:** Unit test with 3 mock loop.json files — the first two have same failure on element X, third has different failure → pattern count = 2. Integration test: after running 2 loops with intentional boundary violations, `agent history` shows pattern.

---

## What NOT to Fix Right Now

| Gap                                 | Why Skip                                                                                               |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Pair/DLC/Multi skip the grader      | Intentional — these are experimental coordination patterns. Wire grader only on explicit demand.      |
| No E2E test for`sruja agent loop` | High cost, low marginal value for the core thesis. The mock-based tests already verify loop logic.     |
| Calibration halt path untested      | Important but narrow. Add as a sub-task of#3 (reflect already reads loop results).                     |
| Outer loop automation               | Premature — needs#3 and #5 to provide data first. System evolution from repeated failures is Phase 2. |

---

## Implementation Order

| Step | Gap                              | Files                                                             |
| ---- | -------------------------------- | ----------------------------------------------------------------- |
| 1    | #1 Default grader + intent check | `loop_grader.rs`                                                |
| 2    | #4 Grader self-health check      | `loop_grader.rs`, `agent_loop.rs`                             |
| 3    | #2 Evidence in comprehend        | `agent_loop.rs`, `goal.rs`, `cognition/mod.rs`              |
| 4    | #3 Deep retrospective            | `agent_reflect.rs`                                              |
| 5    | #5 Cross-run patterns            | `cross_run.rs` (new), `agent_loop.rs`, `agent.rs` (history) |

Each step is independently mergable. Steps 1 and 2 have zero coupling. Steps 3-5 depend on nothing but existing types.

## Verification Plan

For each change:

1. **Unit tests** — new/modified test functions in the same file (inline `#[cfg(test)]`)
2. **Existing test suite** — `cargo test --workspace` passes with no regressions
3. **CLI smoke test** — `sruja agent loop --dry-run --goal "test"` on this repo and a repo with intent artifacts
4. **Edge cases** — repos with no `.sruja/adr/`, no `repo.sruja`, no `author_evidence.json`

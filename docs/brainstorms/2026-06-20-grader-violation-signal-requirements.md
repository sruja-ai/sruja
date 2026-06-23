  ---
date: 2026-06-20
topic: grader-violation-signal
---

# Grader: Real Violation Signal + Tiered Output

## Summary

Wire the agent loop's critique path to the existing violation engine so it emits real, change-scoped violations, and make every finding **accuracy-gated** — only signal that is actually relevant to the change surfaces by default. The critique stops restating metadata as findings and starts reporting what a change actually broke; passes that cannot be accurate do not fire by default.

---

## Problem Frame

The agent loop's critique path is the grader the loop depends on to decide pass / critique / replan. But as it stands today it does not actually *detect* violations. Its one real-detector pass is an empty stub, and the remaining passes restate metadata as findings: they emit Medium/High "findings" whenever a changed file is path-proximate to an element (constraints, gotchas, incident matches, a generic behavioral-drift "review this"), or flag essentially every change as an unproposed-change violation at High/Critical. Severity and confidence are hardcoded high (0.8–1.0) regardless of evidence.

The cost is exactly the reported pain — "very verbose, sometimes useless." Every change produces a wall of restated context rather than detected breaks, so real violations are buried and the overall risk level is driven by noise. A staff/principal running critique cannot trust it as an independent grader.

Meanwhile a mature, tested, change-scoped violation engine already exists and is wired into `sruja check` (`sruja-engine` rules such as `crates/sruja-engine/src/rules/layer_violation.rs`, and `sruja-diff` drift detection in `crates/sruja-diff/src/drift.rs` / `crates/sruja-diff/src/compare.rs`). The critique path simply does not call any of it. So this is an integration + presentation gap, not a missing-capability gap — and leaving it means the product ships two parallel graders (one real, one fake-noisy), which directly undercuts the core thesis of a single independent grader that never grades itself.

---

## Actors

- A1. **Agent loop (critique phase):** consumes critique output to judge a change and decide whether to pass, critique, or replan.
- A2. **Staff/principal engineer (or developer via IDE/MCP):** reads critique findings and risk level to decide whether a change is safe.
- A3. **Existing violation engine (`sruja-engine` rules + `sruja-diff`):** the real detector being wired into the critique path; already used by `sruja check`.

---

## Key Flows

- F1. **Change-scoped violation grading**
  - **Trigger:** the agent loop (or a human) requests a critique of a change.
  - **Actors:** A1, A3
  - **Steps:** resolve the change to a scoped graph diff → run the violation engine against that diff → collect real violations into the Violation tier → collect remaining architectural context into the Context tier → compute risk from violations only → present violations (Context suppressed by default).
  - **Outcome:** the consumer sees real, change-attributed violations when present, and Clear/silence when absent; context is available on request.
  - **Covered by:** R1, R2, R4, R7, R9

---

## Requirements

**Real violation detection**

- R1. The critique path must surface real architectural violations produced by the existing violation engine (`sruja-engine` rules + `sruja-diff`), scoped to the change being critiqued — not the whole repository.
- R2. A change that introduces a forbidden dependency, layer break, cycle, orphan, or custom-constraint breach must produce a Violation-tier finding that names what was violated and why, sourced from the engine's existing output.
- R3. The empty policy-violation pass must be replaced by the real engine; no Violation-tier finding may be fabricated from metadata restatement or file-path proximity alone.

**Finding tiers**

- R4. Critique output must separate findings into two tiers — **Violations** (real detector output) and **Context** (declared constraints, gotchas, incident matches). The behavioral-drift heuristic and unproposed-change notices are not accurate detectors and are removed from the default output until they can be made accuracy-gated.
- R5. Only accurate signal surfaces by default. Violation-tier findings come from the real detector; Context-tier items surface only when the change genuinely implicates them — never on mere file proximity or unconditionally. There is no suppression toggle; a pass that cannot be accurate does not fire by default.
- R6. Context-tier items must not be presented as violations and must not raise the risk level; they are awareness, surfaced only on accurate relevance.

**Risk and summary integrity**

- R7. The overall risk level must be computed from Violation-tier findings only; Context-tier items must not raise it.
- R8. The summary must distinguish violation count from context count (e.g., "0 violations, 12 context items"), not collapse them into a single "issues" number.

**Change-scoping**

- R9. Violation detection must be attributed to what the change introduced (the diff), so a pre-existing violation unrelated to the current change is not reported as if this change caused it.

---

## Acceptance Examples

- AE1. **Covers R2, R3, R9.** Given a change that adds a module→database dependency forbidden by a declared boundary, when critiqued, the output contains a Violation-tier finding naming the boundary break, sourced from the violation engine (not from path proximity).
- AE2. **Covers R4, R5.** Given a change that touches an element which has declared constraints and gotchas but neither introduces a violation nor genuinely implicates those constraints, when critiqued, the output shows zero findings — no violations and no context — because the context is not relevant to the change.
- AE3. **Covers R7, R8.** Given the no-violation change from AE2, the risk level is Clear and the summary reports "0 violations" rather than counting context items as issues.
- AE4. **Covers R9.** Given a repo with a pre-existing layer violation unrelated to the current change, when the change is critiqued, that pre-existing violation is not attributed to the change.
- AE5. **Covers R5, R6.** Given a change that genuinely alters a state machine covered by a gotcha but introduces no violation, when critiqued, the gotcha surfaces as Context (accurate relevance), but the risk level stays Clear.

---

## Success Criteria

- A staff/principal running critique on a real change sees real violations when present and silence/Clear when absent — the "useless, verbose" feeling is resolved, with context available but not in the way.
- A change that breaks a boundary yields a Violation-tier finding traceable to the existing engine; a clean change yields zero Violation-tier findings; the risk level reflects violations only.
- The agent loop's critique consumers (planner/replan, IDE/MCP rendering) continue to function with the new tier model and the more-frequent Clear results — no regression in loop behavior.

---

## Scope Boundaries

- Mutation testing / recall audit (the original ideation seed) — deferred until real signal and the noise cut have landed; recall is meaningless while there is nothing real to recall.
- Consolidating the critique path into the check engine / retiring the parallel grader (Approach C) — the documented follow-up destination, not v1.
- Building new violation types beyond what the engine already detects (layer / cycle / orphan / god-module / custom-constraint).
- Fixing the agent-loop verify-step grep-gaming (the `crates/sruja-agent/src/verify/mod.rs` substring path) — a separate path and a separate effort.
- Escape/rework telemetry (a separate measurement track).
- The external-context / edge-agent vision — deferred per strategy.

---

## Key Decisions

- **Wire the existing engine rather than build a detector from scratch:** a mature, tested, change-scoped detector already exists and backs `sruja check`; building parallel would duplicate the very grader the thesis says should be singular.
- **Two tiers (Violations / Context) rather than deleting the Context passes:** constraints, gotchas, and incident matches carry awareness value *when relevant*; the fix is to gate them on accuracy, not to remove the capability.
- **Accuracy-gated by default (not a suppression toggle):** the grader must be accurate out of the box — restatement passes surface only when the change genuinely implicates them, and until accurate they don't fire at all. Hiding noise behind a toggle would miss the point of the tool.
- **Risk computed from Violations only:** risk driven by restated metadata was the core noise source.
- **Behavioral-drift heuristic and unproposed-change notice do not fire by default (not rewritten in v1):** neither detects anything real today; they are removed from the default output rather than tiered, until they can be made accuracy-gated.

---

## Dependencies / Assumptions

- **Verified:** a real, change-scoped violation engine exists (`crates/sruja-engine/src/rules/layer_violation.rs`, `crates/sruja-diff/src/drift.rs`, `crates/sruja-diff/src/compare.rs::detect_violations`) and is wired into `sruja check` but not into the critique path.
- **Verified:** the critique path's noise is structural — six of its seven passes restate metadata or flag proximity/paperwork; the seventh (policy violations) is an empty stub. There is no hidden real detector inside critique.
- **Assumption:** the violation engine can be scoped to a change (diff). `compare.rs::detect_violations` already operates on graph diffs; bridging `changed_files` to such a diff is implementation work deferred to planning.
- **Dependency:** critique output consumers (agent loop planner/replan, IDE/MCP rendering) must tolerate the new tier model and more-frequent Clear results — see Outstanding Questions.

---

## Outstanding Questions

### Deferred to Planning

- [Affects R5][Needs research] What makes a constraint, gotcha, or incident "genuinely implicated" by a change (the accuracy gate)? Define the relevance test each Context pass must pass before surfacing.
- [Affects R1, R9][Technical] How are `changed_files` bridged to a change-scoped graph diff the violation engine can consume? (`sruja-scan` + `sruja-diff` primitives exist.)
- [Affects R4][Technical] Does the finding model carry a new tier field on `CritiqueFinding`, or is Context carried in a separate structure? Planning decides the shape.
- [Affects R5][Needs research] Which critique consumers (agent loop planner/replan, IDE, MCP) read the restatement-pass output today, and does removing it from the default change agent behavior — not just human-facing output?

# Plan: DR-on-proceed audit trail

## Problem

The calibration verdict (`Ask` / `ProceedSilent` / `ProceedAndFlag` / `ProceedCitingPrecedent`)
is computed and displayed in `sruja focus`, but `sruja agent loop` **ignores it entirely**.
When the agent proceeds without asking, there is no structured audit trail explaining *why
it was allowed to act autonomously*. The existing LLM-generated DecisionRecord captures "why
the change was made" but not "why the agent decided to act without asking."

## Design principle (loop-engineering thesis)

The calibration is a **pure deterministic function owned by the grader/governance layer**.
The actor (agent loop) invokes it but cannot override it. This slice wires the verdict into
`agent loop` as a **pre-flight gate**: the actor asks the grader "should I proceed?" before
it starts, and the grader's answer is recorded as an immutable artifact.

## Flow

```
agent_loop(goal, goal_spec, repo)
  │
  ├─ compute AskInput
  │    reversibility ← infer_reversibility(goal + target element kinds)
  │    blast_radius   ← target_elements.len() + target_files.len()  (heuristic, capped)
  │    confidence     ← None  (unmeasured — no real signal yet)
  │    has_precedent  ← memory query for hitl_kind == "precedent"
  │    policy_says_ask← false (no policy DSL yet)
  │
  ├─ calibration::decide() → AskPlan
  │
  ├─ match verdict:
  │    Ask                    → halt unless --yes; print reason; exit
  │    ProceedSilent          → proceed (no DR — matches "silent" semantics)
  │    ProceedAndFlag         → write calibration DR (flagged); proceed
  │    ProceedCitingPrecedent → write calibration DR (citing precedent); proceed
  │
  ├─ agent.run_loop()  ← proceeds as normal
  │
  └─ (existing) LLM-generated DR  ← "why the change was made"
```

**Two DRs, two questions:**
- Calibration DR (deterministic, pre-execution): "Why did the agent decide to ACT without asking?"
- LLM DR (generated, post-execution): "Why was this change made?"

Both live in `.sruja/decisions/` with distinguishable titles.

## Slice breakdown

### 1. Pure function: `proceed_decision_record()` — `calibration.rs`

Constructs a `DecisionRecord` from a `Proceed*` verdict. Returns `None` for `Ask` (no DR for
the halt case — the halt message IS the record).

```rust
pub fn proceed_decision_record(plan: &AskPlan, goal: &str) -> Option<DecisionRecord>
```

- Title: `"Calibration: proceeded — {goal}"` (truncated)
- Status: `Accepted`
- Context: reversibility, blast radius, confidence, precedent, policy
- Decision: the verdict + reason from AskPlan
- Consequences: risk profile derived from the verdict
- Alternatives: "Ask human first" + why it was not needed

**Tests (pure, no I/O):**
- Returns `Some` for each `Proceed*` variant; `None` for `Ask`.
- DR title contains the goal.
- DR context contains blast radius + reversibility.
- DR decision contains the verdict reason.
- `to_markdown()` renders valid markdown.

### 2. Pure function: `calibration_gate()` — `agent_loop.rs`

Extracts the gate decision into a testable boundary (no async, no LLM, no I/O):

```rust
enum GateOutcome {
    Halt { reason: String },
    Proceed { plan: AskPlan, record: Option<DecisionRecord> },
}

fn calibration_gate(
    goal: &str,
    target_elements: &[String],
    target_files: &[String],
    has_precedent: bool,
    thresholds: &Thresholds,
    force_proceed: bool,
) -> GateOutcome
```

**Tests (pure):**
- One-way-door goal, no precedent, no force → `Halt`.
- One-way-door goal, no precedent, `force_proceed=true` → `Proceed` (no DR — it was forced, not calibrated).
- Two-way-door, bounded blast → `Proceed` with `ProceedSilent`, `record=None`.
- Mid-confidence → `Proceed` with `ProceedAndFlag`, `record=Some`.
- Precedent → `Proceed` with `ProceedCitingPrecedent`, `record=Some`.

### 3. Wiring: `agent_loop.rs` pre-loop gate

Before `agent.run_loop()`:
- Load thresholds via `load_ask_thresholds()` (reuse from focus.rs, or move to shared util).
- Query memory for precedent (reuse the `hitl_kind == "precedent"` check).
- Call `calibration_gate()`.
- On `Halt`: print message, return `Ok(())` early (clean exit, not an error).
- On `Proceed`: if `record` is `Some`, write it to `.sruja/decisions/` via `std::fs::write`.
- Print verdict line to console (human) or include in JSON (machine).

### 4. CLI flag: `--yes` — `subcommands.rs` + `run.rs`

Add `--yes` (aliased `--force-proceed`) to `agent loop`:
- When set, `Ask` verdict is overridden (proceed with a warning, no calibration DR — it was
  forced, not calibrated).
- Default: `false` (calibration gate is active).

Add `force_proceed: bool` to `AgentLoopOptions`.

### 5. Telemetry (optional, defer if time-constrained)

Add `calibration_verdict: Option<AskPlan>` to `LoopResult` so hosts can see what the gate
decided. Low priority for this slice — the DR is the primary artifact.

## Files to change

| File | Change |
|------|--------|
| `crates/sruja-agent/src/calibration.rs` | Add `proceed_decision_record()` + tests |
| `crates/sruja-agent/src/lib.rs` | Re-export `proceed_decision_record` |
| `crates/sruja-cli/src/commands/agent_loop.rs` | `calibration_gate()`, pre-loop wiring, `force_proceed` field |
| `crates/sruja-cli/src/cli/subcommands.rs` | `--yes` flag on `agent loop` |
| `crates/sruja-cli/src/cli/run.rs` | Pass `force_proceed` through |
| `crates/sruja-cli/src/commands/mod.rs` | (if `GateOutcome` needs re-export) |

## Key design decisions

1. **Gate at CLI boundary, not in `run_loop`**: keeps the agent crate LLM-focused; calibration
   is a pre-flight check the host owns. `run_loop` doesn't change for this slice.

2. **`ProceedSilent` writes no DR**: matches its "silent" semantics. `ProceedAndFlag` and
   `ProceedCitingPrecedent` carry explicit signal and get DRs. A future `record_all_proceeds`
   config can opt-in full auditing.

3. **`--yes` is explicit override, not calibration**: when forced, we proceed but write no
   calibration DR — the DR records *calibrated* autonomy, not human-forced bypass. The console
   warns "proceeding despite calibration Ask."

4. **Blast radius heuristic**: `target_elements.len() + target_files.len()`, capped at `u16::MAX`.
   When no targets are specified, blast is 0 (unknown — reversibility still drives decisions).
   Real graph-based blast radius is a future refinement (requires loading the architecture graph).

5. **Precedent from memory**: reuse the `hitl_kind == Some("precedent")` check from focus. The
   agent's memory store is loaded via `.memory(repo_path)`; we query it before the loop.

6. **Halt is `Ok(())`, not `Err`**: calibration halting is *expected* behavior (the gate worked),
   not a failure. The exit message explains why.

## Validation

```bash
# Pure tests
cargo test -p sruja-agent calibration
cargo test -p sruja-cli --bin sruja -- calibration_gate proceed_decision

# Lint + format
cargo clippy -p sruja-agent -p sruja-cli -- -D warnings
cargo fmt -p sruja-agent -p sruja-cli -- --check

# Manual smoke (dry-run, one-way-door goal — should halt)
cargo run -p sruja-cli -- agent loop --goal "migrate the database schema" --dry-run

# Manual smoke (forced proceed — should write calibration DR)
cargo run -p sruja-cli -- agent loop --goal "refactor API handler" --dry-run --yes
ls .sruja/decisions/
```

## Out of scope (future slices)

- `VerifyProfile::Design` — pre-code architectural drift gate.
- Real graph-based blast radius in the gate (current: heuristic from scope size).
- Policy-as-code DSL (`policy_says_ask` is hardcoded `false` for now).
- Confidence signal from the grader/critique (currently `None` — unmeasured).
- Interactive prompt on `Ask` (currently halt-only; `--yes` for CI).
- `record_all_proceeds` config for full ProceedSilent auditing.

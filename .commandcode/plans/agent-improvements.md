# sruja Agent System: Consolidation Plan

**Approach**: Consolidation-first — clean the foundation, then add pipeline YAML generation.

## Phase 1 — Consolidate compression to one path

The Agent has **two** compression strategies and a gate flag to stop them from fighting:

- **`CompressingClient`** — a transparent wrapper around `LlmClient` that compresses old tool results before every LLM call. Reversible, preserves CCR. This is the modern path — CLI wraps the tiered client with it.
- **Inline `compress_tool_results()`** — fires every 3rd iteration inside `run_tool_loop_inner`, mutating the message history in-place. This is the legacy path.
- **`disable_legacy_compression`** gate flag — prevents both from running simultaneously.

**Fix**: Eliminate the dual path. Remove the inline compressor and the gate flag. `CompressingClient` becomes the single compression mechanism. If someone constructs an `Agent` without wrapping in `CompressingClient`, they get no compression (simpler, no silent default).

**Files**: `crates/sruja-agent/src/cognition/mod.rs`
- Delete `compress_tool_results()` function (~lines 542–614)
- Delete `disable_legacy_compression` field from `AgentConfig` (line 109)
- Remove the gated compression block at line 1634–1637
- Remove the `disable_legacy_compression` assignment in CLI `agent_loop.rs` line 853

## Phase 2 — Unify orchestration paths

`Agent::run()` (full TDD pipeline) and `Agent::run_loop()` (simplified loop) share infrastructure but are separate code paths. `run_loop()` already drives from `PipelineConfig` — fold `run()`'s logic into it.

**Files**: `crates/sruja-agent/src/cognition/mod.rs`
- Add `Plan`, `Critique`, `Reflect` stage handlers to `run_loop()` — they currently no-op for those stages. Wire them to call `self.plan()`, `self.critique()`, `self.reflect()` same way `run()` does.
- Move adaptive routing (`try_direct_execution`) into `run_loop()` — run it before the pipeline loop when complexity is Trivial/Simple.
- Delete `Agent::run()` and `try_direct_execution()` / `should_try_direct()` from the `Agent` impl.
- Update `crates/sruja-agent/src/cognition/parsing.rs` if it references `run()` internals.
- Update `crates/sruja-agent/src/lib.rs` re-exports.
- Update `crates/sruja-agent/src/cognition/tests.rs` — migrate `run()` integration tests to `run_loop()` with appropriate pipeline configs.

## Phase 3 — Typed error variants

**Files**: `crates/sruja-agent/src/cognition/mod.rs`
- Add to `AgentError`:
  ```rust
  #[error("agent loop timed out after {0}s")]
  Timeout(u64),
  #[error("validation error: {0}")]
  Validation(String),
  #[error("checkpoint error: {0}")]
  Checkpoint(String),
  #[error("MCP error: {0}")]
  Mcp(String),
  ```
- Replace ~12 `AgentError::Other(...)` call sites with typed variants.
- Update match arms in callers that pattern-match on `AgentError`.

## Phase 4 — Extract monoliths

**Files to create**:
- `crates/sruja-agent/src/cognition/agent.rs` — `Agent` struct impl, `AgentBuilder`, all `impl Agent { ... }` methods. `mod.rs` drops to ~800 lines (types + config + re-exports).
- `crates/sruja-cli/src/commands/loop_report.rs` — `LiveReportHook`, `ReportState`, `SubtaskInfo`, `PersonaInfo`. `agent_loop.rs` drops to ~900 lines.

**Files to update**:
- `crates/sruja-agent/src/cognition/mod.rs` — add `mod agent;`
- `crates/sruja-cli/src/commands/agent_loop.rs` — move LiveReportHook out, import from `loop_report`
- `crates/sruja-cli/src/commands/mod.rs` — add `pub mod loop_report;`

## Phase 5 — Configurable stuck-detection thresholds

**Files**: `crates/sruja-agent/src/cognition/mod.rs`
- Add to `AgentConfig`:
  ```rust
  pub max_consecutive_tool_only: usize,   // default 3, 0 = disable
  pub max_consecutive_same_call: usize,   // default 3, 0 = disable
  pub max_non_converged_fraction: f64,    // default 0.5, >1.0 = disable
  ```
- Replace hardcoded values in `run_tool_loop_inner` with config lookups.
- Wire these through `LoopManifest` so users can set them in `.sruja/loop.toml`.

## Phase 6 — Pipeline YAML generation from goal

**Goal**: `sruja auto` generates a pipeline from the goal, writes it for editing, then follows it.

**Files**: `crates/sruja-agent/src/manifest.rs`
- Add `PipelineConfig::from_goal(goal: &GoalSpec) -> PipelineConfig`:
  - Trivial → `[Implement]`, no retries
  - Simple → `[Comprehend, Implement, Verify]`, 1 retry
  - Moderate → `[Comprehend, Plan, Implement, Verify, Critique]`, 1 retry
  - Complex → `[Comprehend, Plan, TestAuthor, Implement, Verify, Critique, Reflect]`, 2 retries with DiagnoseThenRetry

**Files**: `crates/sruja-cli/src/commands/auto_cmd.rs`
- On first run (no `--pipeline`): generate pipeline, write to `.sruja/auto-pipeline.toml`, print path, instruct user to edit and re-run with `--pipeline`.
- With `--pipeline <file>`: load and use it.
- Remove the "not yet implemented" eprint.

## Phase 7 — Clean up

- Remove `auto_format()` from `mod.rs` (calls `npx prettier` and `cargo fmt` via shell — let verify steps handle formatting).
- Trim `tests.rs`: migrate `run()` integration tests to `run_loop()`; delete dead test helpers.
- Update docs/AGENTS.md if it references `Agent::run()`.

## Verification

For each phase:
1. `cargo build` — must compile
2. `cargo clippy -- -D warnings` — no new warnings
3. `cargo test -p sruja-agent` — all tests pass
4. `cargo test -p sruja-cli` — CLI tests pass
5. Manual: `sruja auto "list files"` — produces same output as before

## Dependencies between phases

Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7

Each phase builds on the previous one. Do NOT reorder.

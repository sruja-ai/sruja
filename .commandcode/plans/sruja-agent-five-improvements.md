# Plan: 5 Agent Improvements

## Files Changed

| File | Change | Complexity |
|---|---|---|
| `crates/sruja-agent/src/manifest.rs` | Add `StageKind::Fix`, add `group` to `VerifyStep`, add `Fix` to pipeline defaults | Low |
| `crates/sruja-agent/src/cognition/mod.rs` | Add `fix()` method, `ScopeDrift` struct, `pre_conditions` on `Comprehension`, modify `replan()` prompt, modify `execute()` prompt, integrate Fix + drift into `run_loop()` | High |
| `crates/sruja-agent/src/cognition/prompts.rs` | Add `FIX_SYSTEM_PROMPT` | Low |
| `crates/sruja-agent/src/cognition/parsing.rs` | Add `extract_file_references()`, `parse_fix_response()` | Low |
| `crates/sruja-agent/src/verify/mod.rs` | Refactor `run_verification_steps()` for group-based concurrency, add `run_sequential_group()` | Medium |

---

## 1. Incremental Fix Mode

### New `StageKind::Fix` in `manifest.rs`

Add `Fix` to the `StageKind` enum and its mapping methods. `Fix` maps to `Phase::Implement` (can touch code) and `LoopPhase::Execute`.

### New `FIX_SYSTEM_PROMPT` in `prompts.rs`

A precision-focused prompt that:
- Takes the existing diff, critique issues with file/line attribution, and previously-written files
- Only modifies flagged files
- Outputs a JSON array of edit operations: `[{file, operations: [{type: replace|insert|delete, start_line, end_line, new_text}]}]`
- Uses lower temperature (0.1) and premium model tier

### New parsers in `parsing.rs`

- `extract_file_references(issues: &[String]) -> Vec<(String, Vec<usize>)>` — extract file paths and line numbers from critique issues using regex `path/to/file.rs:\d+` or `in file path/to/file.rs`
- `parse_fix_response(content: &str) -> Result<Vec<EditOperation>>` — parse the LLM's JSON fix output into structured edit operations

### New `fix()` method on `Agent` in `cognition/mod.rs`

```rust
pub async fn fix(
    &self,
    critique: &Critique,
    prior_results: &[StepResult],
) -> Result<Vec<EditOperation>, AgentError>
```

Steps:
1. Set `guard` to `Phase::Implement`
2. Call `extract_file_references()` — if empty, return empty (fall back to replan)
3. Get `git diff HEAD` for ground truth
4. Serialize critique context (issues, suggestions, criteria, persona_breakdown, file_references) as JSON
5. Build prompt with diff + structured critique
6. Run premium model with temperature 0.1
7. Parse result via `parse_fix_response()`

### Integration in `run_loop()`

When `StageKind::Fix` executes:
- If critique exists, has issues, and has file-level references → run `fix()`, apply edits, continue to Verify
- Otherwise → skip Fix stage (fall through to next stage)

### Pipeline Wiring

`PipelineConfig::from_goal()` inserts `Fix` after `Critique` for Moderate and Complex pipelines.

---

## 2. Structured Critique → Replan

### Modified `replan()` in `cognition/mod.rs`

Replace the current text-soup formatting with structured JSON injection:

```rust
let critique_json = serde_json::to_string_pretty(&serde_json::json!({
    "approved": critique.approved,
    "score": critique.score,
    "issues": critique.issues,
    "suggestions": critique.suggestions,
    "persona_breakdown": critique.persona_breakdown.iter().map(|p| {
        json!({"persona_id": p.id, "approved": p.approved, "issues": p.issues})
    }).collect::<Vec<_>>(),
    "criteria_matrix": critique.criteria.iter().map(|c| {
        json!({"index": c.index, "criterion": c.criterion,
               "status": c.status, "reason": c.reason})
    }).collect::<Vec<_>>(),
})).unwrap_or_default();

let user = format!(
    "## Goal\n{goal_str}\n\n\
     ## Prior Review Outcome (Structured)\n\
     The independent critic REJECTED the previous attempt.\n\
     Each issue is tagged with its originating persona.\n\n\
     ```json\n{}\n```\n\
     {failure_context}\
     ## Instructions\n\
     Produce a revised plan that addresses EVERY issue. \
     The `criteria_matrix` shows which acceptance criteria are not met — \
     each `missing` or `partial` entry MUST be addressed by new subtasks.\
     ...{tdd_note}{pressure_note}",
    ...
);
```

This is a small, low-risk change — only the prompt formatting changes. The `issues` field already contains persona-prefixed strings like `[correctness] null input on line 42`.

---

## 3. Adaptive Pipeline Escalation

### New `ScopeDrift` struct in `cognition/mod.rs`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeDrift {
    pub files_read: Vec<String>,
    pub files_edited: Vec<String>,
    pub exceeded: bool,
    pub escalated: bool,
}

impl ScopeDrift {
    /// Thresholds per complexity: (max_read, max_edited)
    pub fn detect(&mut self, initial: TaskComplexity) -> bool {
        let (max_read, max_edited) = match initial {
            TaskComplexity::Trivial => (1, 0),
            TaskComplexity::Simple => (3, 2),
            TaskComplexity::Moderate => (8, 5),
            TaskComplexity::Complex => (15, 10),
        };
        self.exceeded = self.files_read.len() > max_read
            || self.files_edited.len() > max_edited;
        self.exceeded
    }

    /// Add Plan and/or Critique if missing from the current pipeline.
    pub fn escalated_stages(&self, current: &[StageKind]) -> Vec<StageKind> {
        let mut stages = current.to_vec();
        if !stages.contains(&StageKind::Plan) {
            if let Some(pos) = stages.iter().position(|s| *s == StageKind::Comprehend) {
                stages.insert(pos + 1, StageKind::Plan);
            }
        }
        if !stages.contains(&StageKind::Critique) {
            stages.push(StageKind::Critique);
        }
        stages
    }
}
```

### Integration in `run_loop()`

- After each Implement/TestAuthor/Replan stage, extract file paths from `tool_signals` and update `ScopeDrift`
- After each Verify stage, call `ScopeDrift::detect()` — if exceeded, call `escalated_stages()` and update the loop's pipeline
- Requires `loop_config.pipeline` to be `&mut` in `run_loop()` (change from `&`)

---

## 4. Actionable Error Pre-conditions

### New `pre_conditions` field on `Comprehension` in `cognition/mod.rs`

```rust
pub struct Comprehension {
    /// ... existing fields ...
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_conditions: Vec<String>,
}
```

### Modified `comprehend()` in `cognition/mod.rs`

After building the `error_history` string (existing code), compute pre-conditions from error frequencies:

```rust
let pre_conditions: Vec<String> = frequencies.iter().filter_map(|f| {
    let pct = f.count as f64 / total as f64;
    if pct < 0.2 { return None; }
    Some(match f.error_class {
        ErrorClass::Compilation =>
            "Run `cargo check` before editing — high rate of compilation errors.".into(),
        ErrorClass::Type =>
            "Check type annotations carefully — type errors are common here.".into(),
        ErrorClass::Test =>
            "Verify test assertions against acceptance criteria before implementing.".into(),
        ErrorClass::Runtime =>
            "Check for unwrap/None — runtime panics are frequent.".into(),
        ErrorClass::Lint =>
            "Run `cargo clippy --fix` after changes.".into(),
        ErrorClass::Architecture =>
            "Run `sruja drift` before verification — boundary violations are common.".into(),
        _ => None,
    })
}).collect();
```

### Inject into execute prompts in `execute()`

When building the user prompt for each subtask, add a pre-condition section:

```
## Pre-conditions from Error History
- Run `cargo check` before editing — high rate of compilation errors.
```

---

## 5. Parallel Verification

### Add `group` field to `VerifyStep` in `verify/mod.rs`

```rust
pub struct VerifyStep {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub expected: Option<String>,
    /// Steps with different group values run concurrently.
    /// Steps without a group run in the default sequential group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}
```

### Refactored `run_verification_steps()` in `verify/mod.rs`

Partition steps by group → spawn concurrent `tokio::spawn` per group → sequential execution within each group → collect results preserving original order.

```rust
pub async fn run_verification_steps(
    steps: &[VerifyStep],
    opts: &VerifyOptions,
    workdir: &Path,
) -> Vec<VerifyResult> {
    // Partition by group
    let mut groups: HashMap<Option<String>, Vec<&VerifyStep>> = HashMap::new();
    for step in steps {
        groups.entry(step.group.clone()).or_default().push(step);
    }

    // Run each group concurrently
    let mut tasks = Vec::new();
    for (_key, group_steps) in groups {
        let opts = opts.clone();
        let wd = workdir.to_path_buf();
        let steps: Vec<VerifyStep> = group_steps.into_iter().cloned().collect();
        tasks.push(tokio::spawn(async move {
            run_sequential_group(&steps, &opts, &wd).await
        }));
    }

    // Collect and reorder
    let mut results_by_id: HashMap<String, VerifyResult> = HashMap::new();
    for task in tasks { ... }

    steps.iter().filter_map(|s| results_by_id.remove(&s.id)).collect()
}
```

`run_sequential_group()` is a copy of the current sequential execution logic.

**Backward compatible**: Steps without `group` → all in `None` group → sequential (current behavior).

---

## Verification

1. **Unit tests**: Add tests for `extract_file_references()`, `parse_fix_response()`, `ScopeDrift::detect()`
2. **Verify test**: Add test for parallel group execution in `verify/mod.rs`
3. **Compilation**: `cargo check -p sruja-agent` for each change
4. **Existing tests**: `cargo test -p sruja-agent` — all existing tests must pass
5. **Smoke test**: Run `sruja auto "fix typo in docs"` with a simple goal to validate the loop doesn't break

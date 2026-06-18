# Sruja Agent Efficiency Analysis

> **Last updated:** 2026-06-17 (refreshed after `agent loop` improvements)

## Current State

The sruja agent system has been tested with multiple LLM providers and real coding tasks. Here's what works and what needs improvement.

## What Works Well

### 1. Multi-Provider Support
- **Z.AI (GLM)**: Works with glm-4-flash model
- **XIMIMO**: Works with mimo-v2.5-pro model  
- **OpenRouter**: Works with google/gemini-2.5-flash, anthropic/claude-sonnet-4, meta-llama/llama-4-maverick
- **Anthropic**: Works via OpenRouter proxy

### 2. Deterministic Plan/Apply Workflow
- `sruja agent plan` generates reproducible plans from repo evidence
- `sruja agent apply` executes verification steps
- Plans are grounded in architecture context (drift, intent, focus)

### 3. Memory System
- `sruja agent history` shows learning history
- `sruja agent record` records new learnings
- `sruja agent clusters` shows thematic groups
- `sruja agent curate` suggests merges/deletions
- `sruja agent distill` records task outcomes
- `sruja agent session-summary` writes handoff summaries
- `sruja agent propose-fact` proposes architectural facts

### 4. Architecture Integration
- Agent understands codebase structure via sruja graph
- Plans respect layer boundaries and architecture rules
- Verification includes drift, intent, and compliance checks

### 5. Autonomous Loop (`sruja agent loop`)
- Closed-loop: comprehend → plan → execute via tools → critique → replan
- Spend cap, oscillation detection, max iterations guardrails
- `.sruja/loop.toml` manifest for declarative config
- Calibration gate with interactive TTY prompt or `--yes` override
- Default shell allowlist (`cargo`, `git`) — works out of the box
- Persisted trajectory to `.sruja/runs/<id>/loop.json`

## Previously-Reported Issues (now fixed)

| Issue | Status | Fix |
|-------|--------|-----|
| Auth from env vars only, ignores `.sruja/config.toml` | **FIXED** | `config::resolve_multi_provider_config` reads config.toml with multi-tier routing |
| Provider config duplication (setup vs loop) | **FIXED** | Unified config resolution chain: CLI > env > config.toml > defaults |
| No code modification support in apply | **FIXED** | `agent loop` uses `ToolRegistry::with_builtin` (file_read/write/edit, shell) |
| No default shell allowlist (silent tool failures) | **FIXED** | Defaults to `["cargo", "git"]` when `loop.toml` omits it |
| No interactive prompt on calibration halt | **FIXED** | TTY detection + `[y/N]` prompt; non-TTY falls back to `--yes` flag |

## Remaining Efficiency Issues

### 1. No Streaming Per-Iteration Progress
**Problem**: `sruja agent loop` runs the full cognition loop and prints results only at the end. A multi-minute run is silent.

**Impact**: Users see no feedback during execution. Feels unresponsive.

**Fix**: Add a hook/callback in the cognition loop (the `Hooks` infra exists in `cognition/hook.rs`) to print iteration N/M, critique score, and verify pass/fail as they complete.

### 2. No Eval Harness for Regression Tracking
**Problem**: No automated way to run the loop against a benchmark suite and detect regressions.

**Fix**: Implement `EVAL_HARNESS_PLAN.md` — benchmark suite with `expected.json` criteria, `run_benchmark.sh`, and CI integration.

### 3. Focus Not Auto-Grounded in Agent Loop
**Problem**: The sruja tools (focus, explain, drift) are registered but the agent isn't prompted to use them. Architecture grounding depends on LLM choosing to call them.

**Fix**: Add a system prompt nudge: "call sruja_focus before planning" when `repo.sruja` exists.

### 4. No Resume After Spend-Cap/Timeout
**Problem**: If a loop hits the spend cap or max iterations, the next run starts from scratch.

**Fix**: Support `--resume <run_id>` to continue from the last plan/iteration state.

## Recommendations

### Immediate (done)
1. ~~Unify configuration system~~ — Done (multi-provider config resolution)
2. ~~Default shell allowlist~~ — Done (`cargo`, `git`)
3. ~~Interactive calibration prompt~~ — Done (TTY `[y/N]`)
4. ~~Trajectory persistence~~ — Done (`.sruja/runs/<id>/loop.json`)

### Next
1. Add per-iteration streaming progress
2. Ship eval harness (`EVAL_HARNESS_PLAN.md`)
3. Auto-ground in `focus` via system prompt
4. Add `--resume` for interrupted loops

## Testing Results

### Provider Performance
| Provider | Model | Plan Generation | Loop Execution | Notes |
|----------|-------|-----------------|----------------|-------|
| Z.AI | glm-4-flash | ✅ Works | ✅ Works | Fast, deterministic |
| XIMIMO | mimo-v2.5-pro | ✅ Works | ✅ Works | Good quality |
| OpenRouter | google/gemini-2.5-flash | ✅ Works | ✅ Works | Best quality |
| OpenRouter | anthropic/claude-sonnet-4 | ✅ Works | ✅ Works | High quality |
| OpenRouter | meta-llama/llama-4-maverick | ✅ Works | ✅ Works | Good quality |

### Task Completion
| Task | Plan Quality | Loop Success | Verification | Notes |
|------|--------------|--------------|--------------|-------|
| Add CLI comment | High | ✅ | ✅ | Deterministic plan |
| Architecture analysis | High | ✅ | ✅ | Comprehensive evidence |
| Code refactoring | Medium | ✅ | ✅ | Via tool execution in loop |
| Fix failing test | Medium | ✅ | ✅ | TDD mode |

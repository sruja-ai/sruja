# Sruja Agent Delivery Plan

**Status:** Active — Phase 0 closing.
**Owner:** Product + CLI/MCP (`sruja-cli`, `sruja-agent`, `sruja-memory`).
**Last updated:** 2026-05-20.

**Related docs (current truth):**

| Doc | Role |
|-----|------|
| [AI_DLC_ACE_SYNTHESIS_PLAN.md](AI_DLC_ACE_SYNTHESIS_PLAN.md) | AI-DLC + ACE integration plan |
| [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](../GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) | Harness vs host boundary |
| [AGENTIC_ORCHESTRATION_AND_SRUJA.md](../AGENTIC_ORCHESTRATION_AND_SRUJA.md) | What Sruja does *not* ship |
| [AIDLC_INTEGRATION.md](../AIDLC_INTEGRATION.md) | AIDLC CLI/MCP/CI docs |
| [CONTEXT_ENGINEERING.md](../CONTEXT_ENGINEERING.md) | MCP ladder, focus, pruning |

---

## North star

Sruja = portable harness + governed memory (any host, any number of skills).
Hosts + community skills = Act / Reflect (coding, debug, review).
Self-improvement = repo artifacts (memory, events, DRs, rules) — not an in-repo autonomous runtime or auto-minted skills.

## Non-goals (hold the line)

- `agent run --autonomous`
- In-process LLM orchestration
- Sruja skill router for N skills
- Auto-generated skill packs from trajectories

## Guiding principles

1. **Compose, don't duplicate** — Use addyosmani/agent-skills / skills.sh for SDLC process; Sruja for architecture truth and gates.
2. **Same end-of-task contract everywhere** — focus → work → verify → record → (optional) evolve rules.
3. **Two-layer planning** — Scope (issue/DR/workflow) vs task (plan.md, files, commands).
4. **Learn only after verify** — Tie `record_learning` / `auto_record_learnings` to passing checks when possible.
5. **Dogfood on this repo** — Every harness feature used on a real Sruja PR before docs-only ship.

---

## Codebase baseline (2026-05-20)

Grounded assessment of what exists today, what's in progress, and what's genuinely missing.

### What exists and is mature

| Area | State | Key files |
|------|-------|-----------|
| Agent memory (CRUD, Zettelkasten, curation) | **Complete** | `crates/sruja-agent/src/memory.rs` (~1300 lines, 20+ tests) |
| Agent run loop (observe-plan-apply-verify) | **Complete** | `crates/sruja-cli/src/commands/agent_run.rs` (~1745 lines) |
| Agent plan / apply / reflect | **Complete** | `agent_plan.rs`, `agent_reflect.rs` |
| MaTTS (multi-trajectory contrast) | **Complete** | `crates/sruja-agent/src/matts.rs` (~400 lines) |
| Workflow engine (3-phase gates) | **Complete** | `commands/workflow.rs` (~875 lines) |
| AIDLC integration | **Complete** | `commands/workflow_aidlc.rs` (~329 lines) |
| MCP tools (49+ tools) | **Complete** | `commands/mcp/` (definitions.rs, read.rs, governance.rs, graph.rs, memory.rs) |
| Context events (v1 + v2 schemas) | **Complete** | `commands/context_events.rs` (~686 lines) |
| Facts bundle schemas | **Complete** | `facts_bundle/v1` in agent_run.rs |
| Integration tests | **~70%** | `tests/commands_integration.rs` (40 tests); gap: no full lifecycle test |
| CI AIDLC gate | **Complete** | `.github/workflows/sruja-aidlc-gate.yml` (label-triggered) |
| Cursor commands (6 prompts) | **Complete** | `.cursor/commands/` |
| Cursor rules (2 .mdc) | **Complete** | `.cursor/rules/sruja-dev.mdc`, `sruja-context-host.mdc` |
| Skills (sruja-architecture, grill-me) | **Complete** | `skills/` |
| Extension (26 commands) | **Complete** | `extension/src/` |
| Config schema (code-driven) | **Complete** | `integrations/mod.rs` ([integrations], [agent], [baseline], [sandbox], [context_engineering]) |

### What's genuinely missing

| Item | Phase | Effort |
|------|-------|--------|
| `sruja verify-task` CLI command | 1.2 | 3–5 days (extract from `agent_run.rs`) |
| `sruja_verify_task` MCP tool | 1.2 | Trivial (once CLI exists) |
| `docs/HOST_AGENT_INTEGRATION.md` | 1.1 | 1–2 days (consolidate from existing docs) |
| `docs/COMMUNITY_SKILLS_STACK.md` | 2.3 | 1 day |
| `docs/SRUJA_SKILL_IMPROVEMENTS.md` | 2.3 | Half day (broken link in `skills/README.md:139`) |
| `skills/sruja-harness/SKILL.md` | 2.1 | Half day |
| `.cursor/commands/sruja-bugfix-triage.md` | 2.2 | Half day |
| `.cursor/commands/sruja-pre-merge-review.md` | 2.2 | Half day |
| `[verify]` config profiles in `.sruja/config.toml` | 1.2 | 1 day (new config section) |
| `facts_bundle/v2` (host, skills_used[], verify_profile) | 3.1 | 1 day (additive schema bump) |
| Full workflow lifecycle integration test | 0.2 | Half day |
| One dogfood PR through AIDLC workflow | 0.4 | 1–2 days |

### Key realization

The codebase is further along than the plan implies. Most Phase 0–1 work is **extraction or documentation**, not greenfield engineering. The only genuinely new runtime code is `verify-task` (Phase 1.2).

---

## Dependency graph

```
Phase 0 (workflow MVP — closing)
    ↓
Phase 1 (host contract + verify-task)  ←── blocks multi-skill everywhere
    ↓
Phase 2 (skills + commands)            ←── can parallel doc with 1.1
    ↓
Phase 3 (learning hardening)
    ↓
Phase 4 (AI-DLC automation)            ←── optional strict; needs 1+3
Phase 5 (packaging)                    ←── parallel after 2.1
```

---

## Phase 0 — Finish in-flight foundation (2–3 days)

Close the loop on existing workflow/AIDLC work before new surface area.

### 0.1 Land W1 MVP — DONE

`workflow.rs` + `workflow_aidlc.rs` are mature. `sruja-aidlc-gate.yml` exists. AIDLC pilot workflow exists at `.sruja/workflows/sruja-aidlc-pilot/`.

### 0.2 Integration tests — ~70%

`commands_integration.rs` has 40 tests including `workflow init --with-aidlc`, `workflow status`, `workflow advance`, `workflow audit`. Gap: no full lifecycle test.

**Missing:** Add one test exercising full lifecycle: init → approve inception → construction artifacts → gate_check.

### 0.3 AIDLC doc pass — DONE

`docs/AIDLC_INTEGRATION.md` (61 lines) matches CLI flags, linked from AGENTS.md.

### 0.4 Dogfood one PR — NOT STARTED

Run `.cursor/commands/sruja-workflow-aidlc.md` on a small `repo.sruja`/CLI change. Verify `audit.jsonl` + phase approvals on disk.

**This is the real Phase 0 gate.** Everything else in Phase 0 is already done.

### Phase 0 exit criteria

- `sruja workflow status --check` passes on a sample workflow in this repo
- One dogfood PR with `audit.jsonl` + phase approvals on disk
- Full lifecycle integration test green in CI

---

## Phase 1 — Host contract, any environment (1.5 weeks)

Makes Sruja usable with N skills in Cursor, Claude Code, CI, etc., without Sruja knowing skill names.

### 1.1 Document the universal run envelope

#### 1.1.1 `docs/HOST_AGENT_INTEGRATION.md`

Consolidate from existing docs. Much of this content already lives in:
- `GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md` (272 lines) — harness vs host boundary
- `AGENTIC_ORCHESTRATION_AND_SRUJA.md` — what Sruja does not ship
- `.cursor/rules/sruja-context-host.mdc` (38 lines) — session wiring

New doc should be the single source for:
- Boundary table (host owns what, Sruja owns what)
- Run envelope: START drift/focus → ACT (host) → VERIFY (verification bundle) → RECORD event → LEARN memory
- MCP vs CLI usage patterns
- Read-only mode (`SRUJA_MCP_READONLY`)
- Bot approval patterns

**Avoid duplication** — link to existing docs for deep details; this doc is the integration contract.

#### 1.1.2 Run envelope (normative)

The envelope already exists in code (`agent_run.rs` apply mode). Document it:

```
START → sruja drift / sruja focus → ACT (host/skill) → VERIFY (verification bundle) → RECORD (event) → LEARN (memory)
```

ASCII diagram + JSON examples for each step. The verification bundle format (`verification_bundle/v1`) already exists in `agent_run.rs`.

#### 1.1.3 `context_event/v2` extensions

`context_event/v2` already has: `trace_id`, `decision_id`, `run_id`, `workflow_id`, `actor`, `source`, `tool`, `elements`, `subject_ids`, `evidence_refs`.

Add: `host` (string), `skills_used` (string[]), `session_id` (string) — all optional. Backward-compatible additive change, not a new schema version.

#### 1.1.4 Examples directory

`docs/examples/host-gates/` — shell pre-apply, GitHub Action, MCP readonly inception. Copy-paste for teams.

### 1.2 Verification bundle (skill-agnostic)

#### 1.2.1 `sruja verify-task` CLI

**The only genuinely new runtime code in the plan.**

Extract from the existing verification executor in `agent_run.rs`. The existing code has:
- `run_sruja_cmd()` — executes sruja subcommands via current binary
- `run_allowlisted_process()` — executes external commands with allowlist + timeout
- `validate_sruja_cmd_args()` — restricts exact subcommand shapes
- `StepObservation` — captures status, exit_code, stdout, stderr, elapsed_ms
- Allowlist enforcement for both sruja subcommands and verify executables

`verify-task` reuses all of this. New surface:
- `--profile coding|bugfix|review|arch` — selects verification step subset
- Always emits a JSON report (success or failure); non-zero exit code on any failing step
- Profiles define which steps to run:

| Profile | Steps |
|---------|-------|
| `coding` | `just check` (or `make check`) + `lint` + `drift` (if architecture sources touched) |
| `bugfix` | `just check` (or `make check`) + `intent check` |
| `review` | `review -f json` + `intent check` + `drift` |
| `arch` | `lint` + `drift` + `intent check` + `review -f json` |

Ship `coding` + `default` first. Add profiles based on dogfood feedback.

#### 1.2.2 MCP `sruja_verify_task`

Thin wrapper returning same JSON as CLI. Same pattern as other 49 MCP tools.

#### 1.2.3 Profiles in `.sruja/config.toml`

**Note:** `.sruja/config.toml` does not currently exist. The config schema is code-driven in `integrations/mod.rs`. Two options:

1. **Create `config.toml`** — New file with `[verify]` section. More flexible, user-editable.
2. **Code-driven profiles** — Add to existing `[agent]` section defaults. Simpler, no file creation.

Recommend starting with option 2 in Phase 1.2 (ship `verify-task` with a hardcoded `default` + `coding`), then move to option 1 once there is dogfood signal for customization. The `[verify]` section:

```toml
[verify]
default_profile = "coding"

[verify.coding]
steps = ["lint", "check", "drift-if-arch"]
timeout_ms = 30000

[verify.bugfix]
steps = ["check", "intent"]
timeout_ms = 30000
```

### 1.3 Cursor / repo wiring

#### 1.3.1 `.cursor/commands/sruja-verify-task.md`

Calls verify profile from frozen plan or user args. 15–30 line prompt.

#### 1.3.2 Thicken `.cursor/rules/sruja-dev.mdc`

Add mandatory gates:
- `sruja focus` before non-trivial edit
- `verify-task` before "done"
- `drift`/`intent` when `.sruja` or cross-crate imports change

#### 1.3.3 Update `.cursor/rules/sruja-context-host.mdc`

Add: end session with `verify` + `record_learning` if verify failed.

### Phase 1 exit criteria

A contributor using only `docs/HOST_AGENT_INTEGRATION.md` + MCP can run the same loop in CI and Cursor without reading architecture skill internals.

---

## Phase 2 — Skills composition (community + Sruja) (1 week)

Coding, bugfix, review via Addy/skills.sh; Sruja stays thin.

### 2.1 Publish Sruja adapter skills (agentskills.io format)

#### 2.1.1 `skills/sruja-harness/SKILL.md`

~30 lines: "Use with any other skills; before done run verify-task; on failure record_learning." Installable via `npx skills add`.

The `agentskills.io` format is already used by `skills/sruja-architecture/SKILL.md` (YAML frontmatter). Follow same pattern.

#### 2.1.2 `skills/sruja-governed-delivery/SKILL.md`

Maps Addy commands (`/build`, `/review`) → Sruja verify profiles; links Cursor commands. Optional meta-skill.

#### 2.1.3 Rule `rules/sdlc/code-change-with-arch.md`

Under `sruja-architecture`: impact before edit; proposal not direct repo.sruja; post-change intent+drift.

#### 2.1.4 Update `docs/INSTALL_AS_SKILL.md`

Recommended stack: `sruja-architecture` + `sruja-harness` + optional Addy skills + post-steps table. No false "community replaces Sruja" framing.

### 2.2 Cursor commands (task-type)

#### 2.2.1 `sruja-bugfix-triage.md`

Repro → focus → memory search → hypothesis → minimal fix plan → verify profile `bugfix`. 30–50 line prompt.

#### 2.2.2 `sruja-pre-merge-review.md`

`review -f json` + intent check + structured blocker/suggestion/nit output. 30–50 line prompt.

#### 2.2.3 Cross-link commands

`plan-feature` → `implement-from-plan` → `verify-task` → `reflect-on-run`. Links in each command header.

### 2.3 Community skills doc

#### 2.3.1 `docs/COMMUNITY_SKILLS_STACK.md`

agentskills.io spec; addyosmani install for Cursor; skills.sh search tips; conflict avoidance (interview-me vs grill-me).

#### 2.3.2 `docs/SRUJA_SKILL_IMPROVEMENTS.md`

**Fix broken link** in `skills/README.md:139` first. Backlog: lint JSON loop, scope ladder, per-language validation. Living doc.

### Phase 2 exit criteria

New user can install 3 skills + run one Cursor command and get verify + memory without custom prompts.

---

## Phase 3 — Self-learning loop (harness hardening) (2 weeks)

Multi-skill, multi-env "evolution" without auto-spawning skills.

### 3.1 Traceability for N skills

#### 3.1.1 `facts_bundle.json` schema bump

Additive fields to `facts_bundle/v1`: `host` (optional string), `skills_used` (optional string[]), `verify_profile` (optional string), `verify_outcome` (optional string). Backward-compatible.

`agent_reflect` already reads `facts_bundle.json` — update to surface new fields.

#### 3.1.2 `sruja agent reflect` — finish docs + test

Already implemented (`agent_reflect.rs`, 130 lines). Default is suggest-only; document `--write` + curate flow.

#### 3.1.3 Learning conventions

Document `evidence_refs: ["skill:..."]` in `HOST_AGENT_INTEGRATION.md`. Convention only or optional field later.

### 3.2 Learning quality

#### 3.2.1 Record only on verify pass

**Design decision needed.** Current behavior:
- `auto_record_learnings = false` (default) → no auto-recording
- When enabled, `agent_run.rs` records on verify *failure* as correction learning

The plan says "don't record on failure." Consider:
- **Correction learnings on failure** (current behavior — useful for "what went wrong")
- **Affirmation learnings on pass** (new — "what worked")
- Separate flags: `auto_record_on_pass`, `auto_record_on_failure`

Config: `auto_record_learnings_on_verify_fail = false` (default) in `[agent]` section.

#### 3.2.2 Dedupe / cap

`agent curate` already suggests merges. Document utility pruning: `task_success_after` low → delete candidate. Add section to AGENTS.md.

#### 3.2.3 MCP `sruja_search_memory` — already exists

Tool is implemented and dispatched. Document in review/bugfix commands as step 2.

### 3.3 Outer loop (evolve rules, not auto-skills)

#### 3.3.1 After 3 repeated corrections

Run `sruja-evolve-rules-after-miss` (command already exists) → DR or AGENTS.md patch. Process documented in `COMMUNITY_SKILLS_STACK.md`.

#### 3.3.2 DR template for agent guardrails

`type: agent` in `.sruja/decisions/`. Example DR in repo.

#### 3.3.3 Dogfood metrics

Track context-score, learning utility, % PRs with workflow. Internal dashboard or CI comment.

### Phase 3 exit criteria

Failed verify runs produce correction learnings; stable policy promoted to DR/rules; bad learnings pruned by utility.

---

## Phase 4 — AI-DLC + minimal human (optional strictness) (3 weeks)

After Phases 1–3; builds on workflow already in progress.

| Step | Deliverable |
|------|-------------|
| 4.1 Phase-aware MCP | `workflow_id` + phase on `sruja_get_task_context`; token caps in construction |
| 4.2 Delta merge gates | Construction gate requires propose-approve path, not full repo.sruja rewrite |
| 4.3 Bot phase approval doc | Service account + `workflow approve` + CI required checks |
| 4.4 `sruja workflow design-review` | Stable JSON contract (enrich-cmd optional) |
| 4.5 CI matrix | Label `aidlc-workflow` + optional `verify-task` on all PRs |

### Phase 4 exit criteria

Headless bot can advance phases when artifacts + verify bundle pass; humans only on exceptions (DR `hitl_kind: exception`).

---

## Phase 5 — Product packaging & ecosystem (ongoing)

| Step | Detail |
|------|--------|
| 5.1 skills.sh listing | Ensure `sruja-harness` installable; README on leaderboard |
| 5.2 Book chapter | "AI-DLC with Sruja" + "Composable skills + harness" |
| 5.3 Extension | Optional: "Sruja: Verify task" command → runs CLI `verify-task` |
| 5.4 OpenHands / CI sample | One reference agent repo using MCP only (no Cursor) |

---

## Suggested execution order (first 30 days)

| Week | Focus | Concrete outputs |
|------|-------|------------------|
| 1 | Phase 0.4 + 1.1.1–1.1.4 | Dogfood PR + `HOST_AGENT_INTEGRATION.md` + host-gates examples |
| 2 | 1.2.1–1.2.3 + 1.3.1–1.3.3 | `verify-task` CLI/MCP + Cursor commands |
| 3 | 2.1.1–2.2.3 + 2.3.1 | `sruja-harness` skill + `INSTALL_AS_SKILL` stack |
| 4 | 3.1–3.3 + start 4.1 | `reflect`/`facts_bundle` + learning config + dogfood PR metrics |

---

## Acceptance criteria (program level)

1. Any host with MCP or CLI can run verify + `record_learning` without Cursor-specific APIs.
2. User with 5+ community skills + `sruja-harness` still gets drift/intent/review gates.
3. Coding / bugfix / review each have a Cursor command + verify profile + doc section.
4. No new autonomous mode; CI can block merge on verify + workflow.
5. One dogfood PR per phase proves the loop on the Sruja repo itself.

## What to defer

| Item | Why defer |
|------|-----------|
| In-process Bedrock / multi-agent runtime | Violates product boundary |
| Sruja skill router / skill marketplace | Host + skills.sh own discovery |
| Auto-generate skills from memory | Noise + maintenance; use DR/rules instead |
| W6 IaC scan | After verify-task + ops gate stable |
| Full HOST crypto/SSO | Org-specific; document pattern only |

## Immediate next 5 actions (this week)

1. **Finish Phase 0.4** — One dogfood PR with `workflow status --check`.
2. **Draft `docs/HOST_AGENT_INTEGRATION.md`** — Run envelope + N skills + learning (consolidate from existing docs).
3. **Spec `sruja verify-task`** — Short ADR or plan issue (profiles + JSON schema). Identify extraction points in `agent_run.rs`.
4. **Implement minimal `verify-task` (`default` + `coding`)** — Emit JSON on success/failure; wire through allowlists.
5. **Wire editor affordances** — Add `.cursor/commands/sruja-verify-task.md` + tighten `.cursor/rules` to require verify before “done”.

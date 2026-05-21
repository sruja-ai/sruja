# Host Agent Integration

**Status:** Active  
**Last updated:** 2026-05-20

Single source for integrating Sruja with any AI agent host (Cursor, Claude Code, CI, OpenHands, etc.). Sruja is a **deterministic harness** — the host owns the LLM loop.

---

## Boundary Table

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| **Harness** | Sruja CLI + MCP | `sync`, `lint`, `drift`, `intent check`, `verify-task`, focus briefings, agent memory, MCP tools |
| **Agent host** | Your editor / CI / script | Act (generate code/DSL), optional Reflect/Learn, tool orchestration beyond Sruja |
| **Reviewed truth** | Humans + promotion flow | `repo.sruja`, Decision Records, approved proposals |

**Sruja does NOT ship:**
- `agent run --autonomous` mode
- In-process LLM orchestration
- Skill router for N skills
- Auto-generated skill packs from trajectories

See [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md) and [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) for details.

---

## Run Envelope (Normative)

Every task follows the same end-of-task contract, regardless of host or skill:

```
START → drift/focus → ACT (host/skill) → VERIFY (bundle) → RECORD (event) → LEARN (memory)
```

### Step-by-step

#### 1. START — Ground the task

```bash
# Check for architectural drift before starting
sruja drift -r . -f json

# Get a focus briefing for the target file/element
sruja focus --file src/auth.rs -r . -f for-ai
```

**MCP equivalent:** `sruja_get_focus_briefing`, `sruja_check_drift`

#### 2. ACT — Host/skill does the work

The host (Cursor agent, Claude Code, CI script) performs the code or DSL changes. Sruja is not involved during this step.

#### 3. VERIFY — Run verification bundle

```bash
# Coding task (default)
sruja verify-task --profile coding -r .

# Bugfix targeting a specific file
sruja verify-task --profile bugfix --file src/auth.rs -r .

# Pre-merge review
sruja verify-task --profile review -r .

# Architecture change
sruja verify-task --profile arch -r .

# JSON output for CI/MCP
sruja verify-task --profile coding -r . -f json
```

**Verification bundle format (`verify_task/v1`):**
```json
{
  "schema_version": "verify_task/v1",
  "profile": "coding",
  "repo": ".",
  "all_passed": true,
  "steps": [
    {
      "step_id": "lint_repo_sruja",
      "status": "ok",
      "exit_code": 0,
      "stdout": "...",
      "stderr": "",
      "elapsed_ms": 42
    }
  ],
  "elapsed_ms": 1250
}
```

**Profiles:**

| Profile | Steps |
|---------|-------|
| `coding` | `lint repo.sruja` + `make check` (or `sruja check`) + `drift` |
| `bugfix` | `focus --file` + `make check` (or `sruja check`) + `intent check` |
| `review` | `review -f json` + `intent check` + `drift` |
| `arch` | `lint repo.sruja` + `drift` + `intent check` + `review -f json` |

**MCP equivalent:** `sruja_verify_task`

#### 4. RECORD — Log the event

```bash
# Events are auto-recorded for drift/intent/propose operations
# Manual event recording:
sruja event append -r . --json '{"kind": "task_complete", "details": {...}}'
```

**MCP equivalent:** `sruja_append_event`

#### 5. LEARN — Record learnings (on verify pass or failure)

```bash
# Record a learning from the task
sruja agent record -r . \
  -c "Refactored auth module" \
  -H "Migrate to JWT tokens" \
  -o success \
  -g "Always run verify-task after auth changes"

# Or on failure (correction learning)
sruja agent record -r . \
  -c "Auth refactor" \
  -H "Merge auth into API container" \
  -o failed \
  -g "Auth must remain a separate container per architecture"
```

**MCP equivalent:** `sruja_record_learning`

---

## MCP vs CLI Usage Patterns

### CLI (scripts, CI, shell hooks)

```bash
# Pre-commit hook
sruja verify-task --profile coding -r . || exit 1

# CI gate
sruja drift -r . -f json | jq '.violations | length' | grep -q '^0$'

# Post-merge sync
sruja sync -r .
```

### MCP (AI editors, interactive sessions)

```
# Cursor/Claude/Copilot with Sruja MCP server
# Command: sruja mcp -r .

Tools available vary by profile:
- minimal (~10-12 tools): Core ladder + focus briefing + essential utilities
- coding (~15-18 tools, default): Minimal + hybrid query + critique + context pruning
- arch: Coding + read-only authoring helpers
- full: All tools (backward compatible)

Examples:
- sruja_get_focus_briefing — task-scoped briefing
- sruja_verify_task — run verification profile
- sruja_check_drift — detect architectural drift
- sruja_search_memory — search past learnings
- sruja_record_learning — record new learning
- sruja_get_topology — upstream/downstream dependencies
- ... (full list via MCP initialize)

Profile control:
- Set via environment: SRUJA_MCP_TOOL_PROFILE=coding
- Set via MCP initializationOptions
- Defaults to coding profile
```

### Read-only mode

Set `SRUJA_MCP_READONLY=1` to expose only read/query tools. Mutating calls (record_learning, propose, etc.) return an error.

```bash
SRUJA_MCP_READONLY=1 sruja mcp -r .
```

---

## Bot Approval Patterns

### GitHub Actions

```yaml
# .github/workflows/sruja-verify.yml
name: Sruja Verify
on: [pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install sruja-cli --locked
      - name: Verify task
        run: sruja verify-task --profile coding -r . -f json
      - name: Drift check
        run: sruja drift -r . -f json
```

### Service account + workflow approval

For AI-DLC workflows with phase gates:

```bash
# Initialize workflow
sruja workflow init --with-aidlc --title "Add auth endpoint"

# Bot advances phases when artifacts pass
sruja workflow approve --id wf-001 --phase construction

# CI required checks: verify-task + workflow status
sruja workflow status --check
```

See [AIDLC_INTEGRATION.md](AIDLC_INTEGRATION.md) for full AI-DLC workflow docs.

---

## Config Profiles

Customize verification profiles in `.sruja/config.toml`:

```toml
[verify]
default_profile = "coding"

[verify.coding]
steps = ["lint", "check", "drift-if-arch"]
timeout_ms = 60000

[verify.bugfix]
steps = ["focus", "check", "intent"]
timeout_ms = 30000
```

See `integrations/mod.rs` for the full config schema.

---

## Examples

### Shell pre-apply hook

```bash
#!/bin/bash
# .sruja/hooks/pre-apply.sh
# Run before any agent apply step

set -e
REPO="${1:-.}"

echo "Running pre-apply verification..."
sruja verify-task --profile coding -r "$REPO" -f json > /tmp/verify.json

if ! jq -e '.all_passed' /tmp/verify.json > /dev/null; then
  echo "Verification failed. Steps:"
  jq -r '.steps[] | select(.status == "error") | "  ✗ \(.step_id): \(.stderr)"' /tmp/verify.json
  exit 1
fi

echo "All verification steps passed."
```

### MCP readonly inception

```json
{
  "tool": "sruja_get_focus_briefing",
  "arguments": {
    "path": ".",
    "file": "src/main.rs"
  }
}
```

### GitHub Action with verify-task

See `.github/workflows/sruja-aidlc-gate.yml` for the label-triggered AIDLC gate.

---

## References

- [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) — Harness vs host boundary, continual learning
- [AGENTIC_ORCHESTRATION_AND_SRUJA.md](AGENTIC_ORCHESTRATION_AND_SRUJA.md) — What Sruja does not ship
- [AIDLC_INTEGRATION.md](AIDLC_INTEGRATION.md) — AI-DLC workflow integration
- [CONTEXT_ENGINEERING.md](CONTEXT_ENGINEERING.md) — MCP ladder, focus, pruning
- [docs/plans/AGENT_DELIVERY_PLAN.md](plans/AGENT_DELIVERY_PLAN.md) — Delivery roadmap

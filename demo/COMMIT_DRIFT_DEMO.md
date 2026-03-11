# Commit-to-Commit Architecture Intelligence Demo

This demo answers: **“Give me a repo and two commits — show me Architecture Intelligence at the first commit, then show me what drifted by the second.”**

---

## Scenario

1. **Input:** A git repo + **baseline commit** (e.g. `main`, `v1.0`, or a SHA).
2. **Step 1:** Extract **Architecture Intelligence at that commit**: inventory, health score, top findings, actionable fixes (same as `sruja quickstart` at that tree).
3. **Input:** A **second commit** (e.g. your branch tip, or `HEAD`).
4. **Step 2:** Compare baseline → head: **drift report** — health change, **new** violations only, and changed files.

So you get:
- **At commit A:** “What does the architecture look like here?” (full snapshot.)
- **Between A and B:** “What got worse (or better) in this range?” (PR-scoped drift.)

---

## Insights we provide

### At baseline commit (Step 1)

| Insight | What you get |
|--------|-------------------------------|
| **Inventory** | Module/service/database counts, total dependencies. |
| **Health score** | 0–100 with a clear label (Good / Fair / Poor). |
| **Top critical findings** | God modules, bottlenecks, with **file paths**. |
| **Actionable fixes** | Prioritized (HIGH/LOW), impact text, list of affected components. |
| **Domain map** | Largest packages/crates so you see where weight is. |
| **Next steps** | Suggested follow-up commands. |

So you get a **full “quickstart” snapshot** at the baseline ref — no config, no API keys.

### Between baseline and head (Step 2)

| Insight | What you get |
|--------|-------------------------------|
| **Changed files** | List of files that changed between the two commits (from `git diff --name-only base...head`). |
| **Health score change** | Baseline health → head health (e.g. 85 → 78, “-7”). |
| **New violations only** | Violations that **exist at head but not at base** (new cycles, new god modules, new orphans, new layer violations). Existing issues are not re-listed. |
| **Severity and location** | Each new violation with severity (Error/Warning/Info), message, file/location, and suggestion. |
| **CI-friendly output** | `--format github-actions` for annotations; exit code 1 when new violations are introduced so CI can fail the PR. |

So you get a **before/after** view: “This PR introduced 2 new god modules and 1 new cycle,” with exact locations.

---

## How to run

### Prerequisites

- Git repo with the baseline and head refs available (e.g. `main` and your branch).
- Sruja CLI built: `make build` (or `sruja` on PATH).
- Repo **checked out at the commit you want as “head”** before running (e.g. checkout your feature branch, then run the demo).

### Script (recommended)

From the **sruja repo root** (or with `sruja` on PATH):

```bash
cd demo
chmod +x run_commit_drift_demo.sh
./run_commit_drift_demo.sh [REPO] [BASELINE_REF] [HEAD_REF]
```

**Defaults:** `REPO=.` (current dir), `BASELINE_REF=HEAD~1`, `HEAD_REF=HEAD`.

**Examples:**

```bash
# Compare previous commit vs current (e.g. last commit vs working tree)
./run_commit_drift_demo.sh

# Compare main vs current branch (run from repo root, with your branch checked out)
./run_commit_drift_demo.sh . main HEAD

# Compare two specific commits
./run_commit_drift_demo.sh /path/to/repo abc123 def456

# Compare origin/main vs current HEAD
./run_commit_drift_demo.sh . origin/main HEAD
```

The script:

1. Creates a temporary git worktree at **baseline**, runs `sruja quickstart` there (and caches the graph under `.sruja/cache/`).
2. Runs `sruja drift-pr -r REPO -b BASELINE_REF -H HEAD_REF` so you see health delta and **new** violations only.

### Manual commands

If you prefer to run steps yourself:

```bash
# 1. Architecture Intelligence at baseline (use a worktree or clone)
git worktree add --detach /tmp/base main
sruja quickstart -r /tmp/base
git worktree remove /tmp/base

# 2. Drift from baseline to current HEAD (repo must be at the “head” commit)
sruja drift-pr -r . --base main --head HEAD

# JSON or CI output
sruja drift-pr -r . --base main -f json
sruja drift-pr -r . --base origin/main -f github-actions
```

---

## Use in CI

To **fail the build when a PR introduces new architectural violations**:

```yaml
# .github/workflows/architecture-drift.yml
- name: Checkout
  uses: actions/checkout@v4
  with:
    fetch-depth: 0   # so refs like origin/main exist

- name: Install Sruja
  run: curl -fsSL https://sruja.ai/install.sh | bash

- name: Drift check (new violations only)
  run: sruja drift-pr -r . --base origin/main -f github-actions
```

Exit code is 1 when there are new violations; `github-actions` format prints annotations on the PR.

---

## Summary

| Question | What we provide |
|----------|------------------|
| “What’s the architecture at commit A?” | Full quickstart report at A (inventory, health, findings, paths). |
| “What drifted between commit A and B?” | Health delta + list of **new** violations (with severity and location) + changed files. |
| “Can we gate PRs on new violations?” | Yes: `sruja drift-pr --base origin/main` and use exit code + optional `-f github-actions`. |

This gives you a clear **commit → intelligence**, then **second commit → drift** story with concrete, actionable insights.

---

## Test run (sruja repo, HEAD~1 vs HEAD)

A quick run on this repo with `./run_commit_drift_demo.sh . HEAD~1 HEAD`:

**Step 1 (baseline at HEAD~1):**
- **Inventory:** 1138 modules, 1 service, 1 database, 3813 dependencies.
- **Health:** 99/100 (Good).
- **Top findings:** God modules with file paths (e.g. `sruja-intent/src/lib.rs`, `parser/adr.rs`) and suggestions.
- **Actionable fixes:** HIGH (decouple bottlenecks), LOW (review orphans), with affected file lists.

**Step 2 (drift HEAD~1 → HEAD):**
- **Changed files:** 1.
- **Health:** 99 → 99 (no change).
- **New violations:** 0 (existing 141 at both commits).
- **Message:** “No NEW architectural violations introduced in this PR!”

**Conclusion:** The demo provides useful insights: a full snapshot at the baseline commit, and a clear drift summary (changed files, health delta, new violations only). When the diff is small (e.g. one merge commit), drift correctly reports “no new violations.” For a stronger demo, use two commits that touch more code (e.g. a feature branch vs `main`).

**Caveat:** When Step 1 runs in a temporary worktree, the “Repository” and path-based domain map in quickstart use the worktree path (e.g. `/var/.../sruja-baseline-xxx`). Paths in findings still point at real crate paths under that tree; only the top-level “Domain Map” segment can look odd. This does not affect the drift step.

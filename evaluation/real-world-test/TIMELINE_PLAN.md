# Detailed Plan: ADR Capture, Architecture Snapshots, and Timeline Playback

**Status:** Implemented  
**Audience:** Implementers, reviewers  
**Related:** [TEST_ON_REAL_PROJECTS.md](./TEST_ON_REAL_PROJECTS.md) (ADR/timeline section), [drift_by_commit.sh](./drift_by_commit.sh)

### Implementation status

| Phase | Status | Notes |
|-------|--------|--------|
| **Phase 1** (timeline capture) | Done | `capture_timeline.sh`, ref sanitization, manifest, `timelines/REPO/`; `sruja timeline suggest-refs` (LLM); REPO/refs optional, `--overwrite`, `failed_refs` in manifest. |
| **Phase 2** (timeline report) | Done | `timeline_report.sh`, drift-diff loop, md + json, single-ref handling. |
| **Phase 3** (ADR capture) | Done | `sruja intent adr-index` (multi-dir, auto-detect, `-o` JSON); capture script runs adr-index when ADR dirs exist; report uses `adr_<ref>.json` for adrs_at_base/head. |
| **Phase 4** (polish) | Partial | Dirty tree/`--force`, missing-ref skip + `failed_refs`, TEST_ON_REAL_PROJECTS subsection done. `--parallel N` and `sruja timeline` (report) CLI not implemented. |

**Deliverables:** `capture_timeline.sh`, `timeline_report.sh`, `sruja timeline suggest-refs`, `sruja intent adr-index`, manifest with optional `failed_refs`, ADR index JSON with ref/sha/captured_at.

---

## 1. Goals and scope

### 1.1 Goals

| # | Goal | Success criterion |
|---|------|-------------------|
| G1 | **Capture ADRs at multiple points in time** | For any OSS repo (with ADRs in known paths), we can list/parse ADRs at ref A, ref B, … and store an index (ref → list of ADR metadata: path, title, status, date). |
| G2 | **Store architecture snapshots at multiple refs** | For a list of refs (tags, branches, commits), we can checkout each ref, run `sruja scan`, and persist the graph JSON under a stable directory layout so we don’t re-scan every time. |
| G3 | **Produce a timeline report** | Given stored snapshots (and optionally ADR indices), we can generate a single report (markdown or JSON) that describes, for each consecutive pair of refs: new/removed components, new/removed edges, and optionally ADR presence at each ref. |
| G4 | **Play the timeline** | Users can “play” the timeline by stepping through the report (human-readable steps or machine-readable JSON for a future UI). |

### 1.2 In scope

- Timeline capture and report for **real-world-test OSS repos** (gitea, etcd, caddy, react-admin, saleor, express, etc.).
- Ref selection: **automatic by default** via LLM analysis of commit messages (no user input); or explicit list; or fallback to tags + `main` / `--commits N`.
- Reuse existing: `sruja scan`, `sruja drift-diff`, `sruja intent check` (and intent’s ADR parsing), and the same LLM provider stack as `sruja eval` (OpenRouter, OpenAI, Anthropic, Gemini, Ollama).
- Scripts in `evaluation/real-world-test/` and, if useful, a minimal `sruja timeline` subcommand that reads from a timeline directory.

### 1.3 Out of scope (for this plan)

- Full timeline **visualization UI** (only the data/report; UI can be a later project).
- Changing how drift or health scoring works.
- Supporting non-git workflows (e.g. uploads of graph JSONs without a repo).
- Parsing ADR formats other than what `sruja-intent` already supports (MADR/Nygard-style markdown and .sruja).

---

## 2. Current state

| Component | Location | Behavior |
|-----------|----------|----------|
| Scan at one ref | `sruja scan -r . -o out.json` | Produces one graph JSON per run. |
| Drift between two graphs | `sruja drift-diff -b base.json -h head.json` | Diffs two graph JSONs; output: new/removed components and edges, violations. |
| Two-ref flow (commits) | `drift_by_commit.sh REPO [BASE] [HEAD]` | Checkout base → scan → checkout head → scan → drift-diff; restores branch. Does not persist graphs. |
| Intent / ADR load | `sruja intent check -r . -i DIR` | Loads from `DIR` (default `repo/docs/architecture`): `DIR/adr/decisions/*.md` and any `*.sruja` under `DIR`. Parsed ADRs: title, status, date, context, decision, consequences, implications. |
| ADR parser | `crates/sruja-intent/src/parser/adr.rs` | Parses .md/.markdown in one directory; expects MADR/Nygard-style sections. Path is currently `dir.join("adr").join("decisions")` only. |

**Gaps:**

- No multi-ref capture that **persists** graphs (and optionally ADR index) per ref.
- No ref selection from tags/branches.
- No “timeline report” that chains drift-diff over consecutive pairs.
- ADR discovery only under `docs/architecture/adr/decisions`; many OSS repos use `docs/adr/` or `doc/adr/`.

---

## 3. Data model and file layout

### 3.1 Directory layout

All timeline data for a repo lives under one directory. Proposed layout:

```
evaluation/real-world-test/
  timelines/
    REPO_NAME/                    # e.g. gitea, etcd, caddy
      manifest.json               # list of refs in order + metadata (see below)
      graph_<ref>.json            # one per ref; <ref> = tag name or ref sanitized (e.g. main, v1.0, abc1234)
      adr_<ref>.json              # optional; ADR index at that ref
      timeline_REPO_NAME.md       # generated report (or timeline_REPO_NAME.json)
```

**Ref naming in filenames:** Sanitize so filenames are safe: replace `/` with `-`, drop `^refs/heads/` or `^refs/tags/` if present, use short SHA when ref is a raw commit (e.g. `abc1234`). Examples: `main`, `v1.0`, `release-2.x`, `abc1234`.

### 3.2 manifest.json

Describes which refs we have and in what order. **Order is always oldest → newest** (index 0 = oldest ref, last index = newest). The timeline report compares consecutive pairs in this order (base = ref_i, head = ref_{i+1}). When refs come from LLM suggest-refs, the implementation must return SHAs in **oldest-first** order (e.g. reverse `git log` output, which is newest-first by default).

```json
{
  "repo": "gitea",
  "repo_path": "test-repos/gitea",
  "refs": [
    { "ref": "main", "sha": "abc123...", "captured_at": "2026-02-25T12:00:00Z" },
    { "ref": "v1.20.0", "sha": "def456...", "captured_at": "2026-02-25T12:05:00Z" },
    { "ref": "v1.21.0", "sha": "789abc...", "captured_at": "2026-02-25T12:10:00Z" }
  ],
  "graph_files": ["graph_main.json", "graph_v1.20.0.json", "graph_v1.21.0.json"],
  "adr_capture": true
}
```

- `refs`: ordered list; each entry has `ref` (branch/tag/sha), `sha` (full or short commit SHA at capture time), optional `captured_at` (ISO8601).
- `graph_files`: list of graph JSON filenames in the same order as `refs`.
- `adr_capture`: whether we also captured ADR indices for each ref.

### 3.3 Graph files

Same format as today’s `sruja scan -o out.json`: the existing `sruja_scan::Graph` JSON (nodes, edges, etc.). No schema change.

### 3.4 ADR index (adr_<ref>.json)

Per-ref index of what ADRs existed at that ref. Lightweight so we can “play” timeline with ADR context.

```json
{
  "ref": "v1.20.0",
  "sha": "def456...",
  "captured_at": "2026-02-25T12:05:00Z",
  "intent_dirs_tried": ["docs/architecture", "docs/adr"],
  "adrs": [
    {
      "path": "docs/architecture/adr/decisions/0020-use-postgres.md",
      "number": 20,
      "title": "Use PostgreSQL",
      "status": "Accepted",
      "date": "2024-01-15T00:00:00Z",
      "tags": ["database"]
    }
  ]
}
```

- If we add multiple intent dirs, `intent_dirs_tried` records which we looked at.
- `adrs`: list of parsed ADR metadata (path relative to repo root, number, title, status, date, tags). Full context/decision/consequences can be omitted in the index to keep it small; optional “full” export later.

### 3.5 Timeline report (timeline_REPO.md or .json)

**Markdown** (human-readable “play”):

```markdown
# Architecture timeline: gitea

Refs: main → v1.20.0 → v1.21.0

## main → v1.20.0
- New components: 12
- Removed components: 0
- New edges: 45
- Removed edges: 3
- ADRs at base: 5 | at head: 7

(Optional: list component IDs or top 10.)

## v1.20.0 → v1.21.0
...
```

**JSON** (machine-readable for future UI):

```json
{
  "repo": "gitea",
  "refs": ["main", "v1.20.0", "v1.21.0"],
  "steps": [
    {
      "base_ref": "main",
      "head_ref": "v1.20.0",
      "new_components": 12,
      "removed_components": 0,
      "new_edges": 45,
      "removed_edges": 3,
      "adrs_at_base": 5,
      "adrs_at_head": 7,
      "violations_summary": { "errors": 0, "warnings": 2 }
    },
    ...
  ]
}
```

---

## 3.6 User input review: LLM or auto vs ask only when needed

Every place we might assume “user provides X” is listed below. We prefer: **LLM or auto-detect first; ask user only when strictly necessary.**

| # | Input | Where | Prefer (LLM / auto) | Ask user only when |
|---|--------|--------|----------------------|---------------------|
| 1 | **Refs to capture** | capture_timeline.sh | **LLM** suggests architecture-significant commits from git log (default when no refs). | User passes explicit refs only if they want to override. |
| 2 | **REPO name** (which repo to capture) | capture_timeline.sh | **Auto:** if no REPO arg and CWD is a git repo (e.g. `test-repos/express`), use it; if CWD is `evaluation/real-world-test` and `test-repos/` has exactly one dir, use that; if multiple dirs and LLM key set, **LLM** picks one from list. | Only when multiple repos exist, no LLM key, and no REPO arg → prompt “Which repo? (name from test-repos/)”. |
| 3 | **Timeline dir / repo for report** | timeline_report.sh | **Auto:** if no arg and CWD is `evaluation/real-world-test`, and `timelines/` has exactly one subdir, use it; if multiple and LLM key set, **LLM** picks one. | Only when multiple timeline dirs, no LLM key → prompt “Which timeline? (name from timelines/)”. |
| 4 | **ADR / intent dirs** | intent, adr-index, capture --adr | **Auto-detect:** scan repo for existence of `docs/architecture`, `docs/adr`, `doc/adr` (and optionally `docs/adr/decisions`); use all that exist. No config needed. | Not needed; we don’t ask. |
| 5 | **Whether to capture ADRs** (--adr) | capture_timeline.sh | **Auto:** enable ADR capture when any of the ADR root paths above exist in the repo; user can pass `--no-adr` to disable. | Never. |
| 6 | **Dirty working tree** (continue or abort) | capture_timeline.sh, drift_by_commit.sh | **Auto in non-interactive:** if not a TTY, don’t prompt; abort with clear message. Support `--force` or `SRUJA_FORCE=1` to proceed without prompt. | Only when **interactive** (TTY), dirty tree, and no `--force` → “Continue anyway? [y/N]”. |
| 7 | **Report format** (-f text|json|both) | timeline_report.sh | Default `both`; no user input required. | Never. |
| 8 | **Max refs / commits** (--max-refs, --commits) | capture_timeline.sh | Sensible defaults (e.g. 30, or 20 for LLM cap); no user input required. | Never unless user wants to override. |

**Summary:** Ref list → LLM. Repo/timeline choice → auto (single) or LLM (multiple). ADR dirs and --adr → auto. Dirty tree → prompt only when interactive and no --force. Everything else → defaults; ask only when ambiguous and no LLM.

---

## 4. Components to build

### 4.1 Component A: Timeline capture script

**Deliverable:** `capture_timeline.sh` in `evaluation/real-world-test/`.

**Behavior:**

- **Usage:** `./capture_timeline.sh [REPO] [ref1 ref2 ...]`  
  **REPO** optional: if omitted, use CWD if it’s a git repo, or the only dir in `test-repos/`, or (if multiple) LLM picks one; only prompt when multiple repos and no LLM key (see Section 3.6). **Refs** optional: if omitted, LLM suggests architecture-significant commits (when API key set); else tags + `--max-refs` or `--commits` fallback.
- **Output directory:** All outputs go under `$SCRIPT_DIR/timelines/REPO/` (so from repo root: `evaluation/real-world-test/timelines/REPO/`). When running `sruja scan` the script is in `test-repos/REPO`, so pass **absolute path** for `-o`, e.g. `"$OUT_DIR/graph_<sanitized_ref>.json"` where `OUT_DIR="$(cd "$SCRIPT_DIR" && pwd)/timelines/$REPO_NAME"`.
- **For each ref (in order):**
  1. `git checkout <ref>` (dirty-tree: abort with message if non-interactive or `--force`; only prompt “Continue anyway? [y/N]” when interactive and no `--force` — see Section 3.6).
  2. `sruja scan -r . -o "$OUT_DIR/graph_<sanitized_ref>.json"`.
  3. Optional (flag, e.g. `--adr`): run ADR capture for this ref (see 4.2) and write `"$OUT_DIR/adr_<sanitized_ref>.json"`.
  4. Record ref, sha, timestamp in manifest.
- **After all refs:** Restore original branch/ref. Write or update `$OUT_DIR/manifest.json`.

**Inputs:**

- **REPO** (optional): name under `test-repos/`, or path to a git repo. If omitted → auto-detect (CWD if git repo, or only dir in test-repos, or LLM pick from list); only prompt when multiple repos and no LLM (Section 3.6).
- **Refs** (optional): if empty → **automatically** choose refs (LLM by default when API key set; else tags or `--commits`).
- **Flags:** `--adr` / `--no-adr` (auto: enable ADR capture when ADR dirs exist in repo; override with `--no-adr`), `--no-llm` (skip LLM ref selection), `--max-refs N` (default 30), `--commits N`, `--force` (don’t prompt on dirty tree; abort in non-interactive if not set), `--overwrite` (replace existing timeline dir; default = incremental, append/update refs and manifest).
- **Existing timeline dir:** If `timelines/REPO/` already exists, default behavior is **incremental**: add or update graphs for refs in this run and update manifest. Use `--overwrite` to clear or replace the timeline dir before capturing.
- **Error recovery:** If checkout or scan fails for a ref (e.g. ref doesn’t exist): skip with warning, continue to next ref; optionally record failed refs in manifest (e.g. `failed_refs: ["v1.5"]`) for visibility.

**Smart ref selection (automatic by default, no user input):**

We never capture every commit. When the user **does not** pass refs, we choose a small set of refs automatically. Preferred method is **LLM-based analysis of commit messages** so users don’t have to choose.

| Mode | When | What we capture | How |
|------|------|------------------|-----|
| **Explicit refs** | User passes refs: `./capture_timeline.sh REPO main v1.0 v2.0 HEAD` | Exactly those refs, in order. | No automation; no cap. |
| **Smart (LLM, default)** | No refs, no `--no-llm`, and an LLM API key is set (e.g. in `.env`) | Commits the LLM marks as **architecture-significant**. | See below. |
| **Fallback (tags)** | No refs and `--no-llm`, or LLM unavailable / failed | Default branch + all tags sorted by commit date (oldest first), capped. | `--max-refs N` (default 30). |
| **Fallback (commits)** | No refs, `--commits N` (e.g. `--commits 20`) | Default branch + last N commits. | `git rev-list -n N main`. |

**LLM-based selection (default when no refs given):**

1. **Input to LLM:** A chronologically ordered list of commits from the repo (e.g. last 200–500, or a time-based sample). Format per line: `SHORT_SHA  ISO_DATE  SUBJECT` (e.g. `a1b2c3d  2024-06-15  refactor: split auth into separate module`). No need to send full diff or code.
2. **Prompt:** Ask the LLM to identify commits that likely represent **significant architecture or structure changes** (e.g. refactors, new modules, splits, migrations, new services, major dependency changes). Exclude routine fixes, docs, style. Return a **JSON array of short SHAs** only, in chronological order, **at most 20–25** commits so capture stays fast.
3. **Output:** Parse the JSON; resolve each short SHA to a ref (or use as-is if git accepts it). This list becomes the ref list for capture. If parsing fails or LLM errors, fall back to tag-based selection (or `--commits 15`).
4. **Implementation:** A small CLI entry point (e.g. `sruja timeline suggest-refs -r REPO_PATH`) that: runs `git log` to get the commit list, builds the prompt, calls the same LLM stack as `sruja eval` (Rig + provider resolution from env), parses the model response to a JSON array of SHAs, prints it (or writes to stdout for the capture script). Capture script: when no refs and not `--no-llm`, call this and use the output as the ref list; otherwise use tags or `--commits N`.

**Flags:**

- `--no-llm`: Skip LLM; use tag-based (or `--commits N`) ref selection even if an API key is set.
- `--max-refs N`: Cap for tag-based fallback (default 30). Also cap LLM-suggested list at N if needed.
- `--commits N`: Force “last N commits” mode (no LLM, no tags).

**Default branch:** Resolve once (e.g. `main` or `master` via `git rev-parse -q --verify main \|\| git rev-parse -q --verify master` or `git symbolic-ref refs/remotes/origin/HEAD`). Use for `git log` and for fallback ref lists.

**Outputs:**

- `timelines/REPO/graph_<ref>.json` for each ref.
- `timelines/REPO/adr_<ref>.json` for each ref if `--adr`.
- `timelines/REPO/manifest.json`.

**Dependencies:** `find_sruja` (lib.sh), `sruja scan`. For smart ref selection: same LLM env as `sruja eval` (`.env` with one of OPENROUTER_API_KEY, OPENAI_API_KEY, etc., or SRUJA_LLM_PROVIDER=ollama). ADR capture (4.2) if `--adr`.

---

### 4.2 Component B: ADR capture per ref (and path flexibility)

**Deliverables:**

1. **Script or CLI hook** that, at a given ref (current checkout), discovers and parses ADRs and writes `adr_<ref>.json`.  
   - Option (a): small Rust binary or `sruja` subcommand that outputs ADR index JSON (reads intent dir(s), uses existing `IntentIntelligence` + parser, writes index only).  
   - Option (b): shell script that runs `sruja intent check -r . -i DIR -f json` and, if we extend the JSON output to include “list of ADRs”, parses that and writes `adr_<ref>.json`. Prefer (a) if we want multiple dirs and a clean index schema.

2. **ADR path flexibility** in `sruja-intent` (see Section 3.6, input #4):  
   - **Auto-detect** ADR roots: scan repo for existence of `docs/architecture`, `docs/adr`, `doc/adr`; use all that exist so user doesn’t configure. Optionally allow override via env or flag (e.g. `SRUJA_ADR_DIRS=dir1:dir2`).  
   - For each root, look for: `root/adr/decisions/*.md` (current) and optionally `root/*.md` (flat ADR dir).  
   - No change to parser format; only where we look. Optionally support `--full-adr` in Phase 3 to include full context/decision/consequences in the index for future UI.

**Tasks:**

- [ ] Intent: add support for multiple `-i` directories (merge all) or one env var `SRUJA_ADR_DIRS` (colon-separated).  
- [ ] Intent: for each dir, try `dir/adr/decisions` and optionally `dir` for .md files.  
- [ ] Add “ADR index” export: either JSON output from intent that includes ref + list of ADR metadata, or a small `sruja adr-index -r . -i DIR -o adr.json` that writes the index file.  
- [ ] capture_timeline.sh: when `--adr`, after each checkout run ADR index export and save to `timelines/REPO/adr_<ref>.json`.

---

### 4.3 Component C: Timeline report script/CLI

**Deliverable:** Either a script `timeline_report.sh` or a `sruja timeline` subcommand (or both: script that calls CLI).

**Behavior:**

- **Input:** Repo name or path to timeline dir (e.g. `gitea` or `timelines/gitea`, or `-d timelines/gitea`). **Optional:** if omitted, use only timeline dir when there’s exactly one under `timelines/`, or LLM pick when multiple and API key set; only prompt when multiple and no LLM (Section 3.6).
- **Read** `manifest.json` to get ordered list of refs and graph filenames.
- **For each consecutive pair (ref_i, ref_{i+1}):**
  - Run `sruja drift-diff -b <timeline_dir>/graph_ref_i.json -h <timeline_dir>/graph_ref_{i+1}.json -f json`.
  - Parse JSON: In `drift-diff`, **actual = head** and **proposed = base**. So `summary.missing_components` = components present in head but not in base (= **new at head**); `summary.new_components` = components in base but not in head (= **removed since base**). Map to report schema: report `"new_components"` := summary.missing_components, `"removed_components"` := summary.new_components, `"new_edges"` := summary.new_dependencies, `"removed_edges"` := summary.removed_dependencies. Use `violations` for violations_summary (count by severity).
  - If `adr_<ref_i>.json` and `adr_<ref_{i+1}>.json` exist, count `adrs.length` in each for adrs_at_base / adrs_at_head.
- **Output:**
  - **Markdown:** `timelines/REPO/timeline_<REPO>.md` with sections per step (see 3.5).
  - **JSON:** `timelines/REPO/timeline_<REPO>.json` with `steps` array (see 3.5).
- **Flags:** `-f text` (only markdown), `-f json` (only JSON), `-f both` (default).

**Dependencies:** `sruja drift-diff`, existing graph and manifest layout.

---

### 4.4 Component D: “Play” the timeline

**Deliverable:** No new tool required for “play” in the minimal sense: the timeline report is the play script. User opens `timeline_REPO.md` or consumes `timeline_REPO.json` step by step.

**Optional enhancement:** A tiny “play” printer that reads `timeline_REPO.json` and prints one step per second (or on keypress) to the terminal—nice-to-have, not in initial scope.

**Future:** A UI that visualizes the timeline (e.g. graph at t0, t1, t2 and highlights what changed) is out of scope for this plan; the JSON report is designed to feed such a UI.

---

## 5. Implementation phases

### Phase 1: Timeline capture (snapshots only)

**Goal:** Persist graph JSONs at multiple refs and a manifest.

| Task | Description | Owner |
|------|-------------|-------|
| 1.1 | Define ref sanitization: map ref name → safe filename (e.g. `release/2.x` → `release-2.x`, SHA → 7-char). | - |
| 1.2 | Implement `capture_timeline.sh`: usage, ref list (explicit or from suggest-refs/tags/commits), loop: checkout → scan → write graph + update manifest, restore branch. No ADR yet. | - |
| 1.3 | Create `timelines/REPO/` dir and write `manifest.json` (refs, graph_files, no adr_capture). | - |
| 1.4 | Implement `sruja timeline suggest-refs -r REPO`: git log sample → LLM prompt (architecture-significant commits) → JSON array of SHAs; reuse Rig/LLM from eval. | - |
| 1.5 | capture_timeline.sh: when no refs and has_llm_key and not --no-llm, call suggest-refs and use output as ref list; else fallback to tags + --max-refs or --commits N. | - |
| 1.6 | capture_timeline.sh: REPO optional; when omitted, resolve via CWD / single test-repos dir / LLM pick or prompt (Section 3.6). | - |
| 1.7 | Dirty tree: only prompt when TTY and no --force; else abort with message. Add --force. | - |
| 1.8 | Test on one repo (e.g. express) with explicit refs, then with default (LLM or tag fallback); test with no REPO when CWD is repo. | - |

**Exit criterion:** (1) Running `./capture_timeline.sh express main HEAD` produces `timelines/express/graph_main.json`, `graph_<head>.json`, and `manifest.json`. (2) With an LLM key set, running `./capture_timeline.sh express` (no refs) uses LLM to suggest refs and captures at those points; with `--no-llm` it falls back to tags (or --commits) and produces a valid manifest and graph set.

---

### Phase 2: Timeline report

**Goal:** Generate a single timeline report from stored snapshots.

| Task | Description | Owner |
|------|-------------|-------|
| 2.1 | Implement `timeline_report.sh` (or `sruja timeline -d timelines/REPO`): read manifest, for each consecutive pair run `sruja drift-diff -b ... -h ... -f json`, parse and aggregate. | - |
| 2.2 | Emit `timeline_<REPO>.md` with sections “ref A → ref B” and counts (new/removed components and edges). | - |
| 2.3 | Emit `timeline_<REPO>.json` with `steps` array (base_ref, head_ref, new_components, removed_components, new_edges, removed_edges). | - |
| 2.4 | Single ref: if manifest has only one ref/graph, emit report with empty steps and a short message ("One snapshot only; no steps to compare"). | - |
| 2.5 | timeline_report.sh: timeline dir/repo arg optional; when omitted, use only dir if one under timelines/, else LLM pick or prompt (Section 3.6). | - |
| 2.6 | Test: after Phase 1, run timeline report and open .md / inspect .json. | - |

**Exit criterion:** Given `timelines/gitea/` with at least two graph files, running the report produces markdown and JSON that correctly reflect drift between consecutive refs. With one graph only, report completes without error and states there are no steps.

---

### Phase 3: ADR capture per ref

**Goal:** At each ref, capture ADR index and optionally support multiple ADR dirs.

| Task | Description | Owner |
|------|-------------|-------|
| 3.1 | sruja-intent: auto-detect ADR roots (scan for docs/architecture, docs/adr, doc/adr); support multiple `-i` or `SRUJA_ADR_DIRS` as override. Merge models from all. | - |
| 3.2 | sruja-intent: for each dir, look for `dir/adr/decisions/*.md` and optionally `dir/*.md` (config or convention). | - |
| 3.3 | Add ADR index export: either extend `sruja intent check -f json` to include “adrs”: [ { path, number, title, status, date, tags } ], or add `sruja adr-index -r . -i DIR -o out.json`. | - |
| 3.4 | capture_timeline.sh: auto-enable ADR capture when any ADR root exists in repo; `--no-adr` to disable. When enabled, after each checkout run ADR index export and write `timelines/REPO/adr_<ref>.json`; set `adr_capture: true` in manifest. Optional `--full-adr` to include full context/decision in index. | - |
| 3.5 | Timeline report: if `adr_<ref>.json` exist, include in each step “adrs_at_base”, “adrs_at_head” (and optionally list new/removed ADR titles). | - |

**Exit criterion:** With `--adr`, capture produces adr_<ref>.json for each ref; report includes ADR counts (and optionally delta) per step.

---

### Phase 4: Polish and docs

**Goal:** Robustness, edge cases, and documentation.

| Task | Description | Owner |
|------|-------------|-------|
| 4.1 | Ref selection: when no refs given, sort tags by date (`git log -1 --format=%ci`); add `--max-refs N` (default 30). Add `--commits N` to sample last N commits on default branch when repo has few/no tags. | - |
| 4.2 | Dirty tree: non-interactive or `--force` → abort with message (no prompt). Interactive and no `--force` → prompt “Continue? [y/N]”. | - |
| 4.3 | Handle missing ref (e.g. tag doesn’t exist): skip with warning, continue; optionally log failed refs in manifest. | - |
| 4.4 | Update TEST_ON_REAL_PROJECTS.md: add “Timeline capture and report” subsection with usage of capture_timeline.sh and timeline_report.sh (or sruja timeline), and link to this plan. | - |
| 4.5 | Add README or section in this doc: “Example: capture and play timeline for gitea” (copy-paste commands). | - |
| 4.6 | Consider `--parallel N` (Phase 4+): run checkout+scan for N refs in parallel to speed up capture on large ref lists; document as optional. | - |

---

## 6. CLI changes (summary)

| Change | Type | Description |
|--------|------|-------------|
| Multiple intent dirs | Intent | `-i` repeatable or `SRUJA_ADR_DIRS=dir1:dir2`; look in each for adr/decisions and optionally flat .md. |
| ADR index export | New output | `sruja intent check -f json` extended with adr list, or new `sruja adr-index -r . -i DIR -o out.json`. |
| `sruja timeline suggest-refs` | New | `sruja timeline suggest-refs -r REPO` outputs JSON array of short SHAs (architecture-significant commits) using LLM; used by capture_timeline.sh when no refs given. Reuses Rig/LLM from eval. |
| `sruja timeline` | New subcommand (optional) | `sruja timeline -d timelines/REPO [-f text|json|both]` reads manifest, runs drift-diff for each pair, writes report. If we only do shell script first, this can be Phase 2 follow-up. |

---

## 7. Acceptance criteria

- [ ] **Capture:** With default refs (LLM or tags), `./capture_timeline.sh gitea` or `./capture_timeline.sh` (when REPO is inferrable) produces a valid manifest and at least two graph JSONs under `timelines/<repo>/`.
- [ ] **Report:** Running the timeline report on that directory produces `timeline_gitea.md` and `timeline_gitea.json` whose steps match manual runs of `sruja drift-diff` for the same pairs.
- [ ] **ADR (Phase 3):** With `--adr`, capture produces at least one `adr_<ref>.json` when the repo has ADRs in a supported path; report includes adrs_at_base / adrs_at_head for steps where both adr files exist.
- [ ] **Play:** A human can “play” the timeline by reading the markdown report step by step; a future consumer can parse the JSON for automation/UI.

---

## 8. Risks and dependencies

| Risk | Mitigation |
|------|------------|
| Large number of tags (e.g. thousands) | `--max-refs N` (default e.g. 30); only first N refs after sorting by date. See **Smart ref selection** in Section 4.1. |
| Repo has no tags | Use `--commits N` to sample last N commits on default branch; or pass explicit refs (e.g. `main HEAD~100 HEAD`). |
| ADR dirs vary widely across OSS | Start with docs/architecture and docs/adr; document and make configurable; accept that some repos have no ADRs. |
| Long capture time (many refs, big repo) | Run in background; document expected time; consider `--parallel N` (checkout+scan for N refs in parallel workers) in Phase 4+. |
| sruja CLI not on PATH in script | Reuse find_sruja from lib.sh (already used by drift_by_commit.sh). |
| LLM rate limits or cost | Cap commit sample (e.g. 300) and max suggested refs (20–25); fallback to tags on error. Use same .env as eval. |

**Dependencies:** Existing `sruja scan`, `sruja drift-diff`, `sruja intent check`; git; bash (scripts). No new Rust crates required for Phase 1–2; intent changes in Phase 3 are within sruja-intent and sruja-cli.

---

## 9. Appendix A: Ref selection strategies

See **Smart ref selection** in Section 4.1 for how we avoid capturing too many refs on large repos.

| Strategy | When to use | Implementation |
|----------|-------------|----------------|
| Explicit list | User knows refs | `./capture_timeline.sh gitea main v1.20.0 v1.21.0 HEAD` — no cap. |
| main + tags (default) | “Full” history by releases | `git tag -l`; sort by `git log -1 --format=%ci %(refname:short)`; prepend default branch; apply `--max-refs N`. |
| Last N commits | Repo has few/no tags; want commit-based sample | `./capture_timeline.sh gitea --commits 20` → default branch + `git rev-list -n 20 main`. |
| main + HEAD | Minimal (two points) | Same as drift_by_commit today; capture persists. |

**Default branch:** If `main` does not exist, fall back to `master` (e.g. `git rev-parse -q --verify main || git rev-parse -q --verify master`), or use `git symbolic-ref refs/remotes/origin/HEAD` to infer default branch when available.

---

## 10. Appendix B: Example commands (after implementation)

```bash
# Zero args: REPO and refs both auto (CWD or only repo in test-repos; LLM suggests refs if key set)
./capture_timeline.sh

# Explicit repo only (refs still auto via LLM or tags)
./capture_timeline.sh express

# Skip LLM, use tags + max 30 refs
./capture_timeline.sh gitea --no-llm

# Explicit refs (no LLM, no tag logic)
./capture_timeline.sh etcd main v3.5.0 v3.6.0 HEAD

# ADR capture auto-enabled when docs/architecture or docs/adr exist; disable with --no-adr
./capture_timeline.sh gitea

# Generate timeline report (arg optional: only timeline dir used if one exists)
./timeline_report.sh
./timeline_report.sh gitea
# or: sruja timeline -d timelines/gitea -f both

# View play
cat timelines/gitea/timeline_gitea.md
```

---

## 10.1 Appendix C: LLM suggest-refs prompt template

The `sruja timeline suggest-refs` command uses the following prompt shape (see `crates/sruja-cli/src/commands/timeline.rs`).

**System prompt:**

```
You are helping select git commits that represent significant architecture or structure changes in a codebase.

Given a list of commits (one per line: SHORT_SHA TAB ISO_DATE TAB SUBJECT), identify which commits are likely to represent:
- Refactors, new modules, splits or merges of components
- Migrations, new services, major dependency or layout changes
- Architectural decisions (not routine fixes, docs, style, or typo-only changes)

Reply with ONLY a JSON array of the short SHAs (7-char) in chronological order (oldest first), at most 25 commits. No explanation.
Example: ["a1b2c3d","e4f5g6h"]
```

**User prompt (commit list):**

```
List of commits (SHORT_SHA TAB DATE TAB SUBJECT), chronological (oldest at top):

a1b2c3d	2024-06-01 10:00:00 +0000	refactor: split auth into separate module
e4f5g6h	2024-06-15 12:00:00 +0000	add new payment service
...

Return a JSON array of short SHAs (at most 25) that represent architecture-significant commits, in chronological order. Only the array, no other text.
```

**Note:** Input commits are produced by `git log -N --format=%h%x09%ci%x09%s` (newest-first); the implementation must pass them in **oldest-first** order to the LLM (e.g. reverse the list) so that the model’s “chronological order” matches the manifest’s oldest→newest.

---

## 11. Review and implementation notes

- **Capture output path:** Graphs and manifest must be written under `$SCRIPT_DIR/timelines/REPO/`; when the script `cd`s into `test-repos/REPO`, use an absolute path for `sruja scan -o` (see Section 4.1).
- **drift-diff JSON mapping:** `DiffResult.summary` fields map to timeline report as: `missing_components` → "new at head", `new_components` → "removed since base", `new_dependencies` → new_edges, `removed_dependencies` → removed_edges (see Section 4.3).
- **Timeline report input:** Script/CLI can take repo name (and resolve `timelines/REPO` relative to script or cwd) or an explicit path to the timeline dir. Document that running from `evaluation/real-world-test` allows `./timeline_report.sh gitea` with `timelines/gitea` implied.
- **Default branch:** Repos may use `master` instead of `main`; ref selection should try `main` then `master` or use git’s default (Appendix A).
- **Single ref:** If only one ref is captured, timeline report has no pairs; emit a trivial report (e.g. "One snapshot only; no steps to compare") instead of failing.
- **Intent CLI today:** `sruja intent check -i DIR` accepts a single path; Phase 3 adds multiple `-i` or `SRUJA_ADR_DIRS` in sruja-intent and sruja-cli.
- **LLM suggest-refs:** Prompt must request a **strict JSON array of short SHAs only** (e.g. `["a1b2c3d","e4f5g6h",...]`) so the capture script can parse it without heuristics. Commit list to the model: SHAs + date + subject only (no diff or file list) to keep tokens low and avoid rate limits.
- **Ask only when needed:** Per Section 3.6, prompt the user only when (1) multiple repos/timelines and no LLM key (which repo/timeline?), or (2) interactive TTY and dirty tree and no `--force` (continue anyway?). All other inputs are defaulted or derived from LLM/auto-detect.

---

*End of plan.*

# How to Test Sruja on Various Real Projects

Sruja targets **general customer-facing systems** (web apps, APIs, services), **admin systems** (dashboards, back-office), and **ecommerce systems** in supported languages. Two ways to test:

1. **Fast path (no architecture file)** – Run quickstart and drift on any repo. No `.sruja` file or AI generation.
2. **Full evaluation** – Clone test repos, generate `architecture.sruja` (with AI or by hand), then run the evaluation script.

**Cursor CLI (`agent`):** Testing with the Cursor CLI is **local only** (no CI). See [LOCAL_CURSOR_CLI_TESTING.md](LOCAL_CURSOR_CLI_TESTING.md) for using `agent` in the terminal on cloned repos.

## Testing the demo on real projects (skills + CLI)

Integration is **skills + CLI** only (no MCP, no Sruja-owned LLM). To validate the demo on real projects:

- **CLI:** From `evaluation/real-world-test`, run `./run_demo.sh` (quickstart + drift on Express) or `./run_demo.sh --baseline`. On other repos: `sruja quickstart -r <path>`, `sruja drift -r <path>`, optionally `sruja why "question" -r <path>` and `sruja context -r <path> -f cursor-rules -o /tmp/out.cursorrules`. No API keys required.
- **Editor:** Install the Sruja skill in your editor (Cursor, Copilot, etc.); open a real repo; ask e.g. "What's the state of our architecture?" or "Where are the cycles?" and confirm the AI runs `sruja quickstart` or `sruja drift` and summarizes the output.

Record results (repo, quickstart summary, drift result) in `run_results/` or extend [OSS_TEST_RESULTS.md](OSS_TEST_RESULTS.md). Optional: run `./run_demo_real_projects.sh` to run quickstart + drift on each repo in `test-repos/` and append a one-line summary to a timestamped file in `run_results/`.

### Testing the Sruja skill and slash command on real projects

To validate that the Sruja skill (and slash command) work on real OSS projects:

**Prerequisites**

- Sruja CLI built: `make build` (from Sruja repo root).
- One or more real repos present: run `./setup_repos.sh` or `./setup_repos.sh --complex` from `evaluation/real-world-test`.

**Option A – Skill in repo (recommended for testing)**

1. Run `./prepare_skill_in_real_projects.sh` from `evaluation/real-world-test`. This copies the Sruja architecture skill into each repo under `test-repos/` at `.agents/skills/sruja-architecture/`.
2. Open a test repo in Cursor (or VS Code with Copilot), e.g. `test-repos/express`.
3. In chat, type `/` and select **sruja-architecture**.
4. Ask: “Run sruja quickstart and summarize the architecture state.”
5. **Expected:** The agent runs `sruja quickstart -r .` (or equivalent) and summarizes the output (inventory, health, findings).

**Option B – Global skill**

1. Run `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` (choose **Copy** when prompted so the skill appears under Skills).
2. Open any real repo (e.g. `test-repos/gitea`) in Cursor or VS Code.
3. Same as steps 3–5 above: type `/`, select **sruja-architecture**, and ask for quickstart summary.
4. **Expected:** Same as Option A.

**Optional CLI smoke test**

Before testing in the editor, confirm the CLI works on those repos: run `./run_demo.sh` or `./run_demo_real_projects.sh` from `evaluation/real-world-test`.

---

## What matters most: capture, questions, drift, and commits

**The score is not the goal.** Sruja’s main value is:

| Priority | What | How today | Future |
|----------|------|-----------|--------|
| **1. Capture architecture** | Infer structure from code (modules, deps, services). | `sruja scan -r . -o sruja.graph.json`; quickstart/drift run scan under the hood. | Same; optional persistence per repo. |
| **2. Ask questions** | Explore why the system is structured the way it is. | `sruja why "your question" -r .` (deterministic). Use Sruja skill in editor for AI interpretation. | — |
| **3. Drift** | Compare code vs declared architecture; catch violations. | `sruja drift -r .` (structural only) or `sruja drift -r . -a architecture.sruja` (vs baseline). | **GitHub/commit integration** (see below). |
| **4. Commit / CI integration** | Run capture and drift in CI; tie results to commits; review in PRs. | Not yet. | Run drift on push or PR; report “new violations since base”; optional PR comments. |

**Health score** is **indicative only**—a quick signal, not a grade. Use **inventory + findings + drift vs `architecture.sruja`** for real decisions. Improving the “scoring approach” is secondary to making capture, questions, and drift reliable and usable in commit-based workflows.

### Future: GitHub / commit integration

Planned directions (no implementation yet):

- **CI:** Run `sruja scan` and `sruja drift -r . -a architecture.sruja` on every push or PR; fail or warn on new violations or regressions.
- **Diff by ref:** Compare drift at `HEAD` vs `main` (or another ref) to answer “what got worse since last release?”
- **PR comments:** Post a short drift summary (e.g. new orphans, new cycles, components missing from `architecture.sruja`) as a PR comment.
- **Blame-by-commit:** Associate new violations with the commit that introduced them (e.g. “this cycle appeared in commit abc123”).

Until then, run Sruja locally or in your own CI scripts; use the graph and drift output (JSON or text) as the source of truth, not the health number alone.

### Using repos' commits to calculate the diff

For these test repos (gitea, etcd, caddy, react-admin, saleor, etc.) we have **real git history**. You can use their **commits** to compute the **architectural diff** between two refs (e.g. `main` vs `HEAD`, or `HEAD~10` vs `HEAD`):

1. **CLI:** `sruja drift-diff -b base_graph.json -h head_graph.json` — diffs two scan graph JSONs (e.g. produced at two different commits). Output: new/removed components and edges, and violations.
2. **Script:** From `evaluation/real-world-test`, run:
   ```bash
   ./drift_by_commit.sh REPO [BASE_REF] [HEAD_REF]
   ```
   Example: `./drift_by_commit.sh gitea main HEAD` checks out `main`, runs `sruja scan`, then `HEAD`, runs `sruja scan`, then `sruja drift-diff` on the two graphs and restores your branch. So the diff is **calculated from the repo's commits**.

Manual flow (same idea):

```bash
cd test-repos/gitea
git checkout main
sruja scan -r . -o /tmp/gitea_base.json
git checkout HEAD   # or your branch
sruja scan -r . -o /tmp/gitea_head.json
sruja drift-diff -b /tmp/gitea_base.json -h /tmp/gitea_head.json
```

### Next steps (based on this direction)

| Priority | Step | Owner / notes |
|----------|------|----------------|
| **1** | **CI: run drift on push/PR** | Add a CI job (GitHub Actions, GitLab CI, or similar) that runs `sruja scan` and `sruja drift -r . -a architecture.sruja`; fail or warn when there are new violations or regressions. Start with a single workflow file in `.github/workflows/` (or equivalent). |
| **2** | **Drift diff by ref** | Done: use `sruja drift-diff -b base.json -h head.json` with graphs from two commits; use `./drift_by_commit.sh REPO base_ref head_ref` to compute diff from the repo's commits. |
| **3** | **PR comments (optional)** | After (1) or (2): post a short drift summary as a PR comment (e.g. new orphans, new cycles, components missing from `architecture.sruja`). Use GitHub API or similar; keep the comment compact and link to artifacts if needed. |
| **4** | **Blame-by-commit (later)** | Associate new violations with the commit that introduced them (e.g. bisect or diff graphs between commits). Lower priority; depends on (2) and stable scan output format. |
| **5** | **Docs and discovery** | In main README and RUN_GUIDE: state that primary value is capture + questions + drift + (future) commit integration; score is indicative. Link to this doc for “test on real projects” and “next steps”. |
| **6** | **Optional: persist graph per commit** | In CI or pre-push hook, save `sruja.graph.json` (or a digest) per commit so drift-by-ref and blame don’t require re-scanning old refs. Can be a separate follow-up after (2). |

Start with **(1) CI integration**; **(2)** is in place for using commits to calculate the diff.

### ADR capture, architecture snapshots over time, and timeline

**Yes, it’s possible** to use the same OSS repos to:

| Goal | What it means | What exists today | What to add |
|------|----------------|-------------------|-------------|
| **Capture ADRs** | Discover and parse ADRs (and .sruja) at a given ref | `sruja intent check -r . -i docs/architecture` loads from `docs/architecture` (and `docs/architecture/adr/decisions` for .md ADRs). Parsed ADRs have title, status, date, context, decision, consequences, implications. | Run at **multiple refs**: checkout ref → load intent dir → export index (ref, path, title, status, date) so we have “ADRs at each point in time.” Optional: support more paths (e.g. `docs/adr/`) if OSS repos use them. |
| **Architecture snapshots at different points** | Store a scan graph (and optionally intent) per ref (e.g. per tag or every N commits). | Single-ref scan and drift-diff between two refs (see above). | **Timeline capture**: for a list of refs (e.g. `main`, tags, or commit list), checkout each ref → `sruja scan -r . -o timelines/REPO/<ref>.json` (or `<tag>_<short-sha>.json`). Persist graphs so we don’t re-scan every time. |
| **Play the timeline** | Show how architecture (and optionally ADRs) evolved: “at v1.0: N modules; at v2.0: +X, -Y; at v3.0: …” | `sruja drift-diff` between two graphs. | **Timeline report**: given a dir of snapshot JSONs (ordered by ref/date), run drift-diff between consecutive pairs and emit a single report (markdown or JSON): for each step, “ref A → ref B: new components …, removed …, new edges …, removed ….” Optional: simple “play” (print or export each step), or feed the report to a timeline UI later. |

**Concrete approach on OSS repos**

1. **Pick refs** — e.g. `main` + all tags, or `main` + last N commits, or a fixed list (e.g. `v1.0`, `v2.0`, `HEAD`).
2. **Capture snapshots** — For each ref: checkout → `sruja scan -r . -o timelines/REPO/<ref>.json`. Optionally at each ref run intent load and save an ADR index (ref, list of ADR paths/titles/dates).
3. **Capture ADRs per ref** — At each ref, list/parse `docs/architecture` (and if we add: `docs/adr/`, `doc/adr/`). Save `timelines/REPO/adr_<ref>.json` (or a single index with ref → list of ADRs).
4. **Play the timeline** — Script or CLI: for consecutive pairs in the ref list, run `sruja drift-diff -b timelines/REPO/ref1.json -h timelines/REPO/ref2.json` and append to a timeline report. Result: one document or JSON that describes “what changed” between each pair (and optionally “which ADRs existed at each ref”).

**What to build next**

- **Timeline capture script** (e.g. `capture_timeline.sh REPO [ref1 ref2 ...]`): if no refs given, use `git tag -l` and `main`; for each ref, checkout, scan, save graph to `timelines/REPO/<ref>.json`. Optionally call intent and save ADR index.
- **Timeline report** (e.g. `timeline_report.sh REPO` or `sruja timeline -r . -d timelines/REPO`): read ordered refs from stored filenames, run drift-diff for each consecutive pair, output `timeline_REPO.md` (or JSON) with “ref A → ref B: …”.
- **ADR path flexibility**: allow intent dir to be `docs/adr` or multiple dirs so OSS repos that don’t use `docs/architecture/adr/decisions` are still covered.

Once we have stored snapshots and a timeline report, “play” can be as simple as stepping through the report; a future UI could visualize it.

**Detailed plan:** See [TIMELINE_PLAN.md](./TIMELINE_PLAN.md) for phases, data layout, component specs (capture script, timeline report, ADR path flexibility), and acceptance criteria.

### Timeline capture and report (implemented)

From `evaluation/real-world-test` you can capture architecture snapshots at multiple refs and generate a timeline report:

| Step | Command | Notes |
|------|--------|--------|
| **Capture** | `./capture_timeline.sh [REPO] [ref1 ref2 ...]` | If no refs: uses LLM to suggest architecture-significant commits (when API key in `.env`), or falls back to default branch + tags. Use `--no-llm` for tag-only; `--max-refs N`, `--commits N`, `--force` as needed. |
| **Report** | `./timeline_report.sh [REPO] [-f text\|json\|both]` | Reads `timelines/REPO/manifest.json`, runs drift-diff for each consecutive pair, writes `timeline_<REPO>.md` and `.json`. |
| **Play** | `cat timelines/<REPO>/timeline_<REPO>.md` or consume the JSON | Human-readable steps or machine-readable for a future UI. |

**Examples:**

```bash
# Explicit refs (no LLM)
./capture_timeline.sh express master HEAD

# Auto refs (LLM suggests commits when .env has API key)
./capture_timeline.sh express

# Tag fallback
./capture_timeline.sh express --no-llm --max-refs 5

# Generate report
./timeline_report.sh express
```

Requires: `sruja` CLI built (`make build`), `jq`. Optional: `.env` with one LLM API key for smart ref selection. See [TIMELINE_PLAN.md](./TIMELINE_PLAN.md) for full behavior and options.

---

## Two repo tiers: quick vs complex systems

| Tier | Purpose | Repos |
|------|--------|--------|
| **Quick** (default) | Fast clones, demos, CI. Frameworks and libraries. | express, fastapi, next.js, prometheus, django |
| **Complex** | Customer-facing, **admin**, **ecommerce**, and multi-component systems in **supported languages** (Go, JS, Python, TS). Use these to validate Sruja on real product-like architecture. | gitea, etcd, caddy, temporal, minio, **react-admin**, **saleor** |

- **Quick** – Good for “does Sruja run?” and quick demos. Smaller codebases, framework-style structure.
- **Complex** – Good for “does Sruja capture real system architecture?” Includes **admin** (react-admin: dashboard/CRUD framework), **ecommerce** (saleor: headless ecommerce platform), plus Gitea (self-hosted Git), etcd (distributed KV), Caddy (web server), Temporal (workflow engine), MinIO (object storage). All are in Go, TypeScript, or Python so scan results are meaningful. We avoid C-only repos because Sruja does not yet parse C.

Setup:

```bash
./setup_repos.sh           # Quick set only (default)
./setup_repos.sh --complex # Complex systems only
./setup_repos.sh --all     # Both sets
```

---

## End-to-end test on selected OSS systems (step-by-step)

Use these **basic shell commands** in order. No scripts—just copy, paste, and run each block from the `evaluation/real-world-test` directory.

**Selected OSS systems for E2E:** gitea, etcd, caddy (complex systems; all Go). You can also run quickstart/drift on **react-admin** (admin) and **saleor** (ecommerce) after `./setup_repos.sh --complex`.

### 1. Build Sruja CLI (from repo root, once)

```bash
cd /path/to/sruja
make build
```

### 2. Go to evaluation dir and clone the OSS repos

```bash
cd evaluation/real-world-test
./setup_repos.sh --complex
```

### 3. Ensure `sruja` is available

If you didn’t install the CLI on PATH, use the binary directly and set a variable (then use `$SRUJA` in the next steps):

```bash
SRUJA=../target/release/sruja
# Or if sruja is on PATH:
# SRUJA=sruja
$SRUJA --version
```

### 4. Run quickstart on each selected system

```bash
$SRUJA quickstart -r test-repos/gitea
```

```bash
$SRUJA quickstart -r test-repos/etcd
```

```bash
$SRUJA quickstart -r test-repos/caddy
```

### 5. Run drift on each

```bash
$SRUJA drift -r test-repos/gitea
```

```bash
$SRUJA drift -r test-repos/etcd
```

```bash
$SRUJA drift -r test-repos/caddy
```

### 6. (Optional) Run full analyze on one system

```bash
$SRUJA analyze -r test-repos/gitea
```

### 7. (Optional) If you have an `architecture.sruja` in a repo, evaluate it

For example after generating one (e.g. with AI) in `test-repos/gitea/architecture.sruja`:

```bash
./evaluate_architecture.sh gitea
```

Evaluation is validation + checklist only (no LLM). For AI-assisted review, use the Sruja skill in your editor.

---

**Summary:** 1) `make build` → 2) `./setup_repos.sh --complex` → 3) set `$SRUJA` → 4) quickstart on gitea, etcd, caddy → 5) drift on gitea, etcd, caddy → 6) optional analyze → 7) optional evaluate if you have a generated `.sruja`.

---

## Are these results actually correct?

**Short answer:** They are correct for **what the scanner sees**. We target **general customer-facing systems** in supported languages (Go, JS, Python, TS, Rust) so results are meaningful. The scanner does **not** parse C/C++; we do not recommend using Sruja’s scan on C-only repos.

### What the scanner supports

Sruja’s scan today only parses **TypeScript, JavaScript, Python, Go, and Rust**. It does **not** parse C, C++, or other languages. It also uses **npm** (package.json) and **Cargo** (Cargo.toml) when present. The test set (gitea, etcd, caddy, temporal, minio, react-admin, saleor) includes **customer-facing**, **admin**, and **ecommerce** systems in these languages.

### etcd and caddy (Go)

- **Reality:** Both are **Go** repos, so the scanner **does** parse the main code.
- **What Sruja reported:** Many “orphan” modules (no incoming/outgoing dependencies), health 0/100.
- **Interpretation:** Partially correct. In Go repos, **doc packages** (`doc.go`), **tools** (`tools/`), **benchmarks**, and **optional/build-tag** code often have few or no edges in the dependency graph. The scanner treats each package/file as a node; if it doesn’t see imports to/from them, it flags them as orphans. So:
  - Some reported “orphans” are real (e.g. unused or doc-only).
  - Others are **false positives** (e.g. tools, platform-specific files, or packages that are wired at runtime or via build tags).
  - A **0/100 health** score is often driven by counting many such “orphans” and is **overly harsh** for a healthy OSS Go repo. Use the report as a signal (e.g. “lots of tooling/doc packages”) rather than a literal grade. **Update:** Scoring is now red-flag focused (cycles, layer violations); tests/examples/tools/doc paths are excluded from violation counting.

### How to use the results

- **Go/JS/TS/Python/Rust repos:** Quickstart and drift are meaningful for the **supported languages**; health and findings are indicative but can be strict (e.g. orphans).
- **C/C++-heavy repos (e.g. Redis):** Results only reflect the **non-C parts** (e.g. Python/JS); do not treat them as the architecture of the main system.
- **Best use:** Compare relative health across similar repos, spot obvious issues (e.g. huge “god modules”), and use drift + a hand-written or AI-generated `architecture.sruja` for baseline comparison. Do not rely on the raw health number alone for “is this project good?”

---

## How to address these gaps

Concrete ways to improve correctness and usefulness of results:

### 1. Add C (and optionally C++) to the scanner

**Gap:** C/C++-only repos are not analyzed (scanner supports TS/JS/Python/Go/Rust only). We focus the test set on customer-facing systems in supported languages; adding C would allow scanning C-heavy codebases if needed.

**Where:** `crates/sruja-scan/src/tree_sitter/detector.rs` (add `Language::C`), and a new parser e.g. `crates/sruja-scan/src/tree_sitter/languages/c.rs`.

**Steps:**

- Add `Language::C` (and `Cpp` if desired) in `detector.rs` and extend `detect_language()` for `.c`, `.h`, and optionally `.cpp`, `.cc`, `.cxx`.
- Add dependency `tree-sitter-c` (and `tree-sitter-cpp` if needed) in `crates/sruja-scan/Cargo.toml`.
- Implement a C parser that extracts “modules” (e.g. by file or by translation unit) and dependencies from `#include` (and, for C++, also consider other dependency edges). Expose the same `ParsedFile`-style output so the rest of the pipeline stays the same.
- In `tree_sitter.rs`, call the C parser when `detect_language` returns C (and C++ if added).

**Note:** C has no single “import” model like Go/JS; you may treat each `.c`/`.h` as a node and edges as `#include` or use a coarser unit (e.g. directory or component). This is more work than adding a new language that already has clear imports.

### 2. Make orphan detection and health score less harsh (Go/JS doc and tools)

**Gap:** Doc packages, tools, and test helpers are flagged as orphans and can drive health to 0/100.

**Where:** `crates/sruja-diff/src/drift.rs` (`find_orphan_modules` and where violations are built).

**Options (pick one or combine):**

- **Exclude by path pattern:** When building the orphan list, skip nodes whose `path` (from `graph.nodes`) matches patterns that are usually non-product code, e.g.:
  - `doc.go`, `_doc.go`, paths containing `/doc/`
  - `*_test.go`, `*_test.py`, paths containing `/test/`, `/tests/`
  - Paths containing `/tools/`, `/cmd/` (optional), `/vendor/`, `/third_party/`
  - Paths ending in `_test.rs` or in `/examples/`
  Use a small allowlist of patterns (e.g. `tools/`, `doc.go`, `_test.go`) so the rule is predictable and documented.
- **Cap orphan penalty:** In `health.rs` or in the drift layer, cap how much “orphan” violations can subtract (e.g. at most 30 points from orphans, so the rest of the score still reflects cycles and layer violations). That way a repo with 100 doc packages doesn’t go to 0 from orphans alone.
- **Downgrade severity for likely-doc/tool orphans:** If a node matches the same path patterns above, emit it as `Severity::Info` instead of `Warning`, so it affects the score less (e.g. 2 points instead of 5).

**Recommendation:** Start with “exclude by path pattern” for clear cases (`doc.go`, `*_test.go`, `tools/`), and optionally add a cap on total orphan penalty so health stays interpretable.

### 3. Surface “supported languages” and partial-scan warnings

**Gap:** Users don’t know that only TS/JS/Python/Go/Rust (and eventually C) are analyzed, so they may trust results for C-only repos.

**Where:** CLI output (e.g. quickstart/drift in `crates/sruja-cli`) and/or scan summary.

**Steps:**

- After scanning, compute a rough “language mix” (e.g. count of files or nodes per extension or language). If the repo is mostly `.c`/`.h` and the scanner doesn’t support C yet, print a one-line warning: “This repo appears to be mostly C; Sruja currently only parses TS/JS/Python/Go/Rust. Results may not represent the main codebase.”
- In `sruja quickstart` or `sruja drift` output, add a short line like “Parsed: TypeScript, JavaScript, Python, Go, Rust” (and “C” when added) so support is explicit.

### 4. Evaluation and docs

- **TEST_ON_REAL_PROJECTS.md:** Keep the “Are these results actually correct?” and “How to address these gaps” sections up to date as you add C or change scoring.
- **README / docs:** In the “Architecture Intelligence” or “Quickstart” section, state which languages are supported and that health scores can be strict for repos with many doc/tool packages.
- **Real-world test set:** The complex set (gitea, etcd, caddy, temporal, minio, react-admin, saleor) includes customer-facing apps, admin systems, ecommerce, and multi-component systems in supported languages. C-only repos are not included.

**Priority order:** (2) orphan filtering/cap gives immediate benefit for Go/JS/Python/Rust repos with low risk. (3) warnings and docs improve interpretation. (1) C support is optional and only needed if you want to scan C-heavy codebases. (4) keeps the evaluation and docs aligned with behavior.

---

## Fast path: Test on any repo (no .sruja needed)

You can run Sruja’s **architecture intelligence** on any directory. No config, no API keys, no generated file.

### Using the quick set (frameworks)

```bash
# From Sruja repo root: ensure CLI is built
make build

# From evaluation/real-world-test
cd evaluation/real-world-test
./setup_repos.sh          # Clones express, fastapi, next.js, prometheus, django

# Run quickstart on one repo (inventory, health score, findings)
sruja quickstart -r test-repos/express
sruja quickstart -r test-repos/fastapi
sruja drift -r test-repos/express
```

### Using complex systems (recommended for real architecture testing)

```bash
cd evaluation/real-world-test
./setup_repos.sh --complex   # gitea, etcd, caddy, temporal, minio, react-admin, saleor

# Run on customer-facing, admin, ecommerce, and multi-component systems (all in supported languages)
sruja quickstart -r test-repos/gitea
sruja quickstart -r test-repos/etcd
sruja quickstart -r test-repos/react-admin   # admin/dashboard
sruja quickstart -r test-repos/saleor        # ecommerce
sruja drift -r test-repos/gitea
sruja analyze -r test-repos/caddy
```

If `sruja` is not on PATH, use the full path from repo root:

```bash
../../target/release/sruja quickstart -r test-repos/gitea
```

### Using your own projects

Point Sruja at any directory (your app, another OSS clone, etc.):

```bash
sruja quickstart -r /path/to/any/repo
sruja drift -r /path/to/any/repo
sruja analyze -r /path/to/any/repo   # Full analysis (structural + semantic + intent)
```

Example:

```bash
cd ~/projects/my-node-api
sruja quickstart -r .
sruja drift -r .
```

---

## Full evaluation: Generate architecture and evaluate

This flow tests “how good is a **generated** `.sruja` file?” for a real project. You need an `architecture.sruja` in the repo (created by AI skills or manually), then you run the evaluation script.

### Step 1: Clone test repos

```bash
cd evaluation/real-world-test
./setup_repos.sh           # Quick set (frameworks)
./setup_repos.sh --complex # Complex systems (gitea, etcd, caddy, temporal, minio)
./setup_repos.sh --all     # Both
```

**Quick set** (default):

| Repo       | Language   | Description                          |
|-----------|------------|--------------------------------------|
| express   | JavaScript | Node.js web framework                |
| fastapi   | Python     | API framework                        |
| next.js   | TypeScript | React full-stack framework           |
| prometheus| Go         | Monitoring / time series             |
| django    | Python     | Web framework                        |

**Complex systems** (`--complex`):

| Repo         | Language   | Description                                           |
|--------------|------------|-------------------------------------------------------|
| gitea        | Go         | Self-hosted Git service: web UI, API, SSH/HTTP (customer-facing) |
| etcd         | Go         | Distributed KV, Raft consensus, gRPC/HTTP API        |
| caddy        | Go         | Pluggable web server and reverse proxy               |
| temporal     | Go         | Workflow engine: frontend, history, matching, worker  |
| minio        | Go         | S3-compatible object storage, erasure coding         |
| **react-admin** | TypeScript | Admin/dashboard framework: CRUD, auth, data providers |
| **saleor**   | Python     | Headless ecommerce platform: GraphQL API, dashboard, checkout |

Manifest: `test-repos/MANIFEST.md`.

### Step 2: Generate architecture.sruja per repo

For each repo you want to evaluate, create a file **`architecture.sruja`** in that repo’s root.

**Option A – AI in Cursor/VS Code (recommended)**

1. Add the Sruja architecture skill (if not already):
   ```bash
   npx skills add sruja-ai/sruja --skill sruja-architecture-agent
   ```
2. Open the repo:
   ```bash
   code test-repos/express   # or cursor test-repos/express
   ```
3. In the AI chat, use a prompt like:
   ```
   Analyze this codebase and generate a Sruja architecture DSL file.
   Save it as architecture.sruja in the repository root.

   It should capture:
   - Main components and modules
   - Key data flows and relationships
   - External dependencies
   - Technology choices

   Follow Sruja DSL: descriptions for every component, technology for containers,
   specific relationship labels, proper nesting. Use double quotes for all strings.
   ```
4. Save the model’s output as `test-repos/express/architecture.sruja`.
5. Repeat for other repos (e.g. fastapi, django) if desired.

**Option B – Manual / paste**

- Draft or paste a `.sruja` file and save it as `architecture.sruja` in the repo root.
- Validate: `sruja lint path/to/repo/architecture.sruja`.

### Step 3: Run evaluation

From `evaluation/real-world-test`:

```bash
# By repo name (under test-repos/)
./evaluate_architecture.sh express
./evaluate_architecture.sh fastapi
./evaluate_architecture.sh gitea
./evaluate_architecture.sh react-admin
./evaluate_architecture.sh saleor
./evaluate_architecture.sh temporal

# By path (any directory that contains architecture.sruja)
./evaluate_architecture.sh /path/to/repo

./evaluate_architecture.sh gitea
```

The script will:

- Print file stats (lines, systems, containers, relationships).
- Run `sruja lint` on `architecture.sruja`.
- Show a manual checklist (completeness, accuracy, clarity, usefulness).
- Evaluation is validation + checklist (no LLM).
- Write a report under `results/evaluation_<repo>_<timestamp>.md`.

### Step 4: Review reports

```bash
ls results/
cat results/evaluation_express_*.md
```

Use the checklist and scores to decide how useful the generated architecture is (e.g. ≥7/10 = useful).

---

## One-command demo (fast path only)

To run the built-in demo (quickstart + drift on Express, no generation):

```bash
cd evaluation/real-world-test
./run_demo.sh
```

Flags:

- `./run_demo.sh`          – quickstart + drift only.
- `./run_demo.sh --baseline` – add drift vs example architecture.
- `./run_demo.sh --baseline` – drift vs example architecture.

---

## Testing on repos not in the default list

### Quickstart/drift (no .sruja)

Clone or use any path, then:

```bash
sruja quickstart -r /path/to/other/repo
sruja drift -r /path/to/other/repo
```

### Full evaluation (with architecture.sruja)

1. Clone the repo somewhere (e.g. `test-repos/my-app` or `~/repos/some-project`).
2. Generate or add `architecture.sruja` in that repo’s root.
3. Run:
   ```bash
   ./evaluate_architecture.sh /path/to/repo
   ./evaluate_architecture.sh /path/to/repo
   ```

### Adding a repo to the setup list

Edit `setup_repos.sh`: add an entry to `REPOS_QUICK` (frameworks) or `REPOS_COMPLEX` (real systems) and a matching entry in `REPO_META`, then run `./setup_repos.sh` or `./setup_repos.sh --complex` again. New repo will appear under `test-repos/` and you can use it by name: `./evaluate_architecture.sh <name>`.

---

## Checklist summary

| Goal                         | Command / step |
|-----------------------------|----------------|
| Scan quick (framework) repo | `./setup_repos.sh` then `sruja quickstart -r test-repos/express` |
| Scan complex system         | `./setup_repos.sh --complex` then `sruja quickstart -r test-repos/gitea` (or react-admin, saleor, etcd, caddy, …) |
| Scan your own project       | `sruja quickstart -r /path/to/your/repo` |
| Run demo (no .sruja)        | `./run_demo.sh` (uses express from quick set) |
| Evaluate generated .sruja   | Create `architecture.sruja` in repo, then `./evaluate_architecture.sh <repo-name-or-path>` |
| AI-assisted evaluation     | Use the Sruja skill in your editor (Cursor, Copilot, etc.) |

---

## Troubleshooting

- **“sruja: command not found”**  
  Build from Sruja root: `make build`. Use `./target/release/sruja` or add it to PATH; or from `evaluation/real-world-test`: `../target/release/sruja quickstart -r test-repos/express`.

- **“No architecture.sruja found”**  
  Full evaluation requires a file named `architecture.sruja` in the repo root. Use the “Generate architecture” step above or run only quickstart/drift (no evaluation script).

- **“Repository not found”**  
  For named repos, run `./setup_repos.sh` first. For custom paths, use the full path: `./evaluate_architecture.sh /full/path/to/repo`.

- **Evaluation:** Sruja CLI does not use LLM; run `./evaluate_architecture.sh <repo>` for validation and checklist.

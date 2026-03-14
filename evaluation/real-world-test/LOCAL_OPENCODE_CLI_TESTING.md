# Local testing with OpenCode CLI (`opencode`)

This guide mirrors `LOCAL_CURSOR_CLI_TESTING.md`, but uses **OpenCode CLI** instead of Cursor CLI.

Goal: run an LLM-driven architecture analysis on **real cloned repos**, generate `architecture.sruja`, then validate/evaluate using the Sruja CLI and evaluation scripts.

## Prerequisites

- **OpenCode CLI** installed and in PATH (`opencode`)
- **An LLM provider configured for OpenCode** (API key / local model) per your OpenCode setup
- **Sruja CLI** for validation:
  - From this repo: `make build` → `./target/release/sruja`, or
  - Installed `sruja` on PATH

Optional (recommended):
- **Sruja skills present in the repo you analyze** (so the agent sees the DSL conventions):
  - Copy `skills/sruja-architecture` into the target repo under `.agents/skills/` (or equivalent), or
  - Embed the rules/prompt in your OpenCode prompt (shown below).

## 1. Clone test repos (local)

From this directory (`evaluation/real-world-test`):

```bash
./setup_repos.sh            # express, fastapi, next.js, prometheus, django
./setup_repos.sh --complex  # larger set
./setup_repos.sh --apps     # realistic applications: gitea, saleor, documenso, cal.com
```

Repos are cloned under `test-repos/<name>/`.

## 2. Run OpenCode analysis to generate `architecture.sruja`

### Option A (recommended): Use the helper script

From the repo root:

```bash
./scripts/run_opencode_in_repo.sh express "$(pwd)/evaluation/real-world-test/test-repos/express"
```

This runs `opencode` inside the repo directory and asks it to:
- read `AGENTS.md` (if present),
- explore the codebase,
- generate `architecture.sruja` in the repo root.

Notes:
- This script expects `opencode` to support reading/writing files in the working directory.
- If you want stronger guidance (Sruja DSL conventions), add the skill files into the repo (see Option C), or use Option B with an explicit prompt.

### Option B: Run OpenCode manually with an explicit “super prompt”

```bash
cd evaluation/real-world-test/test-repos/express
opencode
```

Then paste:

> Analyze this codebase and generate a Sruja architecture file `architecture.sruja`.
> Be thorough: identify main systems/entry points, containers, key components, and relationships.
> Requirements:
> - 10–30 elements (containers + major components)
> - Every element has `description`
> - Every container has `technology`
> - Relationships are `A -> B "label"` with specific labels (protocol + purpose)
> - Run `sruja lint architecture.sruja` and fix until it passes
> Save the file in the repo root as `architecture.sruja`.

### Option C: Make the Sruja skill visible to OpenCode (per-repo)

If OpenCode can read local skill files, copy the agent skill into the repo:

```bash
cd evaluation/real-world-test/test-repos/express
mkdir -p .agents/skills
cp -r /Users/dilipkola/Workspace/sruja/skills/sruja-architecture .agents/skills/
```

Then run OpenCode (Option A or B) and reference the skill’s guidance in your prompt (e.g. “use sruja-architecture rules”).

## 3. Validate output (mandatory)

From the cloned repo:

```bash
cd evaluation/real-world-test/test-repos/express
/Users/dilipkola/Workspace/sruja/target/release/sruja lint architecture.sruja
```

Or, if you have `sruja` on PATH:

```bash
sruja lint architecture.sruja
```

Optional drift check:

```bash
sruja drift -r . -a architecture.sruja
```

## 4. Evaluate usefulness (scoring + checklist)

If you want the repo-local scoring harness used in `evaluation/results/*`:

1. Copy your `architecture.sruja` into an evaluation results folder (or create one):
   - `evaluation/results/<project>/architecture.sruja`
2. Run:

```bash
./scripts/evaluate_agent_output.sh <project>
```

Alternative: use the manual checklist evaluator:

```bash
./evaluation/real-world-test/evaluate_architecture.sh /absolute/path/to/repo
```

## 5. Running a Mermaid vs Sruja “quality comparison” with OpenCode

To mirror the Cursor CLI experiment (Mermaid baseline vs Sruja with skill), do two runs on the **same repo**:

1. **Mermaid baseline (no Sruja)**: ask OpenCode to output `architecture.mmd` (Mermaid flowchart/subgraphs capturing systems and relationships).
2. **Sruja run (with skill/prompt)**: ask OpenCode to output `architecture.sruja` with the Sruja rules above.

Then compare:
- `architecture.mmd` (visual clarity) vs
- `architecture.sruja` (structured, lintable, driftable)

You can reuse:

```bash
./scripts/summarize_comparison.sh evaluation/results/<comparison_dir>
```

if you save outputs into a comparison directory shaped like:
- `baseline.mmd`
- `sruja.sruja`

## Notes / known gaps

- Some repo scripts under `scripts/` were originally written for the Cursor/Codex `task` tool. For OpenCode, prefer `scripts/run_opencode_in_repo.sh` and manual prompts.
- If OpenCode can’t write files directly, copy/paste the generated DSL into `architecture.sruja` and then run `sruja lint`.

# Local testing with Cursor CLI (`agent`)

**Cursor CLI runs only on your machine.** The `agent` command is for local use only — there is no CI or remote execution. Use it to test Sruja skills on cloned repos locally.

## Using Sruja skills on any repo (any folder)

You can run the Cursor CLI with Sruja skills on **any repo in a completely different folder** — you do not need to be inside the Sruja repo or `test-repos/`.

- **Global skill (recommended):** Run once:  
  `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`  
  After that, the agent has the skill no matter which directory you're in. So you can:
  ```bash
  cd ~/projects/my-app    # or any path
  agent -p "Analyze this codebase and generate architecture.sruja using Sruja DSL. Use sruja-architecture. Run sruja lint on the result."
  ```
- **Per-repo skill:** If you don't use the global install, copy the Sruja skill into that other repo (e.g. `cp -r /path/to/sruja/skills/sruja-architecture /path/to/your-repo/.agents/skills/`), then `cd` to that repo and run the agent. The agent will see the skill when run in that folder.

The `test-repos/` in this directory are only one option for testing; the same flow works for any project directory.

## Prerequisites

- **Cursor CLI** installed (`agent` in your PATH). Install: `curl https://cursor.com/install -fsS | bash`
- **Sruja skills** available to the agent (see below)
- **Sruja CLI** (optional but recommended for validation): `make build` in the Sruja repo, or `cargo install --path crates/sruja-cli`

## Run tests (Sruja repo)

Before or after using the Cursor CLI, ensure the Sruja codebase tests pass. From the **Sruja repo root**:

```bash
make test
# or: cargo test --workspace
```

This runs all Rust unit and integration tests (language, export, CLI, scan, intent, etc.). Optional: `cargo test -p sruja-cli --test why_e2e` for the Why command E2E.

## 1. Clone test repos (local)

From this directory (`evaluation/real-world-test`):

```bash
./setup_repos.sh          # Quick set: express, fastapi, next.js, prometheus, django
./setup_repos.sh --complex   # Larger set: gitea, etcd, caddy, temporal, minio, react-admin, saleor
./setup_repos.sh --all    # Both
```

Repos are cloned under `test-repos/<name>/`.

## 2. Make Sruja skills available to the agent

The Cursor CLI uses the same skills/rules as Cursor IDE. Ensure the agent can see Sruja:

**Option A – Global skill (simplest)**  
If you already ran `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`, the agent will have the skill in scope when you run it from any directory.

**Option B – Per-repo rules**  
Copy Sruja rules into the clone so the agent gets DSL context when you run it inside that repo:

```bash
# From evaluation/real-world-test
./prepare_skill_in_real_projects.sh
```

That script copies the Sruja architecture skill into each `test-repos/<name>/` so opening or running the agent in that directory uses it.

## 3. Run the agent locally (in a cloned repo)

**Interactive** (agent asks for approval for edits/commands):

```bash
cd test-repos/express
agent
# Then in the agent prompt type, for example:
# "Analyze this codebase and generate architecture.sruja using Sruja DSL. Use the sruja-architecture skill. Run sruja lint on the result."
```

**Single prompt (non-interactive)** — good for a quick local test:

```bash
cd test-repos/express
agent -p "Analyze this codebase and generate a Sruja architecture DSL file (architecture.sruja). Use the sruja-architecture skill: identify main systems, containers, and relationships; then run 'sruja lint architecture.sruja' to validate."
```

The agent will read the repo, use the skill, and (if approved) run commands. Output and any generated `architecture.sruja` stay on your machine.

## 4. Validate output locally

After the agent run:

```bash
# From the clone (e.g. test-repos/express)
sruja lint architecture.sruja
sruja drift -r . -a architecture.sruja   # optional: compare code vs baseline
```

Use the same Sruja CLI from the Sruja repo if it’s on your PATH:  
`/path/to/sruja/target/release/sruja lint architecture.sruja`

**Optional – full evaluation (stats + lint + checklist):** From `evaluation/real-world-test`, run  
`./evaluate_architecture.sh express`  
(requires `architecture.sruja` in the repo; run `./run_demo.sh --baseline` first to copy an example into express.)

## 5. Test discovery flow (contextual questions, then generate)

To verify the **contextual discovery** flow (agent gathers repo context, asks tailored questions, then generates architecture):

1. **CLI (no agent):** From the Sruja repo, build and run:
   ```bash
   cargo build -p sruja-cli
   ./target/debug/sruja discover                    # question bank
   ./target/debug/sruja discover --context -r .     # repo context for current dir
   ./target/debug/sruja discover --context -r test-repos/express   # context for a clone
   ```
2. **Agent test script:** From `evaluation/real-world-test` (run with bash):
   ```bash
   bash run_discovery_agent_test.sh --dry-run       # print prompt and repo path
   bash run_discovery_agent_test.sh                 # run agent in test-repos/express (interactive)
   # or, if executable: ./run_discovery_agent_test.sh
   ```
   The prompt tells the agent to run `sruja discover --context -r .` first, then list contextual questions, then generate `architecture.sruja` and run `sruja lint`. Ensure the sruja-architecture skill is installed (`npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`) and `sruja` is on PATH or the script will warn.

3. **Manual on any GitHub clone:** `cd` into any cloned repo, then:
   ```bash
   sruja discover --context -r .
   agent -p "Use sruja-architecture. Run sruja discover --context -r . then list 2-3 contextual questions for this repo, then generate architecture.sruja and run sruja lint until it passes."
   ```

## 6. Test and observe (automated flow)

To run the full local flow and capture results without the interactive agent:

```bash
# From evaluation/real-world-test
./run_test_and_observe.sh          # clone (if needed), prepare_skill, run_demo --baseline, evaluate_architecture express, quickstart fastapi
./run_test_and_observe.sh --no-clone   # same but skip clone; use existing test-repos
```

Observations are written to `run_results/test_and_observe_<timestamp>.md`. Use this to verify everything works, then iterate on improvements.

## Summary

| Step | What |
|------|------|
| **Run tests** | From Sruja repo root: `make test` or `cargo test --workspace` |
| Clone | `./setup_repos.sh` (local only) |
| Skills | Global: `npx skills add ...` or per-repo: `./prepare_skill_in_real_projects.sh` |
| Run agent | `cd test-repos/<name>` then `agent` or `agent -p "..."` (local only) |
| Validate | `sruja lint architecture.sruja` (local) |
| Evaluate | `./evaluate_architecture.sh express` (after architecture.sruja exists) |
| Test & observe | `./run_test_and_observe.sh` (full flow, writes observations) |
| Discovery flow | `sruja discover --context -r .`; `bash run_discovery_agent_test.sh` (see §5) |

**No CI:** Cursor CLI does not run in GitHub Actions or other CI. All testing with `agent` is on your local machine.

For editor-based testing (Cursor IDE chat, slash commands), see [TEST_ON_REAL_PROJECTS.md](TEST_ON_REAL_PROJECTS.md).

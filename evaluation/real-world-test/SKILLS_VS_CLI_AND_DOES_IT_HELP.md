# Sruja skills vs CLI – and does the skill help?

## Short answers

**Did we run on multiple repos?**  
Yes. Use **`./run_test_and_observe.sh --no-clone --multi-repo`** to run quickstart + drift on **all** repos in `test-repos/` (e.g. express, fastapi, caddy, etcd, gitea, minio, react-admin, redis, saleor, temporal). A summary table is written to `run_results/multi_repo_<timestamp>.md` and included in `run_results/test_and_observe_<timestamp>.md`.

**Is running with Sruja skills helping or not?**  
- **CLI** (quickstart, drift, lint) does **not** use skills. It is deterministic and does not read `.agents/skills/` or any skill files. So for CLI-only runs, “skills” don’t change the outcome.
- **Skills are for the AI agent** (Cursor CLI `agent`, or Cursor/VS Code chat). They tell the model how to write or edit Sruja DSL (syntax, patterns, validation). So “does the skill help?” only applies when the **agent** generates or edits `.sruja` files.

## How to see if the skill helps

1. **With skill**  
   - Ensure the skill is present in the repo (e.g. run `./prepare_skill_in_real_projects.sh` so each `test-repos/<name>/` has `.agents/skills/sruja-architecture/`).
   - Open that repo in Cursor (or `cd` there and run Cursor CLI).
   - Ask the agent to generate architecture: e.g. *“Analyze this codebase and generate architecture.sruja using Sruja DSL. Use the sruja-architecture-agent skill. Run sruja lint on the result.”*
   - Note: did it produce valid DSL? Did `sruja lint architecture.sruja` pass? Is the structure and level of detail reasonable?

2. **Quality comparison (automated)**  
   Run **`./scripts/run_comparison_test.sh express`**. This asks the LLM to generate architecture in **Mermaid** (no Sruja) in one run, and in **Sruja** (with skill) in another. Compare **which output captures system details better** (see `run_results/IS_SRUJA_HELPFUL.md` and the generated `QUALITY_COMPARISON.md`).

3. **What “helping” looks like**  
   - With skill: valid DSL, lint passes, components have descriptions and technologies, relationship labels are specific.  
   - Without skill: more chance of syntax errors, missing descriptions, or generic labels; lint may report more issues.

## What we actually ran

| What | Uses skills? | Repos |
|------|--------------|--------|
| `sruja quickstart -r <repo>` | No | All test-repos (with `--multi-repo`) |
| `sruja drift -r <repo>` | No | All test-repos (with `--multi-repo`) |
| `sruja lint` | No | express (after demo baseline) |
| `run_demo.sh --baseline` | No | express |
| `evaluate_architecture.sh express` | No | express |
| Cursor agent generating `architecture.sruja` | **Yes** (if skill is in repo or global) | Not run in automation (interactive) |

So: we **did** run on multiple repos (10) for quickstart + drift. The **skill** is evaluated by comparing architecture **quality**: LLM generates in **Mermaid** (no Sruja) vs in **Sruja** (with skill); we compare which captures system details better. Run **`./scripts/run_comparison_test.sh`** and see **`run_results/IS_SRUJA_HELPFUL.md`**.

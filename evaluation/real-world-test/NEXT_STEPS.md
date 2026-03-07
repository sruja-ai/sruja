# Next Steps: Improving Real-World Evaluation

Ideas to improve code quality, usability, and maintainability (in rough priority).

## Done recently

- **Shared `lib.sh`** — `find_sruja` and `has_llm_key` in one place; scripts source it.
- **`.env.example`** — Clear quick start and verify section.
- **Robust stats** — `evaluate_architecture.sh` normalizes counts so reports are always numeric.
- **MANIFEST from REPOS** — `setup_repos.sh` generates `test-repos/MANIFEST.md` from the `REPOS` array (single source of truth).
- **README vs scripts** — README and EVALUATION_GUIDE aligned with actual default repos (express, fastapi, next.js, prometheus, django).
- **`.gitignore`** — `evaluation/real-world-test/results/` so generated reports aren’t committed.
- **`--help`** — `run_demo.sh` and `evaluate_architecture.sh` support `-h` / `--help`.

---

## High value (recommended next)

1. **Environment check script**  
   Add `check_env.sh` (or a `make eval-check` target) that:
   - Verifies `sruja` is available (built or in PATH).
   - Optionally checks for `.env` and LLM-related vars when `--llm` is desired.
   - Verifies `test-repos/` exists and lists repos with/without `architecture.sruja`.  
   Improves onboarding and CI.

2. **Evaluate all repos**  
   Add `evaluate_all.sh` that:
   - Loops over `test-repos/*` (or a list from MANIFEST).
   - Runs `evaluate_architecture.sh` for each repo that has `architecture.sruja`.
   - Optionally writes a summary table (e.g. one markdown file with all scores).  
   Makes batch evaluation and reporting easier.

3. **CI job**  
   Add a minimal CI step (e.g. GitHub Actions) that:
   - Runs `./run_demo.sh` (no `--llm`) to catch script and CLI regressions.
   - Optionally runs `./setup_repos.sh` and `./evaluate_architecture.sh express` if `sruja` is built.  
   Prevents breakage from refactors.

---

## Medium value

4. **Machine-readable output**  
   Add an option (e.g. `--json`) to `evaluate_architecture.sh` that prints stats + validation result as JSON for CI or tooling.

5. **Configurable repo list**  
   Allow overriding the repo list via env (e.g. `SRUJA_TEST_REPOS="express fastapi"`) or a small config file so CI or users can run a subset without editing `setup_repos.sh`.

6. **Single-repo setup**  
   In `setup_repos.sh`, support one argument: `./setup_repos.sh express` to clone only that repo (and update MANIFEST for that entry only if desired).

---

## Lower priority

7. **QUICKSTART vs README**  
   Keep QUICKSTART as the “run in 2 minutes” path and README as the full narrative; add one-line cross-links at the top of each so users don’t get lost.

8. **Results summary script**  
   A small script that scans `results/*.md`, parses “Score: X/10” (or similar) and outputs a single summary table (e.g. for pasting into issues or docs).

9. **Document “reporting results”**  
   Short section in README or CONTRIBUTING on how to share evaluation results (e.g. gist template, what to redact, where to post).

---

## Not recommended (or defer)

- **Heavy automation of “generate architecture”** — Generation is intentionally AI/editor-driven; avoid replacing that with a single CLI command that might not reflect real usage.
- **Changing default repos too often** — Current set is a good balance of size and diversity; document how to add/override rather than churning the list.

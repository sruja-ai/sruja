# Evaluation

How we test and compare Sruja: real-world runs, Mermaid vs Sruja comparison, and improvement plans.

## What we test

- **Real-world usefulness** – Can the Sruja agent skill produce useful architecture from real codebases? See [real-world-test/](real-world-test/).
- **Quality comparison** – Mermaid (no Sruja) vs Sruja (with skill): which output captures system details better? See [Comparison runs](#comparison-runs) below.

## Comparison runs (Mermaid vs Sruja)

Comparison runs are stored under `evaluation/results/comparison_<project>_<timestamp>/`. Each run contains:

- **baseline.mmd** (or .md) – Architecture generated in Mermaid, no Sruja skill.
- **sruja.sruja** – Architecture generated in Sruja DSL with the Sruja skill.
- **README.md** – Metrics for that run (lines, systems, relationships, **Sruja lint: pass/fail**).
- **QUALITY_COMPARISON.md** – Rubric for judging which captures the system better.
- **HELPFUL_SUMMARY.md** – Written by `scripts/summarize_comparison.sh`.

### Recent comparison runs

| Directory | Project | Notes |
|-----------|---------|------|
| [comparison_express_20260310_0915/](results/comparison_express_20260310_0915/) | Express | Includes [ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md](results/comparison_express_20260310_0915/ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md) |
| [comparison_express_20260310_0759/](results/comparison_express_20260310_0759/) | Express | |
| [comparison_express_20260310_0756/](results/comparison_express_20260310_0756/) | Express | |
| [comparison_express_20260310_0726/](results/comparison_express_20260310_0726/) | Express | |
| [comparison_test_20260309_100147/](results/comparison_test_20260309_100147/) | Legacy (baseline vs enhanced Sruja) | |

To run a new comparison:

```bash
./scripts/run_comparison_test.sh <project_name> <repo_url>
./scripts/summarize_comparison.sh evaluation/results/comparison_<project>_<timestamp>
```

**Example – FastAPI (validate skill on another stack):**

```bash
./scripts/run_comparison_test.sh fastapi https://github.com/tiangolo/fastapi
# Then summarize the new dir, e.g.:
./scripts/summarize_comparison.sh evaluation/results/comparison_fastapi_<timestamp>
```

Runs are **local only** (no CI yet). To keep regression visible, run comparisons periodically and record lint pass/fail in the generated README and `LINT_STATUS.txt`.

### Same content, two formats (Sruja → Mermaid)

To get a Mermaid diagram from a `.sruja` file (e.g. to compare with baseline Mermaid or for presentation):

```bash
sruja export mermaid path/to/sruja.sruja
```

Use `--view-level 2` or `--view-level 3` for container/component focus; see `sruja export --help`.

## Key artifacts and plans

- **Is Sruja helpful?** – [real-world-test/run_results/IS_SRUJA_HELPFUL.md](real-world-test/run_results/IS_SRUJA_HELPFUL.md)
- **Architecture analysis improvements** – [results/comparison_express_20260310_0915/ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md](results/comparison_express_20260310_0915/ARCHITECTURE_ANALYSIS_IMPROVEMENTS.md)
- **Next steps (improvement plan)** – [docs/NEXT_STEPS_IMPROVEMENTS.md](../docs/NEXT_STEPS_IMPROVEMENTS.md)

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/run_comparison_test.sh` | Run Mermaid vs Sruja comparison for a repo; writes results to `evaluation/results/comparison_<project>_<timestamp>/` and records Sruja lint pass/fail. |
| `scripts/summarize_comparison.sh` | Summarize a comparison dir: metrics and HELPFUL_SUMMARY.md. |

## Development

For where skill source and comparison work live in the repo, see [docs/DEVELOPMENT.md](../docs/DEVELOPMENT.md#skills-and-evaluation).

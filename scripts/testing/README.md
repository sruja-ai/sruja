# Testing tools (maintainers)

These scripts are **maintainer utilities** for running Sruja against real-world repositories (mostly large OSS) to catch regressions and measure performance/accuracy. They are **not required** for end users.

All outputs go under `evaluation/local-artifacts/` (git-ignored).

## Quick start

Build the CLI once:

```bash
make build
```

Run a deterministic smoke test on a curated set of large repos (quickstart + scan, writes metrics):

```bash
./scripts/testing/smoke_complex_repos.sh
```

## Scripts

- `smoke_complex_repos.sh`
  - Deterministic, CI-friendly-ish smoke run for large repos.
  - Runs: `sruja quickstart`, `sruja scan` and saves `metrics.json`.

- `agent_skill_benchmark.sh`
  - Runs the CLI analysis suite used for skill/agent evaluation harnesses and writes structured artifacts.

- `service_detection_sanity.sh`
  - Quick local check for service detection changes against already-cloned repos in `/tmp`.

## Output locations

- `evaluation/local-artifacts/testing/` for timestamped runs and logs.
- Repos are cloned/updated under `/tmp/sruja_test_*` (local machine only).


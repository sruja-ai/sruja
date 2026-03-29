# Sruja Dogfooding Playbook (Internal)

This playbook defines our loop for using Sruja on Sruja. We aim to measure whether Sruja is becoming an essential tool in our daily engineering workflow or just a "nice-to-have" demo.

---

## 1. The Core Loop

To prevent architectural decay, we follow this strictly for **all** PRs:

1.  **Reviewed Truth**: Files in `docs/architecture/*.sruja` are the authoritative source. If you change how a system is structured (adding a container, changing a dependency), **you must update these files**.
2.  **Generated Reality**: `repo.sruja` is the automated "reality check". It represents what Sruja's structural analysis currently sees in the codebase.
    *   `repo.sruja` is generated via `sruja quickstart -r . --generate-baseline`.
3.  **Mandatory PR Check**: Every meaningful code change must be validated locally using at least one:
    *   `sruja doctor -r .` (Quick health check)
    *   `sruja daily -r .` (Review what changed and verify evidence)
    *   `sruja drift -r . -a repo.sruja` (Check if code drifts from the baseline)

> [!IMPORTANT]
> If `sruja drift` flags an issue, you have two choices:
> - **Fix the code**: If the new code violates the intended architecture.
> - **Update the architecture**: If the change was intentional, update `docs/architecture/*.sruja` and regenerate the baseline.

---

## 2. Success Metrics (2-3 Week Trial)

We judge Sruja's value by these three outcomes:

### A. Onboarding Efficiency
*   **Question**: Can a new contributor understand the CLI, WASM, and extension by looking at `docs/architecture/*.sruja` *first* before reading the code?
*   **Target**: 80% of structural questions should be answerable via Sruja artifacts.

### B. Drift Detection
*   **Question**: Does Sruja catch architectural drift or missing documentation updates *before* the PR is merged?
*   **Target**: Sruja should flag 100% of new unmapped components or "illegal" dependencies (e.g., CLI calling WASM directly).

### C. AI Context Efficacy
*   **Question**: Does giving the AI Sruja context (via the `sruja-architecture` skill) produce more accurate and grounded changes compared to standard context?
*   **Target**: Reduction in AI hallucinations regarding repo structure.

---

## 3. Log: Wins, Misses, and Noise

Use this section to log feedback during the 3-week trial.

| Date | Type (Win/Miss/Noise) | Description | Outcome/Action |
| :--- | :--- | :--- | :--- |
| | | | |
| | | | |

---

## 4. Current CI Enforcement

The following GitHub Actions are active:
- `sruja-blueprint-pr.yml`: Annotates PRs with architectural diagrams.
- `sruja-drift.yml`: Flags drift between the codebase and `repo.sruja`.
- `unified-ci.yml`: Runs `sruja lint` on all `.sruja` files.

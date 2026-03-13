# Architecture Discovery Evaluation Methodology

How we measure and improve the accuracy of Sruja-generated architecture (from the sruja-architecture-agent skill or `sruja generate`). Use this to score runs, compare golden vs generated files, and tune the skill.

## Goals

- **Quantify** how well generated `.sruja` matches a reference (golden) or the codebase.
- **Compare** runs over time (e.g. after changing the skill’s playbook or prompts).
- **Identify** systematic gaps (e.g. missing data stores, wrong C4 level) to improve heuristics.

## Metrics (structural)

We use **structure-only** metrics that don’t require semantic matching (IDs can differ between golden and generated).

| Metric | Definition | How computed |
|--------|------------|--------------|
| **Systems** | Count of `= system` | `grep -c "= system" file.sruja` |
| **Containers** | Count of `= container` | `grep -c "= container" file.sruja` |
| **Components** | Count of `= component` | `grep -c "= component" file.sruja` |
| **Datastores** | Count of `= database` or `= datastore` | `grep -cE "= database\|= datastore" file.sruja` |
| **Persons** | Count of `= person` | `grep -c "= person" file.sruja` |
| **Relationships** | Count of `->` (edges) | `grep -c "->" file.sruja` |
| **Lint** | Valid DSL | `sruja lint file.sruja` (pass/fail) |

**Derived (when comparing golden vs generated):**

- **Delta** = generated − golden for each count (positive = more elements than golden, negative = fewer).
- **Component recall (approximate):** If we had a mapping from golden IDs to generated IDs, recall = (matched components) / (golden components). Without ID mapping we use **count proximity**: e.g. ratio generated/golden for containers; target near 1.0.
- **Relationship ratio:** (generated relationships) / (golden relationships); target near 1.0.

## Golden (reference) files

- **Location:** `test-repos/<repo>/architecture.sruja` — hand-authored or carefully reviewed.
- **Use:** As the reference when comparing a new agent run. Copy the agent output to e.g. `run_results/generated_<repo>_<timestamp>.sruja` and run the comparison script against the golden file.

## Scripts

| Script | Purpose |
|--------|---------|
| **evaluate_architecture.sh** | Single file: stats, lint, manual checklist. `./evaluate_architecture.sh express` |
| **compare_architecture.sh** | Two files: structural diff (counts, lint). `./compare_architecture.sh test-repos/express/architecture.sruja run_results/generated_express.sruja` |
| **run_architecture_comparison_report.sh** | Batch: for each test-repo with golden `architecture.sruja`, report stats and (if present) comparison vs `run_results/generated_<repo>.sruja`. Output: `run_results/ARCHITECTURE_COMPARISON_REPORT_<timestamp>.md` |
| **run_diff_refine_prompt.sh** | Build a diff-and-refine prompt for the AI: repo context + drift + current elements. Paste output into chat with sruja-architecture-agent to get proposed changes only. `./run_diff_refine_prompt.sh [repo_path] [architecture.sruja]`; use `-` as last arg for stdout. |

## Running a comparison

```bash
# From evaluation/real-world-test/
./compare_architecture.sh test-repos/express/architecture.sruja run_results/generated_express_20260312.sruja
```

Output: side-by-side counts, deltas, lint status for both, and a short summary (e.g. “Generated has +2 containers, -1 relationship vs golden”).

## Manual evaluation (quality)

Structure alone doesn’t capture correctness. Use the **manual checklist** printed by `evaluate_architecture.sh`:

- **Completeness** – Entry points, core modules, data flows, externals.
- **Accuracy** – Names match code, no fabricated components, correct technologies.
- **Clarity** – Understandable, well-labeled relationships.
- **Usefulness** – Would help onboarding and design decisions.

Score each dimension 1–10 and average. Track scores over time to see if skill changes improve quality.

## Improving the skill from results

1. **Run** agent on a test repo; save output as `generated_<repo>.sruja`.
2. **Compare** with `compare_architecture.sh` to golden.
3. **Review** manual checklist and note recurring gaps (e.g. “always misses Redis”, “wrong level for Router”).
4. **Update** skill or REFERENCE: add heuristics (e.g. deployable detection, entry-point table), tighten playbook, or add mode-specific prompts.
5. **Re-run** and compare again; iterate.

See [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](../../docs/ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md) for research-backed practices (phased playbook, discovery modes, dependency context).

**Capturing rich architecture for evaluation:** To persist a full snapshot per repo (scan graph, discover context, context export, intent/ADR report, optional DSL), use the [rich capture pipeline](../../docs/CAPTURING_RICH_ARCHITECTURE_FROM_REPOS.md) and run `./run_rich_architecture_capture.sh <repo_name>` (or `--list repo1 repo2`). Output: `run_results/rich_capture_<repo>_<timestamp>/` with `graph.json`, `discover_context.txt`, `context_export.json`, `intent_report.json`, and optional `architecture.sruja` / `prompt.txt`. Use the bundle to compare runs, tune the skill, and feed the agent with all available signals.

# Architecture Intelligence Demo

This demo shows the full **Architecture Intelligence** flow on a small, self-contained microservices example: intent (rulebook) → scan → intent drift → analyze → why (deterministic).

## What’s in this folder

- **architecture.sruja** – The “rulebook”: Frontend must not talk to Database; requests go via ApiGateway → UserService → Database.
- **frontend.py**, **api_gateway.py**, **user_service.py**, **database.py** – Minimal Python services with imports that Sruja’s scanner parses to build a dependency graph.
- **run_demo.sh** – Runs the five steps below.

## Run the demo

From the **repo root** (after `make build`):

```bash
make demo-intel
```

Or from this directory:

```bash
cd demo
./run_demo.sh
```

The script finds the `sruja` CLI from `target/release`, `target/debug`, or your `PATH`.

## The five steps

1. **The rulebook (intent)** – Prints `architecture.sruja` so you see the intended design.
2. **The reality (code scan)** – `sruja scan` builds a dependency graph from the Python files.
3. **Intent drift (docs vs. code)** – `sruja intent check -r . -i . -f markdown` produces `intent_report.md` and highlights the key unexpected dependency (e.g. `frontend_py` → `database`).
4. **Runtime intelligence** – `sruja analyze --view cto -t traces.json` (skipped if `traces.json` is missing).
5. **Deterministic explainability** – `sruja why` answers "Why does the Frontend access the database?" using graph evidence. For richer natural-language interpretation, use the Sruja skill in your editor (Cursor, Copilot, etc.); the editor's AI runs quickstart/drift/why and interprets the output.

## Commit-to-commit drift demo

For a **repo + two commits** flow (Architecture Intelligence at commit A, then drift from A to commit B), see:

- **[COMMIT_DRIFT_DEMO.md](COMMIT_DRIFT_DEMO.md)** – Scenario, insights, and usage.
- **Script:** `./run_commit_drift_demo.sh [REPO] [BASELINE_REF] [HEAD_REF]` (defaults: `.`, `HEAD~1`, `HEAD`).

Example: `./run_commit_drift_demo.sh . main HEAD` — quickstart at `main`, then new violations vs current HEAD.

## See also

- [Architecture Intelligence](../docs/ARCHITECTURE_INTELLIGENCE.md) – Overview and entry points; use the Sruja skill in your editor for AI interpretation.
- [RUN_GUIDE](../docs/RUN_GUIDE.md) – How to run Sruja (CLI, demos, app).

# Testing Sruja on customer-facing applications

This describes how to run Sruja (quickstart, drift, discover) on repos that are **close to customer-facing applications**: e-commerce, SaaS, admin UIs, collaboration, scheduling — not just frameworks or libraries.

## Repos used

The script `run_customer_facing_apps_test.sh` uses a curated list that matches `setup_repos.sh`:

| Repo | Source | Description |
|------|--------|-------------|
| **gitea** | `--apps` / `--complex` | Self-hosted Git service: web UI, API, SSH/HTTP |
| **saleor** | `--apps` / `--complex` | Headless e-commerce: GraphQL API, dashboard, checkout (Python/Django) |
| **documenso** | `--apps` | Open-source document signing (TypeScript/Next.js) |
| **cal.com** | `--apps` | Scheduling and meetings (TypeScript/Next.js) |
| **react-admin** | `--complex` | Admin/dashboard framework: CRUD, auth, data providers (TypeScript/React) |

## One-time setup (clone repos)

From `evaluation/real-world-test`:

```bash
# Apps only (gitea, saleor, documenso, cal.com)
./setup_repos.sh --apps

# Optional: add react-admin and others
./setup_repos.sh --complex
```

## Run the test

```bash
# From evaluation/real-world-test

# Clone then run (if you haven’t cloned yet)
./run_customer_facing_apps_test.sh --setup

# Or run on existing repos only
./run_customer_facing_apps_test.sh
```

**Output:**

- **Report:** `run_results/CUSTOMER_FACING_APPS_TEST_<timestamp>.md` — table (Quickstart ✓/✗, Drift ✓/✗, Discover ✓/✗, Notes).
- **Logs:** `run_results/customer_facing_<timestamp>/` — per-repo `*_quickstart.txt`, `*_drift.txt`, `*_discover.txt`.

## Run on a custom list

```bash
./run_customer_facing_apps_test.sh --list saleor documenso cal.com
```

Only repos that exist under `test-repos/` are run.

## What gets run per repo

| Command | Purpose |
|---------|--------|
| `sruja quickstart -r <repo>` | Scan, health score, findings (orphans, cycles, layer violations, god modules). Timeout 180s. |
| `sruja drift -r <repo>` | Drift without a baseline (shows structural findings). Timeout 90s. |
| `sruja discover --context -r <repo>` | Gather context (entry points, deployables, deps) for architecture discovery. Timeout 45s. |

## Optional: agent-generated architecture

To generate `architecture.sruja` with the Cursor agent on these same repos:

```bash
./prepare_skill_in_real_projects.sh
./run_agent_architecture_all_repos.sh --list gitea saleor documenso cal.com react-admin
```

Then run `sruja lint architecture.sruja` in each repo. See [LOCAL_CURSOR_CLI_TESTING.md](LOCAL_CURSOR_CLI_TESTING.md).

## Production-grade / enterprise apps

For ERP, CRM, chat, error-tracking (e.g. Mattermost, Sentry, Ever Gauzy), use:

```bash
./setup_repos.sh --production
./run_production_apps_report.sh
```

Report: `run_results/PRODUCTION_APPS_REPORT_<timestamp>.md`.

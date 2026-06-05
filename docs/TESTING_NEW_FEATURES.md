# Testing the New Features (check, review, federation)

How to test the **check**, **review**, **publish**, and **compose** commands locally and in CI.

---

## 1. `sruja drift --ci` (CI drift check, always exit 0)

### Automated tests

Run the e2e tests (uses a temp repo, runs `sruja drift --ci` in JSON, text, and github-actions formats):

```bash
cargo test -p sruja-cli --test check_e2e
```

Or with output:

```bash
cargo test -p sruja-cli --test check_e2e -- --nocapture
```

### Manual testing

From the repo root (or any path with a baseline and optional code):

```bash
# Build the CLI first if needed
cargo build --release -p sruja-cli

# Default: github-actions format (for CI)
sruja drift --ci -r .

# Text summary
sruja drift --ci -r . -f text

# JSON (for tooling)
sruja drift --ci -r . -f json
```

**What to verify:** Exit code is always 0. Output shows truth status, violation count, and (for `-f github-actions`) `::notice`/`::warning` annotations.

### CI: Sruja Check workflow

The workflow [`.github/workflows/sruja-check.yml`](../.github/workflows/sruja-check.yml) runs on **pull_request** and executes:

```bash
cargo install --path crates/sruja-cli
sruja drift --ci -r . --format github-actions
```

To test it: open a PR against `main`; the "Sruja Check" job will run.

---

## 2. `sruja review` (evidence + drift + suggestions)

No automated e2e tests yet. Test manually.

### Manual testing

```bash
# From repo root (uses . or -r path)
sruja review -r .

# JSON output
sruja review -r . -f json
```

**What to verify:** Output includes `truth_status`, `violations_count`, categorized lists (`new_components`, `missing_components`, `drifted_dependencies`), `open_questions`, and `suggestions`. Run in a repo that has a baseline (e.g. a `.sruja` file) and some code so drift can be detected.

---

## 3. Federation: `sruja publish` and `sruja compose`

No dedicated e2e tests. Use manual runs and schema checks (see [FEDERATION.md](FEDERATION.md)).

### Manual: publish (repo → bundle)

```bash
# Publish current repo to repo.bundle.json
sruja publish -r . -o repo.bundle.json

# Publish a subdirectory
sruja publish -r ./services/api -o /tmp/api.repo.bundle.json
```

**Verify:** `repo.bundle.json` exists and contains `schema_version`, `repo_id`, `context`, `truth_status`, and (if a baseline exists) `baseline_path` / `baseline_dsl`.

### Manual: compose (bundles → system index)

```bash
# One bundle
sruja compose -i repo.bundle.json -o system.index.json

# Directory of bundles
sruja compose -i ./bundles -o system.index.json
```

**Verify:** `system.index.json` contains `schema_version`, `repos`, `nodes`, `edges`, and `conflicts` (may be empty).

### Contract checks (optional)

From [FEDERATION.md](FEDERATION.md): validate that:

- **.sruja/context.json** — has `schema_version`, `updated_at`, `truth_status`
- **repo.bundle.json** — has `schema_version`, `repo_id`, `context`, `truth_status`
- **system.index.json** — has `schema_version`, `repos`, `nodes`, `edges`

You can run `sruja sync -r .` first to ensure `.sruja/context.json` exists, then `sruja publish` and inspect the bundle with `jq` or a JSON validator.

---

## 4. Run all relevant tests (check + workspace)

```bash
# Check e2e only
cargo test -p sruja-cli --test check_e2e

# Full workspace (includes check_e2e when running sruja-cli tests)
cargo test --workspace
```

---

## Quick reference

| Feature   | Automated tests              | Manual test                          | CI |
|----------|------------------------------|--------------------------------------|----|
| **check**  | `cargo test -p sruja-cli --test check_e2e` | `sruja drift --ci -r . -f text` / `json` / `github-actions` | Sruja Check workflow on PR |
| **review** | —                            | `sruja review -r .` / `-f json`      | — |
| **publish**| —                            | `sruja publish -r . -o repo.bundle.json` | — |
| **compose**| —                            | `sruja compose -i repo.bundle.json -o system.index.json` | — |

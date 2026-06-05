# Plan: Commit, Push, and Make Git Workflows Pass

## Pre-push checklist (already done)

- [x] **Format:** `cargo fmt --all` (CI runs `cargo fmt --all -- --check`)
- [x] **Clippy:** `cargo clippy --workspace -- -D warnings` (fixed collapsible_else_if and manual_strip in `crates/sruja-export/src/mermaid/views.rs`)
- [x] **Tests:** `cargo test --workspace` (all pass)
- [x] **Version consistency:** `./scripts/verify-version-consistency.sh .` (0.15.0 OK)

## What CI runs (unified-ci.yml)

| Job | Trigger | What it does |
|-----|---------|----------------|
| **changes** | Always | Path filter: `rust` (crates/**), `sruja` (*.sruja), `skills` (skills/**) |
| **version-consistency** | Always | `./scripts/verify-version-consistency.sh .` |
| **rust** | If `rust` changed | `cargo build --release`, `cargo test --workspace`, `cargo fmt --check`, `cargo clippy -D warnings` |
| **sruja-files** | If `sruja` changed | Validate all `**/*.sruja` with sruja-validate action |
| **skill-files** | If `skills` changed | Skill lint/validate on `skills/**/*.md` (links, xrefs, test-code, format) |

Your changes touch **crates/** and **skills/** so **rust** and **skill-files** will run. No `.sruja` or version files in your export-only edits, so **version-consistency** runs but should still pass.

## Suggested commit scope

**Include (export + docs + fixes):**

- `crates/sruja-export/` – exporter, options, mermaid views, tests, EXPORT_COVERAGE.md
- `crates/sruja-language/src/parser/overview_views.rs` – only if you have intentional parser changes (e.g. ViewDef title); otherwise consider reverting
- `skills/sruja-architecture/REFERENCE.md` – export coverage section
- `Cargo.lock` – if dependency or feature changes

**Exclude from this commit (separate PRs or local):**

- `.copilot-instructions.md`, `.cursorrules` – editor/tooling
- `crates/sruja-cli/src/commands/dsl.rs`, `crates/sruja-cli/src/main.rs` – unless they are required for export
- `extension/*` – VS Code extension (different workflow: publish-extension)
- `ROLLOUT-IMPLEMENTATION-PLAN.md`, `docs/TESTING_NEW_FEATURES.md`, `docs/adr/`, `rs-repo.sruja` – add in separate commits/PRs if desired
- Other `skills/sruja-architecture/` edits (AGENTS.md, PROMPTS.md, rules, scripts) – include only if they are part of the same feature

## Commands

```bash
# 1. From repo root
cd /Users/dilipkola/Workspace/sruja

# 2. Stage only export-related and doc changes (adjust paths as needed)
git add crates/sruja-export/
git add crates/sruja-language/src/parser/overview_views.rs   # only if you want parser changes
git add Cargo.lock
git add skills/sruja-architecture/REFERENCE.md

# 3. Commit
git commit -m "feat(export): improve DSL/Markdown/Mermaid export and doc structure

- Markdown: add deployments section, element metadata, scenario/flow description,
  requirement ID/tags, causal loop variables; reorder sections (arc42-style);
  add document title and Stakeholders heading; EXPORT_COVERAGE.md
- Mermaid: export_from_resolved_view for view-driven diagrams
- Tests: deployments, metadata, causal loop (fixed DSL), export_from_resolved_view,
  test_export_captures_detail_fields
- Fix clippy: collapsible_else_if, manual_strip in mermaid/views.rs
- Format: cargo fmt"

# 4. Push (branch name depends on your workflow)
git push origin HEAD
# Or: git push origin your-branch-name
```

If you use a **single branch** (e.g. `main`):

```bash
git add crates/sruja-export/ Cargo.lock skills/sruja-architecture/REFERENCE.md
# Add overview_views.rs only if needed
git status   # review
git commit -m "feat(export): improve DSL/Markdown/Mermaid export and doc structure"
git push origin main
```

## If CI fails

- **Rust job:** Re-run locally: `cargo build --release && cargo test --workspace && cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings`.
- **Skill-files job:** From repo root, run the skill action locally if possible, or fix any reported markdown/link/xref/format issues under `skills/`.
- **Version consistency:** Ensure `Cargo.toml`, `extension/package.json`, and `.release-please-manifest.json` versions match (script output shows expected).

## Optional: sruja-check workflow

`sruja-check.yml` runs on PR: `cargo install --path crates/sruja-cli` then `sruja drift --ci -r . --format github-actions`. It only needs the CLI to build and `sruja drift --ci` to pass; no export changes required for that.

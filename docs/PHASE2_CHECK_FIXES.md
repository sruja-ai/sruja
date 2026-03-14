# Phase 2 Check Command – Fixes and How to Reapply

This document describes the fixes applied to the `sruja check` command and how to fix the workspace doctest failure if it reappears.

## 1. Check command fixes (already applied)

### Text output (no inverted “No violations found”)

- **Issue:** “No violations found” could be missing or logic inverted.
- **Fix:** In `crates/sruja-cli/src/commands/check.rs`, the default (text) branch now has:
  - **When there are no violations:** print `"No violations found."`
  - **When there are violations:** print `"Violations found:"` and then list them.

So “No violations found” is only printed when `output.violations.is_empty()`.

### GitHub Actions output (valid annotations)

- **Issue:** `::notice`/`::warning` messages could break if they contained `%` or newlines.
- **Fix:** In the `"github-actions"` branch:
  - Always emit one `::notice` with a summary (truth status + violation count).
  - When there is drift, emit `::warning` with an escaped message.
  - Escape the message for GitHub: `%` → `%25`, newline → `%0A`, `\r` → `%0D`.

### CLI wiring

- **main.rs:** `Commands::Check { path, format }` is in the enum and dispatched as `commands::check(&path, &format).await`.
- **mod.rs:** `mod check` and `pub use check::check` are declared.

No change needed if this wiring is present.

### E2E tests

- **File:** `crates/sruja-cli/tests/check_e2e.rs`
- **Tests:** `check_exits_zero_json`, `check_exits_zero_text`, `check_exits_zero_github_actions`
- **Run:** `cargo test -p sruja-cli --test check_e2e`

---

## 2. If workspace doctest fails (sruja-engine, “can't find crate sruja_language”)

**Symptom:** `cargo test --workspace` or `cargo test --workspace --doc` fails with:

```text
error[E0463]: can't find crate for `sruja_language`
  --> crates/sruja-engine/src/rules/cycle.rs:10:5
```

**Option A – Clean and rebuild (try first):**

```bash
cargo clean
cargo test --workspace
```

Sometimes a stale or partial build causes the doctest driver to miss the `sruja_language` dependency; a full rebuild fixes it.

**Option B – Skip doctests for that crate in CI:**

In `.github/workflows/*.yml`, if you run doc tests separately, you can skip doc tests for `sruja-engine`:

```yaml
- run: cargo test --workspace --no-doc
```

Or run doc tests only for crates that have no transitive doc issues:

```yaml
- run: cargo test -p sruja-diagnostics --doc
- run: cargo test -p sruja-engine --doc
# etc.
```

**Option C – Mark a specific doctest as `ignore`:**

If a single doc block in `sruja-engine` triggers the error, find it (e.g. in `src/lib.rs` or under `src/utils/`, `src/validator/`) and change the fence to:

```text
/// ```ignore
/// ... code that pulls in sruja_language ...
/// ```
```

so that rustdoc does not compile that block. Use this only if the example cannot be run in the doctest environment.

---

## 3. Verifying everything

```bash
# All unit + integration tests (no doc tests)
cargo test --workspace --no-doc

# All tests including doc tests
cargo test --workspace

# Check command only
cargo test -p sruja-cli --test check_e2e

# Invoke check manually
cargo run -p sruja-cli -- check -r . -f json
cargo run -p sruja-cli -- check -r . -f text
cargo run -p sruja-cli -- check -r . -f github-actions
```

All of the above should succeed after the Phase 2 check fixes are applied.

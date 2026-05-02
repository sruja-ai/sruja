# Testing sruja-wasm

The `sruja-wasm` crate targets `wasm32-unknown-unknown` and is **not** exercised by normal `cargo test`. Standard repo coverage commands therefore **exclude** it from llvm-cov reports so the headline percentage stays meaningful; you verify WASM separately (below).

---

## Option 1: Skip from coverage (current)

- **What:** Exclude `sruja-wasm` from the host coverage run so the percentage reflects `cargo test` on native targets. The repo does this in three places that should stay aligned:
  - **`just test-coverage`** — runs `cargo llvm-cov --workspace --exclude sruja-wasm` (see `justfile`).
  - **`scripts/coverage.sh`** — runs `cargo llvm-cov` with `--exclude-from-report sruja-wasm` (and can pass extra args).
  - **CI** (`.github/workflows/codecov.yml`) — `cargo llvm-cov ... --exclude-from-report sruja-wasm` when generating `lcov.info`.
- **Why:** The WASM crate is built for `wasm32-unknown-unknown` and is not exercised by normal `cargo test`. Excluding it avoids a misleading **0%** line that drags down the workspace total. The bindings are still tested via **Option 2** (`wasm-pack test`) and the core logic is covered in `sruja-language`, `sruja-engine`, and `sruja-export`.
- **When:** Use this for “how good is our Rust test coverage on the host run”; use **Option 2** to validate the WASM API itself.

---

## Option 2: Test the WASM API (recommended, implemented)

**wasm-bindgen-test** is set up so the exported WASM functions are exercised in Node (no browser).

- **What:** `#[wasm_bindgen_test]` tests in `crates/sruja-wasm/src/lib.rs` call `sruja_dsl_to_model`, `sruja_dsl_to_mermaid`, `sruja_get_diagnostics`, and assert on results.
- **How to run:** From the repo root or from `crates/sruja-wasm`:
  ```bash
  cd crates/sruja-wasm && wasm-pack test --node
  ```
  Requires `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`) and `wasm-pack`.
- **CI:** Add a job that runs `wasm-pack test --node` (e.g. after installing wasm-pack and the wasm target).
- **Pros:** Fast, no Playwright, catches regressions in the WASM build and JS boundary.
- **Cons:** Runs in Node, not in the exact book/extension UI.

---

## Option 3: Playwright (one E2E test, implemented)

One **Playwright** E2E test verifies “Show diagram” in the book.

- **What:** `e2e/book-show-diagram.spec.ts` opens the “How Sruja works” page, clicks “Show diagram” on a ```sruja block, and asserts the diagram renders (WASM + Mermaid SVG).
- **Run:** Start the book with `just book-serve` (or `book/serve.sh`) so WASM is copied into the output, then `npm run e2e`. See `e2e/README.md`.
- **When:** Confirms the full path (book → WASM → Mermaid) works; complement to wasm-bindgen-test for the WASM API.

---

## Recommendation

1. **Keep excluding sruja-wasm from host llvm-cov** (`just test-coverage`, `scripts/coverage.sh`, Codecov) so the percentage reflects native `cargo test` runs.
2. **Keep running wasm-bindgen-test** (`just test-wasm` / `wasm-pack test --node`) for the WASM API; add or extend a CI job if this repo does not already run it on every PR.
3. **Add Playwright (or similar) only if** you need explicit E2E for the book or extension; otherwise skip it.

Summary: **skip from coverage** is correct for reporting; **test with wasm-bindgen-test** is the right way to test the WASM crate; **Playwright** is optional for full UI E2E.

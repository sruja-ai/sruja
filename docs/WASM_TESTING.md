# Testing sruja-wasm

The `sruja-wasm` crate is excluded from `cargo llvm-cov` coverage because it is built for `wasm32-unknown-unknown` and is not exercised by normal `cargo test`. You have two complementary options.

---

## Option 1: Skip from coverage (current)

- **What:** Exclude `sruja-wasm` from the coverage report (e.g. `scripts/coverage.sh` uses `--exclude-from-report sruja-wasm`).
- **Why:** Keeps the main coverage number meaningful for code that is actually run by `cargo test`. WASM is a thin FFI layer over the same logic already tested in `sruja-language`, `sruja-engine`, and `sruja-export`.
- **When:** Use this if you are fine with “WASM is covered indirectly by the underlying crates” and don’t need dedicated WASM tests.

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

1. **Keep excluding sruja-wasm from coverage** so the main coverage percentage reflects code run by `cargo test`.
2. **Add wasm-bindgen-test** and run `wasm-pack test --node` in CI so the WASM API is tested without a browser.
3. **Add Playwright (or similar) only if** you need explicit E2E for the book or extension; otherwise skip it.

Summary: **skip from coverage** is correct for reporting; **test with wasm-bindgen-test** is the right way to test the WASM crate; **Playwright** is optional for full UI E2E.

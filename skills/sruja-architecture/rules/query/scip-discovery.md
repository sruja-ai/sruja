# scip-discovery

## Why It Matters

Standard architecture discovery (Tree-sitter) is fast but limited to syntax. For deep semantic analysis (cross-file symbol resolution), Sruja uses SCIP (Source Code Intelligence Protocol). This ensures that "Blast Radius" and "Impact Analysis" are compiler-accurate.

## When to Apply

- User asks for "Deep impact analysis" or "Full refactor context".
- Performing a complex refactor across multiple modules or services.
- You need to verify if a function call in one file actually invokes a component in another file.
- The `sruja-scan` metadata `scip.enriched` is `false` or missing, but you need higher confidence.

## Correct Approach

1. **Check for existing SCIP index.** Look for `index.scip` in the repository root.
   - If it exists, Sruja will automatically use it during scan-backed commands like `sruja scan`, `sruja drift`, `sruja sync`, and `sruja impact`.

2. **Generate SCIP index if missing or stale.** 
   - **Identify Environment:**
     - If `package.json` exists: You are in a Node/TypeScript environment.
     - If `Cargo.toml` exists: You are in a Rust environment.
   - **Run Indexer:**
     - **TypeScript:** Execute `npx @sourcegraph/scip-typescript --infer`.
     - **Rust:** Execute `cargo scip` (requires `cargo-scip` to be installed).
     - *Fallback:* If the indexer fails due to compilation errors, notify the user and proceed with standard Tree-sitter discovery (lower confidence).

3. **Verify Enrichment.** After generating `index.scip`, run `sruja scan . --output sruja.graph.json`. 
   - Check the `metadata` for `"scip.enriched": "true"`.

4. **Perform Deep Query.** Use the enriched graph with:
   - `sruja impact <target> -r . --depth 3`
   - `sruja human explain <id> --file repo.sruja`

## CLI Notes

- `sruja scan .` — Picks up `index.scip` automatically if present in root.
- `npx @sourcegraph/scip-typescript --infer` — The easiest way to generate SCIP for TS/JS projects.
- `cargo scip` — The standard way to generate SCIP for Rust projects.

## Incorrect Approach

- Running SCIP indexers on a broken build (they will fail).
- Assuming `index.scip` is always up to date; if significant code changes occurred, regenerate it.
- Fabricating semantic links that aren't backed by SCIP evidence.

## Summary

**For deep semantic context, check for or generate `index.scip`. Use `npx @sourcegraph/scip-typescript --infer` for TS or `cargo scip` for Rust. Sruja automatically enriches its graph with this data for higher-confidence impact analysis.**

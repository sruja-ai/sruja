# WASM Integration for Go DSL in Studio

## Overview
- Compile Go to WebAssembly to expose DSL parse and print functions to the browser.
- Use `wasm_exec.js` and a small Go bridge to bind functions to `window`.
- Call these functions from React to convert between DSL and `ArchitectureJSON`.

## Build
- Script: `scripts/build-wasm.sh`
- Output: `apps/studio/public/wasm/sruja.wasm` and `apps/studio/public/wasm/wasm_exec.js`
- Requirements: Go 1.25+

## Exposed Functions
- `sruja_parse_dsl(dsl: string) -> { ok: boolean, json?: string, error?: string }`
- `sruja_json_to_dsl(json: string) -> { ok: boolean, dsl?: string, error?: string }`

## UI Usage
- Loader: `apps/studio/src/wasm.ts` initializes WASM and returns an API.
- Studio: `apps/studio/src/App.tsx` initializes viewer and, if WASM loads, parses a sample DSL and renders it.

## Go Bridge
- File: `wasm/main.go`
- Dependencies: `pkg/language`, `pkg/export/json`
- Converts DSL→JSON via parser and exporter; JSON→DSL via JSON→AST converter and printer.

## JSON→AST Converter
- File: `pkg/export/json/json_import.go`
- Maps `ArchitectureJSON` to `language` AST for the printer.

## Index HTML
- Adds `<script src="/wasm/wasm_exec.js"></script>` to load the runtime.

## Notes
- On first run, build the WASM: `bash scripts/build-wasm.sh`.
- If WASM fails to load, Studio falls back to the bundled sample JSON.


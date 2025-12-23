#!/usr/bin/env node
// apps/designer/scripts/check-wasm-freshness.mjs
// Check if WASM file is fresh (recently built) and warn if stale

import { statSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const WASM_PATHS = [
  resolve(__dirname, "../public/wasm/sruja.wasm"),
  resolve(__dirname, "../../website/public/wasm/sruja.wasm"),
];

const MAX_AGE_MS = 5 * 60 * 1000; // 5 minutes - if WASM is older, warn

function checkWasmFreshness() {
  let wasmPath = null;
  let wasmStats = null;

  for (const path of WASM_PATHS) {
    try {
      const stats = statSync(path);
      if (!wasmStats || stats.mtime > wasmStats.mtime) {
        wasmStats = stats;
        wasmPath = path;
      }
    } catch (e) {
      // File doesn't exist, continue
    }
  }

  if (!wasmStats) {
    console.warn("⚠️  WASM file not found. Run 'make wasm' to build it.");
    return false;
  }

  const ageMs = Date.now() - wasmStats.mtime.getTime();
  const ageMinutes = Math.floor(ageMs / 60000);

  if (ageMs > MAX_AGE_MS) {
    console.warn(
      `⚠️  WASM file is ${ageMinutes} minutes old (${wasmPath})\n` +
      `   If you made Go code changes, rebuild with: make wasm\n` +
      `   Then restart the dev server or run: npm run copy:wasm`
    );
    return false;
  }

  console.log(`✅ WASM file is fresh (${ageMinutes} min old): ${wasmPath}`);
  return true;
}

checkWasmFreshness();


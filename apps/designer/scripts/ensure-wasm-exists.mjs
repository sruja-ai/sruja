// Check if WASM exists, if not wait a bit for website to build it
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
// Rust WASM output (wasm-pack / wasm-bindgen) - designer copies from website
const websiteWasmPath = join(
  __dirname,
  "..",
  "..",
  "website",
  "public",
  "wasm",
  "rust",
  "sruja_wasm_bg.wasm"
);

if (!existsSync(websiteWasmPath)) {
  console.warn("⚠️  WASM not found at", websiteWasmPath);
  console.warn("⚠️  Run: make wasm (or build website ensure:wasm first)");
}

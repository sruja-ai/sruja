import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, copyFileSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, "..", "..", "..");
const websiteDir = join(projectRoot, "apps", "website");
const publicWasmDir = join(websiteDir, "public", "wasm");
const wasmOut = join(publicWasmDir, "rust", "sruja_wasm_bg.wasm");

try {
  if (!existsSync(publicWasmDir)) {
    mkdirSync(publicWasmDir, { recursive: true });
  }

  // Check if WASM already exists and is recent (less than 10 seconds old)
  const wasmExists = existsSync(wasmOut);
  if (wasmExists) {
    try {
      const wasmStats = statSync(wasmOut);
      const ageSeconds = (Date.now() - wasmStats.mtimeMs) / 1000;
      if (ageSeconds < 10) {
        console.log("✓ Rust WASM already exists and is recent, skipping build");
        process.exit(0);
      }
    } catch (e) {
      // If we can't check stats, proceed with build
    }
  }

  const variant = (process.env.SRUJA_WASM_VARIANT || "full").toLowerCase();
  let target = "wasm";
  if (variant === "tiny") {
    target = "wasm-tiny";
  }

  console.log(`Building Rust WASM (${variant} variant)...`);
  const build = spawnSync("bash", ["-lc", `make ${target}`], {
    cwd: projectRoot,
    stdio: "inherit",
  });
  if (build.status !== 0) {
    console.error("❌ Rust WASM build failed");
    process.exit(build.status || 1);
  }

  console.log("✅ Rust WASM build complete");
} catch (error) {
  console.error("Error building Rust WASM:", error);
  // Ignore errors, dev server will still run; HTML preview will show a friendly error
}

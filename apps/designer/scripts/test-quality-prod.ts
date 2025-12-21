// apps/designer/scripts/test-quality-prod.ts
// Build production version, serve it, and run e2e tests
import { execSync, spawn } from "child_process";
import { join, dirname } from "path";
import { existsSync } from "fs";
import { fileURLToPath } from "url";
import { promisify } from "util";

const sleep = promisify(setTimeout);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const DESIGNER_DIR = join(__dirname, "..");
const PREVIEW_PORT = 4322;
const PREVIEW_URL = `http://localhost:${PREVIEW_PORT}`;

async function main() {
  console.log("🏗️  Building production version...");
  try {
    execSync("npm run build:test", {
      cwd: DESIGNER_DIR,
      stdio: "inherit",
    });
    console.log("✅ Build completed\n");
  } catch (error) {
    console.error("❌ Build failed:", error);
    process.exit(1);
  }

  // Check if dist exists
  const distDir = join(DESIGNER_DIR, "dist");
  if (!existsSync(distDir)) {
    console.error("❌ dist directory not found after build");
    process.exit(1);
  }

  console.log(`🚀 Starting preview server on port ${PREVIEW_PORT}...`);
  const previewProcess = spawn("npm", ["run", "preview:test"], {
    cwd: DESIGNER_DIR,
    stdio: "pipe",
    shell: true,
    detached: false,
  });

  let previewReady = false;
  let previewOutput = "";

  previewProcess.stdout?.on("data", (data) => {
    const output = data.toString();
    previewOutput += output;
    process.stdout.write(output);
    
    // Check if server is ready
    if (output.includes("Local:") || 
        output.includes(`localhost:${PREVIEW_PORT}`) ||
        output.includes(`http://`) ||
        output.includes("ready in")) {
      previewReady = true;
    }
  });

  previewProcess.stderr?.on("data", (data) => {
    const output = data.toString();
    previewOutput += output;
    // Don't write stderr to stdout, just collect it
  });

  // Also check if server is responding
  async function checkServerReady(): Promise<boolean> {
    try {
      const response = await fetch(`${PREVIEW_URL}/designer`);
      return response.ok;
    } catch {
      return false;
    }
  }

  // Wait for preview server to be ready
  let serverCheckAttempts = 0;
  const maxAttempts = 60; // 30 seconds max

  while (!previewReady && serverCheckAttempts < maxAttempts) {
    await sleep(500);
    serverCheckAttempts++;
    
    // Try to check if server is responding
    if (await checkServerReady()) {
      previewReady = true;
      break;
    }
  }

  if (!previewReady) {
    console.error("❌ Preview server failed to start");
    console.error("Output:", previewOutput);
    previewProcess.kill();
    process.exit(1);
  }

  // Give server a moment to fully stabilize
  await sleep(2000);
  console.log(`\n✅ Preview server ready at ${PREVIEW_URL}\n`);

  // Run e2e test against preview server
  console.log("🧪 Running e2e test against production build...\n");
  let testSuccess = false;
  try {
    execSync("npx playwright test tests/ecommerce-quality.spec.ts", {
      cwd: DESIGNER_DIR,
      stdio: "inherit",
      env: { ...process.env, PLAYWRIGHT_BASE_URL: PREVIEW_URL },
    });
    console.log("\n✅ E2E test completed");
    testSuccess = true;
  } catch (error) {
    console.error("\n❌ E2E test failed");
    testSuccess = false;
  } finally {
    // Cleanup: kill preview server
    console.log("\n🛑 Stopping preview server...");
    previewProcess.kill("SIGTERM");
    
    // Wait a bit for graceful shutdown
    await sleep(1000);
    
    // Force kill if still running
    try {
      previewProcess.kill("SIGKILL");
    } catch {
      // Process already dead
    }
    
    process.exit(testSuccess ? 0 : 1);
  }
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});

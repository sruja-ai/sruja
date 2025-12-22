// apps/vscode-extension/src/test/runStagingTest.ts
// E2E test runner for staging/pre-release extension from marketplace
import * as path from "path";
import { runTests } from "@vscode/test-electron";
import * as fs from "fs";

const EXTENSION_ID = "sruja-ai.sruja";
// Use a stable VS Code version for testing
// Update periodically to test against newer VS Code versions
const VSCODE_VERSION = "1.85.2";

async function main() {
  try {
    // For staging tests, we test the extension as if it were installed from marketplace
    // We use the built extension from the dist folder
    const extensionDevelopmentPath = path.resolve(__dirname, "../../");
    const extensionTestsPath = path.resolve(__dirname, "./suite/staging/index");
    const testWorkspace = path.resolve(__dirname, "../../test-workspace");

    // Create test workspace if it doesn't exist
    if (!fs.existsSync(testWorkspace)) {
      fs.mkdirSync(testWorkspace, { recursive: true });
    }

    // Create a sample .sruja file for testing
    const testFile = path.join(testWorkspace, "test.sruja");
    if (!fs.existsSync(testFile)) {
      fs.writeFileSync(
        testFile,
        `architecture "Test" {
  system TestSystem "Test System"
}`
      );
    }

    console.log("🧪 Running staging e2e tests...");
    console.log(`   Extension ID: ${EXTENSION_ID}`);
    console.log(`   Test workspace: ${testWorkspace}`);
    console.log(`   Testing extension as if installed from marketplace (pre-release)`);

    // Run tests - the extension will be loaded from the development path
    // In a real scenario, this would be installed from marketplace
    // For staging, we test the built extension to verify it works correctly
    await runTests({
      extensionDevelopmentPath,
      extensionTestsPath,
      version: VSCODE_VERSION,
      launchArgs: [
        testWorkspace,
        "--disable-extensions", // Disable other extensions to isolate our extension
      ],
    });

    console.log("✅ Staging e2e tests completed successfully");
  } catch (err) {
    console.error("❌ Failed to run staging e2e tests");
    console.error(err);
    process.exit(1);
  }
}

main();

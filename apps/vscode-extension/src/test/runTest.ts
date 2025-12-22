import * as path from "path";

import { runTests } from "@vscode/test-electron";

async function main() {
  try {
    // The folder containing the Extension Manifest package.json
    // Passed to `--extensionDevelopmentPath`
    // __dirname in compiled code is: out/apps/vscode-extension/src/test
    // We need to go up to the extension root (where package.json is)
    // From out/apps/vscode-extension/src/test, go up 3 levels to out/apps/vscode-extension
    // Then resolve to the actual extension root by going up from 'out' to the workspace root
    const compiledOutDir = path.resolve(__dirname, "../../.."); // out/apps/vscode-extension
    const extensionDevelopmentPath = path.resolve(compiledOutDir, "../.."); // Go from out/apps/vscode-extension to apps/vscode-extension

    // The path to test runner
    // Passed to --extensionTestsPath
    const extensionTestsPath = path.resolve(__dirname, "./suite/index");

    // Download VS Code, unzip it and run the integration test
    await runTests({ extensionDevelopmentPath, extensionTestsPath, version: "1.85.2" });
  } catch (err) {
    console.error("Failed to run tests:", err);
    if (err instanceof Error) {
      console.error("Error message:", err.message);
      console.error("Error stack:", err.stack);
    }
    process.exit(1);
  }
}

main();

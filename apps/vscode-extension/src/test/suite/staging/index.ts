// apps/vscode-extension/src/test/suite/staging/index.ts
import * as path from "path";
import Mocha from "mocha";
import { glob } from "glob";

export async function run(): Promise<void> {
  // Create the mocha test
  const mocha = new Mocha({
    ui: "tdd",
    color: true,
    timeout: 60_000, // 60 seconds timeout for e2e tests
  });

  const testsRoot = path.resolve(__dirname, "..");

  // Find all staging test files
  const files = await glob("**/staging/**/*.test.js", { cwd: testsRoot });

  return new Promise((resolve, reject) => {
    try {
      files.forEach((f: string) => mocha.addFile(path.resolve(testsRoot, f)));

      mocha.run((failures: number) => {
        if (failures > 0) {
          reject(new Error(`${failures} test(s) failed.`));
        } else {
          resolve();
        }
      });
    } catch (err) {
      console.error("Error running staging tests:", err);
      reject(err);
    }
  });
}

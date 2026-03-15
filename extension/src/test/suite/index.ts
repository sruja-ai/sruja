import * as path from "path";
import { globSync } from "glob";

interface TestResult {
  name: string;
  passed: boolean;
  error?: Error;
}

const tests: { name: string; fn: () => Promise<void> }[] = [];
const beforeAllFns: (() => Promise<void>)[] = [];
let currentSuite = "";

function describe(name: string, fn: () => void): void {
  const prev = currentSuite;
  currentSuite = name;
  fn();
  currentSuite = prev;
}

function it(name: string, fn: () => Promise<void>): void {
  tests.push({ name: `${currentSuite} - ${name}`, fn });
}

function beforeAll(fn: () => Promise<void>): void {
  beforeAllFns.push(fn);
}

const assert = {
  ok: (value: unknown, message?: string) => {
    if (!value) throw new Error(message ?? "assertion failed");
  },
  equal: (actual: unknown, expected: unknown, message?: string) => {
    if (actual !== expected) throw new Error(message ?? `expected ${expected} but got ${actual}`);
  },
  fail: (message: string) => {
    throw new Error(message);
  },
  doesNotReject: async (fn: () => Promise<void>, message: string) => {
    try {
      await fn();
    } catch (err) {
      throw new Error(`${message}: ${(err as Error).message}`);
    }
  },
};

export function run(): Promise<void> {
  const testsRoot = path.resolve(__dirname, ".");
  const files = globSync("**/*.e2e.js", { cwd: testsRoot });

  for (const file of files) {
    const module = require(path.join(testsRoot, file));
    if (typeof module.runTests === "function") {
      module.runTests({ describe, it, beforeAll, assert });
    }
  }

  return (async () => {
    for (const fn of beforeAllFns) {
      await fn();
    }

    const results: TestResult[] = [];
    for (const test of tests) {
      try {
        await test.fn();
        results.push({ name: test.name, passed: true });
        console.log(`✓ ${test.name}`);
      } catch (err) {
        results.push({ name: test.name, passed: false, error: err as Error });
        console.error(`✗ ${test.name}`);
        console.error((err as Error).message);
      }
    }

    const failed = results.filter((r) => !r.passed);
    if (failed.length > 0) {
      throw new Error(`${failed.length} test(s) failed.`);
    }
  })();
}

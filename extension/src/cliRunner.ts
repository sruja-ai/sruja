/**
 * Run Sruja CLI commands. Exec function is injectable for tests.
 */

import { execFile } from "child_process";

export interface CliResult {
  stdout: string;
  stderr: string;
  code: number;
}

export type ExecFileFn = (
  command: string,
  args: string[],
  options: { encoding: "utf8"; timeout?: number; maxBuffer?: number; cwd?: string },
  callback: (err: Error | null, stdout: string, stderr: string) => void
) => void;

const LINT_TIMEOUT_MS = 15_000;
const LINT_MAX_BUFFER = 2 * 1024 * 1024;
const CLI_TIMEOUT_MS = 120_000;
const CLI_MAX_BUFFER = 4 * 1024 * 1024;

function normalizeBuffer(stdout: unknown, stderr: unknown): { stdout: string; stderr: string } {
  return {
    stdout: typeof stdout === "string" ? stdout : "",
    stderr: typeof stderr === "string" ? stderr : "",
  };
}

/**
 * Run "sruja lint --format json <filePath>". Uses injectable exec for tests.
 */
export function runLintJson(
  srujaPath: string,
  filePath: string,
  execFn: ExecFileFn = execFile
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve) => {
    execFn(
      srujaPath,
      ["lint", "--format", "json", filePath],
      { encoding: "utf8", timeout: LINT_TIMEOUT_MS, maxBuffer: LINT_MAX_BUFFER },
      (_err: Error | null, stdout: string, stderr: string) => {
        resolve(normalizeBuffer(stdout, stderr));
      }
    );
  });
}

/**
 * Run a Sruja CLI command in the given cwd. Uses injectable exec for tests.
 */
export function runCli(
  srujaPath: string,
  args: string[],
  cwd: string,
  execFn: ExecFileFn = execFile
): Promise<CliResult> {
  return new Promise((resolve) => {
    execFn(
      srujaPath,
      args,
      { encoding: "utf8", cwd, timeout: CLI_TIMEOUT_MS, maxBuffer: CLI_MAX_BUFFER },
      (err: Error | null, stdout: string, stderr: string) => {
        const { stdout: out, stderr: errOut } = normalizeBuffer(stdout, stderr);
        const code: number = err ? 1 : 0;
        resolve({ stdout: out, stderr: errOut, code });
      }
    );
  });
}

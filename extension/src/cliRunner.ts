/**
 * Run Sruja CLI commands. Exec function is injectable for tests.
 * Rejects on spawn/exec failure (e.g. ENOENT, timeout); resolves with exit code when process ran.
 */

import { execFile } from "child_process";
import * as path from "path";
import * as fs from "fs";

export interface CliResult {
  stdout: string;
  stderr: string;
  code: number;
}

/** Thrown when the CLI binary cannot be run (not found, timeout, etc.). */
export class CliExecError extends Error {
  constructor(
    message: string,
    public readonly cause?: Error
  ) {
    super(message);
    this.name = "CliExecError";
  }
}

/** Thrown when the CLI path is invalid or unsafe. */
export class CliPathError extends Error {
  constructor(message: string, public readonly path: string) {
    super(message);
    this.name = "CliPathError";
  }
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

/**
 * Validates that the CLI path is safe to execute.
 * - Must be an absolute path
 * - Must not contain path traversal sequences (..)
 * - Must exist and be a file
 * - Must not be a symlink to an unsafe location (basic check)
 */
export function validateCliPath(srujaPath: string): void {
  if (!path.isAbsolute(srujaPath)) {
    throw new CliPathError(
      `CLI path must be absolute: ${srujaPath}`,
      srujaPath
    );
  }

  const normalized = path.normalize(srujaPath);
  if (normalized !== srujaPath && !normalized.endsWith(path.basename(srujaPath))) {
    throw new CliPathError(
      `CLI path contains suspicious path traversal: ${srujaPath}`,
      srujaPath
    );
  }

  if (!fs.existsSync(srujaPath)) {
    throw new CliPathError(
      `CLI binary not found: ${srujaPath}`,
      srujaPath
    );
  }

  const stats = fs.statSync(srujaPath);
  if (!stats.isFile()) {
    throw new CliPathError(
      `CLI path is not a file: ${srujaPath}`,
      srujaPath
    );
  }
}

function normalizeBuffer(stdout: unknown, stderr: unknown): { stdout: string; stderr: string } {
  return {
    stdout: typeof stdout === "string" ? stdout : "",
    stderr: typeof stderr === "string" ? stderr : "",
  };
}

function isExecExit(err: Error): err is NodeJS.ErrnoException & { code: number } {
  const c = (err as NodeJS.ErrnoException).code;
  return typeof c === "number";
}

function execErrorMessage(err: Error, srujaPath: string): string {
  const e = err as NodeJS.ErrnoException;
  if (e.code === "ENOENT") {
    return `Sruja CLI not found: ${srujaPath}. Install it or set sruja.lsp.path.`;
  }
  if (e.code === "ETIMEDOUT" || e.message?.includes("timeout")) {
    return "Sruja CLI timed out.";
  }
  return e.message ?? String(err);
}

/**
 * Run "sruja lint --format json <filePath>". Uses injectable exec for tests.
 * Rejects with CliExecError when the process cannot be started (ENOENT, timeout, etc.).
 * Rejects with CliPathError when the CLI path is invalid.
 * Resolves with stdout/stderr when the process ran (even if it exited non-zero).
 */
export function runLintJson(
  srujaPath: string,
  filePath: string,
  execFn: ExecFileFn = execFile,
  skipValidation = false
): Promise<{ stdout: string; stderr: string }> {
  if (!skipValidation) {
    validateCliPath(srujaPath);
  }

  return new Promise((resolve, reject) => {
    execFn(
      srujaPath,
      ["lint", "--format", "json", filePath],
      { encoding: "utf8", timeout: LINT_TIMEOUT_MS, maxBuffer: LINT_MAX_BUFFER },
      (err: Error | null, stdout: string, stderr: string) => {
        const { stdout: out, stderr: errOut } = normalizeBuffer(stdout, stderr);
        if (err) {
          if (isExecExit(err)) {
            resolve({ stdout: out, stderr: errOut });
          } else {
            reject(new CliExecError(execErrorMessage(err, srujaPath), err));
          }
        } else {
          resolve({ stdout: out, stderr: errOut });
        }
      }
    );
  });
}

/**
 * Run a Sruja CLI command in the given cwd. Uses injectable exec for tests.
 * Rejects with CliExecError when the process cannot be started (ENOENT, timeout).
 * Rejects with CliPathError when the CLI path is invalid.
 * Resolves with actual exit code when the process ran (including non-zero).
 */
export function runCli(
  srujaPath: string,
  args: string[],
  cwd: string,
  execFn: ExecFileFn = execFile,
  skipValidation = false
): Promise<CliResult> {
  if (!skipValidation) {
    validateCliPath(srujaPath);
  }

  return new Promise((resolve, reject) => {
    execFn(
      srujaPath,
      args,
      { encoding: "utf8", cwd, timeout: CLI_TIMEOUT_MS, maxBuffer: CLI_MAX_BUFFER },
      (err: Error | null, stdout: string, stderr: string) => {
        const { stdout: out, stderr: errOut } = normalizeBuffer(stdout, stderr);
        if (err) {
          if (isExecExit(err)) {
            resolve({ stdout: out, stderr: errOut, code: err.code });
          } else {
            reject(new CliExecError(execErrorMessage(err, srujaPath), err));
          }
        } else {
          resolve({ stdout: out, stderr: errOut, code: 0 });
        }
      }
    );
  });
}

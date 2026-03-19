import { runLintJson, runCli, CliExecError, type ExecFileFn } from "./cliRunner";

describe("cliRunner", () => {
  describe("runLintJson", () => {
    it("returns stdout and stderr from exec callback", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        cb(null, "json stdout", "json stderr");
      };
      const result = await runLintJson("/bin/sruja", "/path/to/file.sruja", execFn, true);
      expect(result.stdout).toBe("json stdout");
      expect(result.stderr).toBe("json stderr");
    });

    it("normalizes non-string stdout/stderr to empty string", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        cb(null, undefined as unknown as string, undefined as unknown as string);
      };
      const result = await runLintJson("/bin/sruja", "/f", execFn, true);
      expect(result.stdout).toBe("");
      expect(result.stderr).toBe("");
    });

    it("calls exec with correct command and args", async () => {
      let captured: { cmd: string; args: string[] } = { cmd: "", args: [] };
      const execFn: ExecFileFn = (cmd, args, _opts, cb) => {
        captured = { cmd, args };
        cb(null, "", "");
      };
      await runLintJson("/bin/sruja", "/f.sruja", execFn, true);
      expect(captured.cmd).toBe("/bin/sruja");
      expect(captured.args).toEqual(["lint", "--format", "json", "/f.sruja"]);
    });

    it("rejects with CliExecError when spawn fails", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        const err = new Error("spawn ENOENT") as NodeJS.ErrnoException;
        err.code = "ENOENT";
        cb(err, "", "");
      };
      await expect(runLintJson("/bin/sruja", "/f.sruja", execFn, true)).rejects.toThrow(CliExecError);
    });
  });

  describe("runCli", () => {
    it("returns code 0 when exec succeeds", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => cb(null, "out", "");
      const result = await runCli("/bin/sruja", ["status", "-r", "."], "/cwd", execFn, true);
      expect(result.code).toBe(0);
      expect(result.stdout).toBe("out");
      expect(result.stderr).toBe("");
    });

    it("returns actual exit code when process exits non-zero", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        const err = Object.assign(new Error("Command failed"), { code: 2 }) as unknown as NodeJS.ErrnoException;
        cb(err, "out", "err");
      };
      const result = await runCli("/bin/sruja", ["status"], "/cwd", execFn, true);
      expect(result.code).toBe(2);
      expect(result.stdout).toBe("out");
      expect(result.stderr).toBe("err");
    });

    it("rejects with CliExecError when spawn fails", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        const err = new Error("spawn ENOENT") as NodeJS.ErrnoException;
        err.code = "ENOENT";
        cb(err, "", "");
      };
      await expect(runCli("/bin/sruja", ["status"], "/cwd", execFn, true)).rejects.toThrow(CliExecError);
    });
  });
});

import { runLintJson, runCli, type ExecFileFn } from "./cliRunner";

describe("cliRunner", () => {
  describe("runLintJson", () => {
    it("returns stdout and stderr from exec callback", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        cb(null, "json stdout", "json stderr");
      };
      const result = await runLintJson("/bin/sruja", "/path/to/file.sruja", execFn);
      expect(result.stdout).toBe("json stdout");
      expect(result.stderr).toBe("json stderr");
    });

    it("normalizes non-string stdout/stderr to empty string", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
        cb(null, undefined as unknown as string, undefined as unknown as string);
      };
      const result = await runLintJson("/bin/sruja", "/f", execFn);
      expect(result.stdout).toBe("");
      expect(result.stderr).toBe("");
    });

    it("calls exec with correct command and args", async () => {
      let captured: { cmd: string; args: string[] } = { cmd: "", args: [] };
      const execFn: ExecFileFn = (cmd, args, _opts, cb) => {
        captured = { cmd, args };
        cb(null, "", "");
      };
      await runLintJson("/bin/sruja", "/f.sruja", execFn);
      expect(captured.cmd).toBe("/bin/sruja");
      expect(captured.args).toEqual(["lint", "--format", "json", "/f.sruja"]);
    });
  });

  describe("runCli", () => {
    it("returns code 0 when exec succeeds", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => cb(null, "out", "");
      const result = await runCli("/bin/sruja", ["status", "-r", "."], "/cwd", execFn);
      expect(result.code).toBe(0);
      expect(result.stdout).toBe("out");
      expect(result.stderr).toBe("");
    });

    it("returns code 1 when exec fails", async () => {
      const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => cb(new Error("fail"), "out", "err");
      const result = await runCli("/bin/sruja", ["status"], "/cwd", execFn);
      expect(result.code).toBe(1);
      expect(result.stdout).toBe("out");
      expect(result.stderr).toBe("err");
    });
  });
});

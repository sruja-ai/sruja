jest.mock("fs", () => ({
  existsSync: jest.fn(),
  statSync: jest.fn(),
}));

import * as fs from "fs";
import { CliExecError, CliPathError, runCli, validateCliPath, type ExecFileFn } from "./cliRunner";

describe("validateCliPath", () => {
  beforeEach(() => {
    (fs.existsSync as unknown as jest.Mock).mockReset();
    (fs.statSync as unknown as jest.Mock).mockReset();
  });

  it("rejects non-absolute paths", () => {
    (fs.existsSync as unknown as jest.Mock).mockReturnValue(true);
    (fs.statSync as unknown as jest.Mock).mockReturnValue({ isFile: () => true });

    expect(() => validateCliPath("relative/sruja")).toThrow(CliPathError);
  });

  it("rejects suspicious path traversal", () => {
    (fs.existsSync as unknown as jest.Mock).mockReturnValue(true);
    (fs.statSync as unknown as jest.Mock).mockReturnValue({ isFile: () => true });

    expect(() => validateCliPath("/a/b/../c/..")).toThrow(CliPathError);
  });

  it("rejects missing files", () => {
    (fs.existsSync as unknown as jest.Mock).mockReturnValue(false);

    expect(() => validateCliPath("/bin/sruja")).toThrow(CliPathError);
  });

  it("rejects non-files", () => {
    (fs.existsSync as unknown as jest.Mock).mockReturnValue(true);
    (fs.statSync as unknown as jest.Mock).mockReturnValue({ isFile: () => false });

    expect(() => validateCliPath("/bin/sruja")).toThrow(CliPathError);
  });

  it("accepts existing files", () => {
    (fs.existsSync as unknown as jest.Mock).mockReturnValue(true);
    (fs.statSync as unknown as jest.Mock).mockReturnValue({ isFile: () => true });

    expect(() => validateCliPath("/bin/sruja")).not.toThrow();
  });
});

describe("execErrorMessage (via runCli)", () => {
  it("uses a friendly ENOENT message", async () => {
    const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
      const err = new Error("spawn ENOENT") as NodeJS.ErrnoException;
      err.code = "ENOENT";
      cb(err, "", "");
    };

    await expect(runCli("/bin/sruja", ["status"], "/cwd", execFn, true)).rejects.toThrow(
      "Sruja CLI not found: /bin/sruja. Install it or set sruja.lsp.path."
    );
  });

  it("uses a timeout message for ETIMEDOUT", async () => {
    const execFn: ExecFileFn = (_cmd, _args, _opts, cb) => {
      const err = new Error("ETIMEDOUT") as NodeJS.ErrnoException;
      err.code = "ETIMEDOUT";
      cb(err, "", "");
    };

    await expect(runCli("/bin/sruja", ["status"], "/cwd", execFn, true)).rejects.toThrow("Sruja CLI timed out.");
  });
});

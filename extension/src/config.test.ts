import { getSrujaLspPath, useWasmFromLspPath, DEFAULT_SRUJA_CMD } from "./config";

describe("config", () => {
  describe("getSrujaLspPath", () => {
    it("returns default when undefined", () => {
      expect(getSrujaLspPath(undefined)).toBe(DEFAULT_SRUJA_CMD);
    });
    it("returns default when empty string", () => {
      expect(getSrujaLspPath("")).toBe(DEFAULT_SRUJA_CMD);
    });
    it("returns default when whitespace only", () => {
      expect(getSrujaLspPath("  ")).toBe(DEFAULT_SRUJA_CMD);
    });
    it("returns trimmed path when set", () => {
      expect(getSrujaLspPath("  /usr/local/bin/sruja  ")).toBe("/usr/local/bin/sruja");
    });
  });

  describe("useWasmFromLspPath", () => {
    it("returns true when path is unset or empty", () => {
      expect(useWasmFromLspPath(undefined)).toBe(true);
      expect(useWasmFromLspPath("")).toBe(true);
      expect(useWasmFromLspPath("  ")).toBe(true);
    });
    it("returns false when path is set", () => {
      expect(useWasmFromLspPath("/path/to/sruja")).toBe(false);
      expect(useWasmFromLspPath("sruja")).toBe(false);
    });
  });
});

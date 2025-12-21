// apps/designer/src/types/__tests__/layout.test.ts
import { describe, it, expect } from "vitest";
import { isC4Level, toC4Level } from "../layout";

describe("layout type utilities", () => {
  describe("isC4Level", () => {
    it("should return true for valid C4 levels", () => {
      expect(isC4Level("L1")).toBe(true);
      expect(isC4Level("L2")).toBe(true);
      expect(isC4Level("L3")).toBe(true);
      expect(isC4Level("L4")).toBe(true);
    });

    it("should return false for invalid levels", () => {
      expect(isC4Level("L0")).toBe(false);
      expect(isC4Level("L5")).toBe(false);
      expect(isC4Level("L")).toBe(false);
      expect(isC4Level("1")).toBe(false);
      expect(isC4Level("level1")).toBe(false);
    });

    it("should return false for null", () => {
      expect(isC4Level(null as unknown as string | undefined)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isC4Level(undefined)).toBe(false);
    });

    it("should return false for empty string", () => {
      expect(isC4Level("")).toBe(false);
    });
  });

  describe("toC4Level", () => {
    it("should return valid C4 level for valid input", () => {
      expect(toC4Level("L1")).toBe("L1");
      expect(toC4Level("L2")).toBe("L2");
      expect(toC4Level("L3")).toBe("L3");
      expect(toC4Level("L4")).toBe("L4");
    });

    it("should return fallback for invalid input", () => {
      expect(toC4Level("invalid")).toBe("L1");
      expect(toC4Level("L0")).toBe("L1");
      expect(toC4Level("L5")).toBe("L1");
    });

    it("should return fallback for null", () => {
      expect(toC4Level(null as unknown as string | undefined)).toBe("L1");
    });

    it("should return fallback for undefined", () => {
      expect(toC4Level(undefined)).toBe("L1");
    });

    it("should use custom fallback", () => {
      expect(toC4Level("invalid", "L2")).toBe("L2");
      expect(toC4Level(null as unknown as string | undefined, "L3")).toBe("L3");
      expect(toC4Level(undefined, "L4")).toBe("L4");
    });

    it("should handle case sensitivity", () => {
      expect(toC4Level("l1")).toBe("L1"); // Should normalize to uppercase
      expect(toC4Level("l2")).toBe("L1"); // Invalid, uses fallback
    });
  });
});

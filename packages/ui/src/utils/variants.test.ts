// packages/ui/src/utils/variants.test.ts
import { describe, it, expect } from "vitest";
import { vx } from "./variants";

describe("vx", () => {
  it("should merge multiple class strings", () => {
    const result = vx("class1", "class2", "class3");
    expect(result).toContain("class1");
    expect(result).toContain("class2");
    expect(result).toContain("class3");
  });

  it("should filter out falsy values", () => {
    const result = vx("class1", undefined, false, "class2");
    expect(result).not.toContain("undefined");
    expect(result).not.toContain("false");
    expect(result).toContain("class1");
    expect(result).toContain("class2");
  });

  it("should handle empty arrays", () => {
    const result = vx();
    expect(result).toBe("");
  });

  it("should handle only falsy values", () => {
    const result = vx(undefined, false);
    expect(result).toBe("");
  });

  it("should merge Tailwind classes correctly", () => {
    // twMerge should handle conflicting classes
    const result = vx("p-4", "p-6");
    // Should only keep the last conflicting class
    expect(result).toBe("p-6");
  });

  it("should handle mixed valid and invalid classes", () => {
    const result = vx("valid-class", undefined, "another-valid", false);
    expect(result).toContain("valid-class");
    expect(result).toContain("another-valid");
  });

  it("should handle single class", () => {
    const result = vx("single-class");
    expect(result).toBe("single-class");
  });

  it("should not filter null (unlike cn)", () => {
    // vx doesn't explicitly filter null, but twMerge should handle it
    const result = vx("class1", null as unknown as string, "class2");
    // twMerge will handle null by converting to string or filtering
    expect(result).toContain("class1");
    expect(result).toContain("class2");
  });
});

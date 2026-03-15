import { parseJsonSafe, isObject } from "./safeJson";

describe("safeJson", () => {
  describe("parseJsonSafe", () => {
    it("returns value when JSON is valid", () => {
      const result = parseJsonSafe<{ a: number }>('{"a":1}');
      expect(result.ok).toBe(true);
      if (result.ok) expect(result.value.a).toBe(1);
    });
    it("returns error when JSON is invalid", () => {
      const result = parseJsonSafe("{ invalid }");
      expect(result.ok).toBe(false);
      if (!result.ok) expect(result.error).toBeDefined();
    });
    it("fails when guard returns false", () => {
      const guard = (v: unknown): v is { x: number } =>
        typeof v === "object" && v !== null && "x" in v && typeof (v as { x: unknown }).x === "number";
      const result = parseJsonSafe('{"y":1}', guard);
      expect(result.ok).toBe(false);
      if (!result.ok) expect(result.error).toContain("validation");
    });
    it("succeeds when guard returns true", () => {
      const guard = (v: unknown): v is { x: number } =>
        typeof v === "object" && v !== null && "x" in v;
      const result = parseJsonSafe<{ x: number }>('{"x":2}', guard);
      expect(result.ok).toBe(true);
      if (result.ok) expect(result.value.x).toBe(2);
    });
  });

  describe("isObject", () => {
    it("returns true for plain object", () => {
      expect(isObject({})).toBe(true);
      expect(isObject({ a: 1 })).toBe(true);
    });
    it("returns false for null, array, primitive", () => {
      expect(isObject(null)).toBe(false);
      expect(isObject([])).toBe(false);
      expect(isObject(1)).toBe(false);
      expect(isObject("s")).toBe(false);
    });
  });
});

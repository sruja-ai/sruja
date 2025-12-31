// packages/shared/src/registry/__tests__/propertyKeys.test.ts
// Unit tests for propertyKeys registry and isInsideBlock function

import { describe, it, expect } from "vitest";
import { isInsideBlock, propertyKeys } from "../propertyKeys";

describe("propertyKeys", () => {
  it("should export a readonly array of property keys", () => {
    expect(Array.isArray(propertyKeys)).toBe(true);
    expect(propertyKeys.length).toBeGreaterThan(0);
    expect(propertyKeys[0]).toBe("capacity.instanceType");
  });

  it("should contain expected property key categories", () => {
    const keys = propertyKeys.join(" ");
    expect(keys).toContain("capacity.");
    expect(keys).toContain("obs.");
    expect(keys).toContain("compliance.");
    expect(keys).toContain("cost.");
  });
});

describe("isInsideBlock", () => {
  describe("input validation", () => {
    it("should return false for empty text", () => {
      expect(isInsideBlock("", 1, "properties")).toBe(false);
    });

    it("should return false for invalid position line (less than 1)", () => {
      expect(isInsideBlock("properties {\n  key: value\n}", 0, "properties")).toBe(false);
      expect(isInsideBlock("properties {\n  key: value\n}", -1, "properties")).toBe(false);
    });

    it("should return false for non-integer position line", () => {
      expect(isInsideBlock("properties {\n  key: value\n}", 1.5, "properties")).toBe(false);
    });

    it("should return false for position line beyond text bounds", () => {
      const text = "properties {\n  key: value\n}";
      expect(isInsideBlock(text, 100, "properties")).toBe(false);
    });

    it("should return false for invalid block type", () => {
      const text = "properties {\n  key: value\n}";
      // @ts-expect-error - Testing invalid input
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      expect(isInsideBlock(text, 2, "invalid" as any)).toBe(false);
    });
  });

  describe("simple block detection", () => {
    it("should detect position inside properties block", () => {
      const text = "properties {\n  key: value\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should detect position inside metadata block", () => {
      const text = "metadata {\n  key: value\n}";
      expect(isInsideBlock(text, 2, "metadata")).toBe(true);
    });

    it("should return false for position before block", () => {
      const text = "some text\nproperties {\n  key: value\n}";
      expect(isInsideBlock(text, 1, "properties")).toBe(false);
    });

    it("should return false for position after block", () => {
      const text = "properties {\n  key: value\n}\nsome text";
      expect(isInsideBlock(text, 4, "properties")).toBe(false);
    });

    it("should return false for position on closing brace", () => {
      const text = "properties {\n  key: value\n}";
      expect(isInsideBlock(text, 3, "properties")).toBe(false);
    });
  });

  describe("block declaration patterns", () => {
    it("should handle properties block on same line as opening brace", () => {
      const text = "properties {\n  key: value\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should handle properties block with whitespace", () => {
      const text = "properties   {\n  key: value\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should handle properties block on previous line", () => {
      const text = "properties\n{\n  key: value\n}";
      expect(isInsideBlock(text, 3, "properties")).toBe(true);
    });

    it("should be case-insensitive", () => {
      const text = "PROPERTIES {\n  key: value\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });
  });

  describe("nested blocks", () => {
    it("should correctly identify position in outer block", () => {
      const text = "properties {\n  inner {\n    key: value\n  }\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
      expect(isInsideBlock(text, 3, "properties")).toBe(true);
    });

    it("should return false for position in nested block of different type", () => {
      const text = "properties {\n  metadata {\n    key: value\n  }\n}";
      // Position is inside nested metadata block, not properties block
      expect(isInsideBlock(text, 3, "properties")).toBe(false);
      expect(isInsideBlock(text, 3, "metadata")).toBe(true);
    });

    it("should handle deeply nested blocks", () => {
      const text =
        "properties {\n  a {\n    b {\n      c {\n        key: value\n      }\n    }\n  }\n}";
      // Position at key: value is inside properties block
      expect(isInsideBlock(text, 5, "properties")).toBe(true);
    });
  });

  describe("multiple blocks", () => {
    it("should correctly identify which block position is in", () => {
      const text = "properties {\n  key1: value1\n}\nmetadata {\n  key2: value2\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
      expect(isInsideBlock(text, 2, "metadata")).toBe(false);
      expect(isInsideBlock(text, 5, "metadata")).toBe(true);
      expect(isInsideBlock(text, 5, "properties")).toBe(false);
    });

    it("should handle properties block after metadata block", () => {
      const text = "metadata {\n  key1: value1\n}\nproperties {\n  key2: value2\n}";
      expect(isInsideBlock(text, 5, "properties")).toBe(true);
      expect(isInsideBlock(text, 2, "properties")).toBe(false);
    });
  });

  describe("edge cases", () => {
    it("should handle single-line block", () => {
      const text = "properties { key: value }";
      expect(isInsideBlock(text, 1, "properties")).toBe(true);
    });

    it("should handle block with only opening brace", () => {
      const text = "properties {\n  key: value";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should handle text with multiple opening braces before block", () => {
      const text = "function() {\n  properties {\n    key: value\n  }\n}";
      expect(isInsideBlock(text, 3, "properties")).toBe(true);
    });

    it("should handle block with comments", () => {
      const text = "// comment\nproperties {\n  // another comment\n  key: value\n}";
      expect(isInsideBlock(text, 4, "properties")).toBe(true);
    });

    it("should handle block with string literals containing braces", () => {
      const text = 'properties {\n  key: "value { with braces }"\n}';
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should handle Windows line endings", () => {
      const text = "properties {\r\n  key: value\r\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });

    it("should handle mixed line endings", () => {
      const text = "properties {\r\n  key: value\n}";
      expect(isInsideBlock(text, 2, "properties")).toBe(true);
    });
  });

  describe("real-world scenarios", () => {
    it("should work with typical Sruja DSL structure", () => {
      const text = `element system:MySystem {
  title "My System"
  properties {
    capacity.instanceType: "m5.large"
    obs.metrics.application: "prometheus"
  }
}`;
      expect(isInsideBlock(text, 4, "properties")).toBe(true);
      expect(isInsideBlock(text, 5, "properties")).toBe(true);
      expect(isInsideBlock(text, 2, "properties")).toBe(false);
    });

    it("should handle multiple properties blocks in same element", () => {
      const text = `element system:A {
  properties {
    key1: value1
  }
  metadata {
    key2: value2
  }
}`;
      expect(isInsideBlock(text, 3, "properties")).toBe(true);
      expect(isInsideBlock(text, 3, "metadata")).toBe(false);
      expect(isInsideBlock(text, 7, "metadata")).toBe(false);
      expect(isInsideBlock(text, 7, "properties")).toBe(false);
    });
  });
});

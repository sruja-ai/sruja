// apps/designer/src/utils/__tests__/stringUtils.test.ts
import { describe, it, expect } from "vitest";
import { slugify } from "../stringUtils";

describe("stringUtils", () => {
  describe("slugify", () => {
    it("should convert text to lowercase", () => {
      expect(slugify("HELLO WORLD")).toBe("hello-world");
      expect(slugify("Hello World")).toBe("hello-world");
    });

    it("should trim whitespace", () => {
      expect(slugify("  hello world  ")).toBe("hello-world");
      expect(slugify("\thello\nworld\t")).toBe("hello-world");
    });

    it("should replace spaces and special characters with hyphens", () => {
      expect(slugify("hello world")).toBe("hello-world");
      expect(slugify("hello@world#test")).toBe("hello-world-test");
      expect(slugify("hello.world.test")).toBe("hello-world-test");
    });

    it("should preserve alphanumeric characters, hyphens, and underscores", () => {
      expect(slugify("hello-world_123")).toBe("hello-world_123");
      expect(slugify("test_123-abc")).toBe("test_123-abc");
    });

    it("should remove leading and trailing hyphens", () => {
      expect(slugify("-hello-world-")).toBe("hello-world");
      expect(slugify("---hello---world---")).toBe("hello-world");
    });

    it("should collapse multiple consecutive hyphens", () => {
      expect(slugify("hello---world")).toBe("hello-world");
      expect(slugify("hello   world")).toBe("hello-world");
    });

    it("should handle empty strings", () => {
      expect(slugify("")).toBe("");
      expect(slugify("   ")).toBe("");
    });

    it("should handle strings with only special characters", () => {
      expect(slugify("!!!@@@###")).toBe("");
      expect(slugify("---")).toBe("");
    });

    it("should handle unicode characters", () => {
      expect(slugify("héllo wörld")).toBe("hllo-wrld");
      expect(slugify("привет мир")).toBe("");
    });

    it("should handle numbers", () => {
      expect(slugify("test123")).toBe("test123");
      expect(slugify("123test")).toBe("123test");
      expect(slugify("test 123 test")).toBe("test-123-test");
    });

    it("should handle mixed case and special characters", () => {
      expect(slugify("Hello World! How are you?")).toBe("hello-world-how-are-you");
      expect(slugify("Test@#$%^&*()Test")).toBe("test-test");
    });

    it("should handle edge cases", () => {
      expect(slugify("a")).toBe("a");
      expect(slugify("A")).toBe("a");
      expect(slugify("a-b")).toBe("a-b");
      expect(slugify("a_b")).toBe("a_b");
    });
  });
});

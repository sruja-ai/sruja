// apps/designer/src/components/shared/forms/__tests__/utils.test.ts
import { describe, it, expect } from "vitest";
import { slugify, collectIds, generateUniqueId } from "../utils";
import type { SrujaModelDump } from "@sruja/shared";

describe("form utils", () => {
  describe("slugify", () => {
    it("should convert text to lowercase slug", () => {
      expect(slugify("Hello World")).toBe("hello-world");
    });

    it("should handle special characters", () => {
      expect(slugify("Hello, World!")).toBe("hello-world");
    });

    it("should handle multiple spaces", () => {
      expect(slugify("Hello    World")).toBe("hello-world");
    });

    it("should trim leading and trailing dashes", () => {
      expect(slugify("---Hello World---")).toBe("hello-world");
    });

    it("should handle empty string", () => {
      expect(slugify("")).toBe("");
    });

    it("should handle already slugified text", () => {
      expect(slugify("hello-world")).toBe("hello-world");
    });

    it("should handle numbers", () => {
      expect(slugify("System 123")).toBe("system-123");
    });

    it("should preserve underscores", () => {
      expect(slugify("system_name")).toBe("system_name");
    });
  });

  describe("collectIds", () => {
    const createMockDump = (elements: Record<string, any> = {}): SrujaModelDump => ({
      specification: { tags: {}, elements: {} },
      elements,
      relations: [],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: { name: "Test", version: "1.0", generated: "", srujaVersion: "" }
    });

    it("should collect element IDs", () => {
      const data = createMockDump({
        "Person1": { id: "Person1", kind: "person", title: "P1", tags: [], links: [] },
        "System1": { id: "System1", kind: "system", title: "S1", tags: [], links: [] }
      });

      const ids = collectIds(data);
      expect(ids.has("Person1")).toBe(true);
      expect(ids.has("System1")).toBe(true);
    });

    it("should return empty set for null data", () => {
      const ids = collectIds(null);
      expect(ids.size).toBe(0);
    });
  });

  describe("generateUniqueId", () => {
    const createMockDump = (existingIds: string[]): SrujaModelDump => {
      const elements: Record<string, any> = {};
      existingIds.forEach(id => {
        elements[id] = { id, kind: "system", title: id, tags: [], links: [] };
      });
      return {
        specification: { tags: {}, elements: {} },
        elements,
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: { name: "Test", version: "1.0", generated: "", srujaVersion: "" }
      };
    };

    it("should generate unique ID from base string", () => {
      const data = createMockDump([]);
      const id = generateUniqueId("My System", data);
      expect(id).toBe("my-system");
    });

    it("should append number if ID exists", () => {
      const data = createMockDump(["my-system"]);
      const id = generateUniqueId("My System", data);
      expect(id).toBe("my-system-1");
    });

    it("should increment number until unique", () => {
      const data = createMockDump(["my-system", "my-system-1", "my-system-2"]);
      const id = generateUniqueId("My System", data);
      expect(id).toBe("my-system-3");
    });

    it("should use default ID for empty base string", () => {
      const data = createMockDump([]);
      const id = generateUniqueId("", data, "system");
      expect(id).toBe("system");
    });

    it("should use type-specific default", () => {
      const data = createMockDump([]);
      const personId = generateUniqueId("", data, "person");
      expect(personId).toBe("person");
    });

    it("should handle special characters in base", () => {
      const data = createMockDump([]);
      const id = generateUniqueId("System #1!", data);
      expect(id).toBe("system-1");
    });

    it("should handle very long base strings", () => {
      const data = createMockDump([]);
      const longBase = "A".repeat(100);
      const id = generateUniqueId(longBase, data);
      // Slugified version should be shorter or equal (removes special chars, but keeps alphanumeric)
      expect(id).toBeTruthy();
      expect(typeof id).toBe("string");
    });
  });
});

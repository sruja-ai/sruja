// apps/designer/src/utils/__tests__/nodeUtils.test.ts
import { describe, it, expect } from "vitest";
import { generateUniqueId, findNodeInArchitecture, getAllNodeIds } from "../nodeUtils";
import type { SrujaModelDump } from "@sruja/shared";

describe("nodeUtils", () => {
  describe("generateUniqueId", () => {
    it("should return base ID if not in existing set", () => {
      const existingIds = new Set<string>(["other-id"]);
      expect(generateUniqueId("new-id", existingIds)).toBe("new-id");
    });

    it("should append number suffix if base ID exists", () => {
      const existingIds = new Set<string>(["test-id"]);
      expect(generateUniqueId("test-id", existingIds)).toBe("test-id-1");
    });

    it("should increment suffix until unique", () => {
      const existingIds = new Set<string>(["test-id", "test-id-1", "test-id-2"]);
      expect(generateUniqueId("test-id", existingIds)).toBe("test-id-3");
    });

    it("should handle empty existing set", () => {
      const existingIds = new Set<string>();
      expect(generateUniqueId("new-id", existingIds)).toBe("new-id");
    });

    it("should handle many existing IDs", () => {
      const existingIds = new Set<string>(Array.from({ length: 100 }, (_, i) => `test-id-${i}`));
      expect(generateUniqueId("test-id", existingIds)).toBe("test-id-100");
    });
  });

  describe("findNodeInArchitecture", () => {
    const mockArch: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
      },
      relations: [],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    it("should find existing node", () => {
      const node = findNodeInArchitecture(mockArch, "System1");
      expect(node).toBeDefined();
      expect(node?.id).toBe("System1");
    });

    it("should return null for non-existent node", () => {
      const node = findNodeInArchitecture(mockArch, "NonExistent");
      expect(node).toBeNull();
    });

    it("should return null for architecture without elements", () => {
      const archWithoutElements: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: undefined as any,
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const node = findNodeInArchitecture(archWithoutElements, "System1");
      expect(node).toBeNull();
    });

    it("should handle empty elements object", () => {
      const archWithEmptyElements: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: {},
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const node = findNodeInArchitecture(archWithEmptyElements, "System1");
      expect(node).toBeNull();
    });
  });

  describe("getAllNodeIds", () => {
    it("should return all node IDs from architecture", () => {
      const arch: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: {
          System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
          System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
          Container1: {
            id: "Container1",
            kind: "container",
            title: "Container 1",
            tags: [],
            links: [],
          },
        },
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const ids = getAllNodeIds(arch);
      expect(ids.size).toBe(3);
      expect(ids.has("System1")).toBe(true);
      expect(ids.has("System2")).toBe(true);
      expect(ids.has("Container1")).toBe(true);
    });

    it("should return empty set for architecture without elements", () => {
      const archWithoutElements: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: undefined as any,
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const ids = getAllNodeIds(archWithoutElements);
      expect(ids.size).toBe(0);
    });

    it("should return empty set for architecture with empty elements", () => {
      const archWithEmptyElements: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: {},
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const ids = getAllNodeIds(archWithEmptyElements);
      expect(ids.size).toBe(0);
    });

    it("should return Set with unique IDs", () => {
      const arch: SrujaModelDump = {
        specification: { tags: {}, elements: {} },
        elements: {
          System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        },
        relations: [],
        views: {},
        sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      const ids = getAllNodeIds(arch);
      expect(ids).toBeInstanceOf(Set);
      expect(ids.size).toBe(1);
    });
  });
});

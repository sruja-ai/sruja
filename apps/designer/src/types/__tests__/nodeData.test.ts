// apps/designer/src/types/__tests__/nodeData.test.ts
import { describe, it, expect } from "vitest";
import {
  isSystemNodeData,
  isContainerNodeData,
  isComponentNodeData,
  isPersonNodeData,
  getNodeTechnology,
  getNodeTags,
  type SystemNodeData,
  type ContainerNodeData,
  type PersonNodeData,
} from "../nodeData";
import type { C4NodeData } from "../index";

describe("nodeData type guards and helpers", () => {
  describe("isSystemNodeData", () => {
    it("should return true for system node data", () => {
      const nodeData: SystemNodeData = {
        id: "System1",
        label: "Test System",
        type: "system",
      };
      expect(isSystemNodeData(nodeData)).toBe(true);
    });

    it("should return false for non-system node data", () => {
      const nodeData: C4NodeData = {
        id: "Person1",
        label: "Test Person",
        type: "person",
      };
      expect(isSystemNodeData(nodeData)).toBe(false);
    });

    it("should return false for null", () => {
      expect(isSystemNodeData(null)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(isSystemNodeData(undefined)).toBe(false);
    });
  });

  describe("isContainerNodeData", () => {
    it("should return true for container node data", () => {
      const nodeData: ContainerNodeData = {
        id: "Container1",
        label: "Test Container",
        type: "container",
      };
      expect(isContainerNodeData(nodeData)).toBe(true);
    });

    it("should return false for non-container node data", () => {
      const nodeData: C4NodeData = {
        id: "System1",
        label: "Test System",
        type: "system",
      };
      expect(isContainerNodeData(nodeData)).toBe(false);
    });
  });

  describe("isComponentNodeData", () => {
    it("should return true for component node data", () => {
      const nodeData: C4NodeData = {
        id: "Component1",
        label: "Test Component",
        type: "component",
      };
      expect(isComponentNodeData(nodeData)).toBe(true);
    });

    it("should return false for non-component node data", () => {
      const nodeData: C4NodeData = {
        id: "System1",
        label: "Test System",
        type: "system",
      };
      expect(isComponentNodeData(nodeData)).toBe(false);
    });
  });

  describe("isPersonNodeData", () => {
    it("should return true for person node data", () => {
      const nodeData: PersonNodeData = {
        id: "Person1",
        label: "Test Person",
        type: "person",
      };
      expect(isPersonNodeData(nodeData)).toBe(true);
    });

    it("should return false for non-person node data", () => {
      const nodeData: C4NodeData = {
        id: "System1",
        label: "Test System",
        type: "system",
      };
      expect(isPersonNodeData(nodeData)).toBe(false);
    });
  });

  describe("getNodeTechnology", () => {
    it("should return technology string when present", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        technology: "React",
      };
      expect(getNodeTechnology(nodeData)).toBe("react");
    });

    it("should return lowercase technology", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        technology: "REACT",
      };
      expect(getNodeTechnology(nodeData)).toBe("react");
    });

    it("should return empty string when technology is missing", () => {
      const nodeData = {
        id: "System1",
        type: "system",
      };
      expect(getNodeTechnology(nodeData)).toBe("");
    });

    it("should return empty string for null technology", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        technology: null,
      };
      expect(getNodeTechnology(nodeData)).toBe("");
    });

    it("should handle non-string technology", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        technology: 123,
      };
      expect(getNodeTechnology(nodeData)).toBe("123");
    });
  });

  describe("getNodeTags", () => {
    it("should return tags array when present", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        tags: ["tag1", "tag2"],
      };
      expect(getNodeTags(nodeData)).toEqual(["tag1", "tag2"]);
    });

    it("should return empty array when tags are missing", () => {
      const nodeData = {
        id: "System1",
        type: "system",
      };
      expect(getNodeTags(nodeData)).toEqual([]);
    });

    it("should return empty array for null tags", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        tags: null,
      };
      expect(getNodeTags(nodeData)).toEqual([]);
    });

    it("should return empty array for undefined tags", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        tags: undefined,
      };
      expect(getNodeTags(nodeData)).toEqual([]);
    });

    it("should filter out non-string tags", () => {
      const nodeData = {
        id: "System1",
        type: "system",
        tags: ["tag1", 123, "tag2", null],
      };
      const result = getNodeTags(nodeData);
      expect(result).toContain("tag1");
      expect(result).toContain("tag2");
      expect(result.length).toBe(2);
      // Verify all items are strings
      result.forEach((tag) => {
        expect(typeof tag).toBe("string");
      });
    });
  });
});

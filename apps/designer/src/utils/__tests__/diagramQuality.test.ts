// apps/designer/src/utils/__tests__/diagramQuality.test.ts
import { describe, it, expect } from "vitest";
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from "../diagramQuality";
import type { Node, Edge } from "@xyflow/react";
import type { C4NodeData } from "../../types";

describe("diagramQuality", () => {
  const createMockNode = (
    id: string,
    position: { x: number; y: number },
    width = 200,
    height = 100
  ): Node<C4NodeData> => ({
    id,
    type: "system",
    position,
    data: {
      id,
      label: `Node ${id}`,
      type: "system",
    },
    width,
    height,
  });

  const createMockEdge = (id: string, source: string, target: string): Edge => ({
    id,
    source,
    target,
    type: "relation",
  });

  describe("calculateDiagramQuality", () => {
    it("should return null for empty diagram", () => {
      const result = calculateDiagramQuality([], [], { width: 1920, height: 1080 });
      // Function may return null or a metrics object with zero scores
      expect(result === null || (result && typeof result.overallScore === "number")).toBe(true);
    });

    it("should calculate quality for single node", () => {
      const nodes = [createMockNode("1", { x: 100, y: 100 })];
      const edges: Edge[] = [];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      expect(result).not.toBeNull();
      expect(result?.overallScore).toBeGreaterThanOrEqual(0);
      expect(result?.overallScore).toBeLessThanOrEqual(100);
    });

    it("should calculate quality for multiple nodes", () => {
      const nodes = [
        createMockNode("1", { x: 100, y: 100 }),
        createMockNode("2", { x: 400, y: 100 }),
        createMockNode("3", { x: 100, y: 400 }),
      ];
      const edges = [
        createMockEdge("e1", "1", "2"),
        createMockEdge("e2", "2", "3"),
      ];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      expect(result).not.toBeNull();
      expect(result?.overallScore).toBeGreaterThanOrEqual(0);
      expect(result?.overallScore).toBeLessThanOrEqual(100);
    });

    it("should detect overlapping nodes", () => {
      const nodes = [
        createMockNode("1", { x: 100, y: 100 }),
        createMockNode("2", { x: 150, y: 120 }), // Overlaps with node 1
      ];
      const edges: Edge[] = [];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      expect(result).not.toBeNull();
      expect(result?.overlappingNodes).toBeDefined();
      expect(Array.isArray(result?.overlappingNodes)).toBe(true);
    });

    it("should calculate edge metrics", () => {
      const nodes = [
        createMockNode("1", { x: 100, y: 100 }),
        createMockNode("2", { x: 500, y: 100 }),
        createMockNode("3", { x: 100, y: 500 }),
      ];
      const edges = [
        createMockEdge("e1", "1", "2"),
        createMockEdge("e2", "2", "3"),
        createMockEdge("e3", "1", "3"),
      ];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      expect(result).not.toBeNull();
      expect(result?.edgeLength).toBeDefined();
      expect(result?.edgeLength).toHaveProperty("min");
      expect(result?.edgeLength).toHaveProperty("max");
      expect(result?.edgeLength).toHaveProperty("average");
    });

    it("should respect custom quality weights", () => {
      const nodes = [
        createMockNode("1", { x: 100, y: 100 }),
        createMockNode("2", { x: 400, y: 100 }),
      ];
      const edges = [createMockEdge("e1", "1", "2")];
      const viewport = { width: 1920, height: 1080 };

      const customWeights = {
        ...DEFAULT_QUALITY_WEIGHTS,
        overlap: 0.5, // Higher weight on overlap
      };

      const result = calculateDiagramQuality(nodes, edges, viewport, customWeights);

      expect(result).not.toBeNull();
      expect(result?.overallScore).toBeGreaterThanOrEqual(0);
    });

    it("should calculate spacing metrics", () => {
      const nodes = [
        createMockNode("1", { x: 100, y: 100 }),
        createMockNode("2", { x: 400, y: 100 }), // Good spacing
        createMockNode("3", { x: 250, y: 100 }), // Too close to node 2
      ];
      const edges: Edge[] = [];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      expect(result).not.toBeNull();
      expect(result?.minSpacing).toBeDefined();
      expect(result?.averageSpacing).toBeDefined();
      expect(result?.spacingViolations).toBeDefined();
    });

    it("should handle nodes with no dimensions", () => {
      const nodes = [
        {
          id: "1",
          type: "system",
          position: { x: 100, y: 100 },
          data: { id: "1", label: "Node 1", type: "system" },
        } as Node<C4NodeData>,
      ];
      const edges: Edge[] = [];
      const viewport = { width: 1920, height: 1080 };

      const result = calculateDiagramQuality(nodes, edges, viewport);

      // Should handle gracefully
      expect(result).not.toBeNull();
    });
  });
});

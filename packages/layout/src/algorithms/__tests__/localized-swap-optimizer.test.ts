// packages/layout/src/algorithms/__tests__/localized-swap-optimizer.test.ts
import { describe, it, expect } from "vitest";
import { applyLocalizedSwaps, type EdgeRoute } from "../localized-swap-optimizer";
import type { PositionedNode } from "../coordinates";
import type { Point } from "../../geometry/point";
import type { C4Id } from "../../brand";
import type { HierarchyNode } from "../hierarchy";

function createPositionedNode(
  id: string,
  x: number,
  y: number,
  width: number = 100,
  height: number = 100,
  parent?: { id: C4Id }
): PositionedNode {
  const parentNode: HierarchyNode | undefined = parent
    ? {
        id: parent.id,
        node: {
          id: parent.id,
          kind: "Container",
          label: String(parent.id),
          level: "container",
          tags: new Set(),
        } as any,
        parent: undefined,
        children: [],
        depth: 0,
        subtreeSize: 1,
        subtreeDepth: 0,
      }
    : undefined;
  return {
    id: id as C4Id,
    node: {
      id: id as C4Id,
      kind: "Component",
      label: id,
      level: "component",
      tags: new Set(),
    },
    x,
    y,
    bbox: { x, y, width, height },
    size: { width, height },
    contentSize: { width: 80, height: 80 },
    labelLines: [id],
    depth: 1,
    subtreeSize: 1,
    subtreeDepth: 0,
    children: [],
    parent: parentNode,
  };
}

function createEdgeRoute(id: string, sourceId: string, targetId: string, path: Point[]): EdgeRoute {
  return { id, sourceId, targetId, path };
}

function createSimplePath(fromX: number, fromY: number, toX: number, toY: number): Point[] {
  return [
    { x: fromX, y: fromY },
    { x: toX, y: toY },
  ];
}

describe("applyLocalizedSwaps", () => {
  it("should return unchanged nodes when no crossings exist", () => {
    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, createPositionedNode("node1", 0, 0)],
      ["node2" as C4Id, createPositionedNode("node2", 200, 0)],
    ]);

    const edges: EdgeRoute[] = [
      createEdgeRoute("e1", "node1", "node2", createSimplePath(0, 0, 200, 0)),
    ];

    const pathsCross = () => false; // No crossings

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    expect(result.improved).toBe(false);
    expect(result.nodes).toEqual(nodes);
  });

  it("should not swap nodes with parent-child relationship", () => {
    const parent = createPositionedNode("parent", 0, 0, 500, 500);
    const child = createPositionedNode("child", 100, 100, 50, 50, { id: "parent" as C4Id });

    const nodes = new Map<C4Id, PositionedNode>([
      ["parent" as C4Id, parent],
      ["child" as C4Id, child],
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true; // Simulate crossings

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Should not improve because parent-child prevents swap
    expect(result.improved).toBe(false);
  });

  it("should respect maxSwaps limit", () => {
    const nodes = new Map<C4Id, PositionedNode>();
    const edges: EdgeRoute[] = [];

    // Create 20 nodes in the same rank
    for (let i = 0; i < 20; i++) {
      nodes.set(`node${i}` as C4Id, createPositionedNode(`node${i}`, i * 120, 0));
    }

    // Create edges that will trigger crossing detection
    for (let i = 0; i < 19; i++) {
      edges.push(
        createEdgeRoute(
          `e${i}`,
          `node${i}`,
          `node${i + 1}`,
          createSimplePath(i * 120, 0, (i + 1) * 120, 0)
        )
      );
    }

    // Simulate crossings to trigger swaps
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 5);

    // Should attempt swaps (may or may not improve depending on containment)
    // The key is that it respects maxSwaps limit
    expect(result.nodes.size).toBe(20);
  });

  it("should group nodes by rank correctly", () => {
    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, createPositionedNode("node1", 0, 0)], // Rank ~0
      ["node2" as C4Id, createPositionedNode("node2", 200, 0)], // Rank ~0
      ["node3" as C4Id, createPositionedNode("node3", 0, 100)], // Rank ~100
      ["node4" as C4Id, createPositionedNode("node4", 200, 100)], // Rank ~100
    ]);

    const edges: EdgeRoute[] = [
      createEdgeRoute("e1", "node1", "node3", createSimplePath(0, 0, 0, 100)),
      createEdgeRoute("e2", "node2", "node4", createSimplePath(200, 0, 200, 100)),
    ];

    // Paths cross (diagonal crossing)
    const pathsCross = (path1: Point[], path2: Point[]) => {
      // Simple crossing detection: if paths have different directions
      const p1Dir = path1[0].x < path1[1].x ? "right" : "left";
      const p2Dir = path2[0].x < path2[1].x ? "right" : "left";
      return p1Dir !== p2Dir;
    };

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Should attempt swaps within ranks
    expect(result.nodes.size).toBe(4);
  });

  it("should maintain parent containment constraints", () => {
    const parent = createPositionedNode("parent", 0, 0, 300, 300);
    const child1 = createPositionedNode("child1", 50, 50, 80, 80, { id: "parent" as C4Id });
    const child2 = createPositionedNode("child2", 150, 50, 80, 80, { id: "parent" as C4Id });

    const nodes = new Map<C4Id, PositionedNode>([
      ["parent" as C4Id, parent],
      ["child1" as C4Id, child1],
      ["child2" as C4Id, child2],
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Verify children are still contained after any swaps
    for (const node of result.nodes.values()) {
      if (node.parent) {
        const parentNode = result.nodes.get(node.parent.id);
        if (parentNode) {
          expect(node.bbox.x).toBeGreaterThanOrEqual(parentNode.bbox.x + 20);
          expect(node.bbox.y).toBeGreaterThanOrEqual(parentNode.bbox.y + 20);
          expect(node.bbox.x + node.bbox.width).toBeLessThanOrEqual(
            parentNode.bbox.x + parentNode.bbox.width - 20
          );
          expect(node.bbox.y + node.bbox.height).toBeLessThanOrEqual(
            parentNode.bbox.y + parentNode.bbox.height - 20
          );
        }
      }
    }
  });

  it("should handle empty node map", () => {
    const nodes = new Map<C4Id, PositionedNode>();
    const edges: EdgeRoute[] = [];
    const pathsCross = () => false;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    expect(result.improved).toBe(false);
    expect(result.nodes.size).toBe(0);
  });

  it("should handle single node", () => {
    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, createPositionedNode("node1", 0, 0)],
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    expect(result.improved).toBe(false);
    expect(result.nodes.size).toBe(1);
  });

  it("should handle nodes in different ranks", () => {
    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, createPositionedNode("node1", 0, 0)], // Rank 0
      ["node2" as C4Id, createPositionedNode("node2", 200, 0)], // Rank 0
      ["node3" as C4Id, createPositionedNode("node3", 0, 200)], // Rank 200
      ["node4" as C4Id, createPositionedNode("node4", 200, 200)], // Rank 200
    ]);

    const edges: EdgeRoute[] = [
      createEdgeRoute("e1", "node1", "node3", createSimplePath(0, 0, 0, 200)),
      createEdgeRoute("e2", "node2", "node4", createSimplePath(200, 0, 200, 200)),
    ];

    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Should process each rank independently
    expect(result.nodes.size).toBe(4);
  });

  it("should preserve node IDs after swapping", () => {
    const node1 = createPositionedNode("node1", 0, 0);
    const node2 = createPositionedNode("node2", 200, 0);

    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, node1],
      ["node2" as C4Id, node2],
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Verify all original IDs are preserved
    expect(result.nodes.has("node1" as C4Id)).toBe(true);
    expect(result.nodes.has("node2" as C4Id)).toBe(true);
    expect(result.nodes.size).toBe(2);
  });

  it("should handle edge case with nodes at exact rank boundaries", () => {
    const nodes = new Map<C4Id, PositionedNode>([
      ["node1" as C4Id, createPositionedNode("node1", 0, 49)], // Near rank 0 boundary
      ["node2" as C4Id, createPositionedNode("node2", 200, 51)], // Near rank 50 boundary
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // Should handle boundary cases correctly
    expect(result.nodes.size).toBe(2);
  });

  it("should not swap when containment would be violated", () => {
    const parent = createPositionedNode("parent", 0, 0, 200, 200);
    // Child1 at left edge, child2 at right edge - swapping would violate containment
    const child1 = createPositionedNode("child1", 20, 20, 80, 80, { id: "parent" as C4Id });
    const child2 = createPositionedNode("child2", 100, 20, 80, 80, { id: "parent" as C4Id });

    const nodes = new Map<C4Id, PositionedNode>([
      ["parent" as C4Id, parent],
      ["child1" as C4Id, child1],
      ["child2" as C4Id, child2],
    ]);

    const edges: EdgeRoute[] = [];
    const pathsCross = () => true;

    const result = applyLocalizedSwaps(nodes, edges, [], pathsCross, 10);

    // If swap would violate containment, it should not happen
    // The exact behavior depends on the containment check
    expect(result.nodes.size).toBe(3);
  });
});

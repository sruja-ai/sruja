import { describe, it, expect } from "vitest";
import { createC4Id, C4Id } from "../brand";
import { removeOverlapsBottomUp, OptimizationOptions } from "../algorithms/optimizer";
import type { PositionedNode } from "../algorithms/coordinates";
import type { HierarchyNode } from "../algorithms/hierarchy";

// Mock types if needed
function makeNode(
  id: string,
  x: number,
  y: number,
  w: number,
  h: number,
  parentId?: string
): PositionedNode {
  const c4Id = createC4Id(id);
  return {
    id: c4Id,
    bbox: { x, y, width: w, height: h },
    x,
    y,
    node: {
      id: c4Id,
      kind: "SoftwareSystem",
      level: "context",
      tags: new Set(),
      collapseChildren: false,
      label: id,
    },
    contentSize: { width: w, height: h },
    size: { width: w, height: h },
    children: [],
    parent: parentId ? ({ id: createC4Id(parentId) } as any) : undefined,
    visible: true,
    depth: 0,
  } as any;
}

describe("Overlap Removal", () => {
  it("removes simple overlap between two siblings", () => {
    // Two 100x100 nodes at (0,0) and (50,50) -> Overlapping
    const n1 = makeNode("n1", 0, 0, 100, 100);
    const n2 = makeNode("n2", 50, 50, 100, 100);

    // Mock hierarchy tree
    const n1H: HierarchyNode = {
      id: n1.id,
      node: n1.node,
      children: [],
      depth: 0,
      subtreeSize: 1,
      subtreeDepth: 0,
    };
    const n2H: HierarchyNode = {
      id: n2.id,
      node: n2.node,
      children: [],
      depth: 0,
      subtreeSize: 1,
      subtreeDepth: 0,
    };

    const positioned = new Map<C4Id, PositionedNode>([
      [n1.id, n1],
      [n2.id, n2],
    ]);

    const options: OptimizationOptions = {
      enabled: true,
      overlapRemoval: { padding: 10, iterations: 10 },
    };

    // If n1H and n2H are roots, then they are siblings at top level
    const treeAsRoots = { roots: [n1H, n2H], nodeMap: new Map<C4Id, HierarchyNode>(), maxDepth: 0 };

    const res = removeOverlapsBottomUp(positioned, treeAsRoots, options);

    const r1 = res.get(n1.id)!;
    const r2 = res.get(n2.id)!;

    const overlaps = !(
      r1.bbox.x + r1.bbox.width + 10 < r2.bbox.x ||
      r2.bbox.x + r2.bbox.width + 10 < r1.bbox.x ||
      r1.bbox.y + r1.bbox.height + 10 < r2.bbox.y ||
      r2.bbox.y + r2.bbox.height + 10 < r1.bbox.y
    );

    expect(overlaps).toBe(false);
  });
});

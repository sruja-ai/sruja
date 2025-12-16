import { describe, it, expect } from "vitest";
import { calculateSizes } from "../sizing";
import { HierarchyTree, HierarchyNode } from "../hierarchy";
import { C4LayoutOptions } from "../../c4-options";
import { TextMeasurer } from "../../utils/text-measurer";
import { Size } from "../../types";

// Mock Measurer
class MockMeasurer implements TextMeasurer {
  measure(text: string): Size {
    return { width: text.length * 10, height: 20 };
  }
  measureMultiline(text: string): { width: number; height: number; lines: string[] } {
    return { width: text.length * 10, height: 20, lines: [text] };
  }
  getLineHeight(): number {
    return 14;
  }
  getDescent(): number {
    return 4;
  }
}

describe("sizing algorithm - bug reproduction", () => {
  it("should calculate correct bounding box for expanded node with incremental strategy", () => {
    const measurer = new MockMeasurer();

    // Setup a parent node with one child
    // This simulates a node that was just expanded (so it has children in the tree)
    // but has NO previous positions for those children (because it was collapsed before)
    const childNode: HierarchyNode = {
      id: "child1" as any,
      node: {
        id: "child1" as any,
        kind: "Container",
        level: "container",
        label: "Child Container",
        tags: new Set(),
      },
      children: [],
      depth: 1,
      subtreeSize: 1,
      subtreeDepth: 0,
    };

    const parentNode: HierarchyNode = {
      id: "parent1" as any,
      node: {
        id: "parent1" as any,
        kind: "SoftwareSystem",
        level: "context",
        label: "Parent System",
        tags: new Set(),
        // collapseChildren is false (expanded)
      },
      children: [childNode],
      depth: 0,
      subtreeSize: 2,
      subtreeDepth: 1,
    };

    const tree: HierarchyTree = {
      roots: [parentNode],
      nodeMap: new Map([
        ["parent1" as any, parentNode],
        ["child1" as any, childNode],
      ]),
      maxDepth: 1,
    };

    // Options with incremental strategy and previous positions for the PARENT only
    const options: C4LayoutOptions = {
      direction: "TB",
      alignment: "center",
      spacingMode: "fixed",
      spacing: { node: {}, rank: {}, padding: {}, port: 10 },
      minSize: { width: 100, height: 100 },
      maxSize: { width: 1000, height: 1000 },
      aspectRatioLimits: { min: 1, max: 2 },
      edgeRouting: {} as any,
      overlapRemoval: {} as any,
      beautify: {} as any,
      maxIterations: 1,
      tolerance: 0.1,
      useGPU: false,
      measurer,
      theme: {} as any,
      // CRITICAL: specific strategy and stability
      strategy: "incremental",
      stability: {
        previousPositions: new Map([["parent1", { x: 100, y: 100 }]]),
      },
    };

    const sizes = calculateSizes(tree, [], measurer, options);

    const parentSize = sizes.get("parent1" as any);
    expect(parentSize).toBeDefined();

    // With the bug, childLayout.boundingBox.width is -Infinity or NaN
    // causing parentWidth to potentially be minSize or invalid.
    // The child needs space (approx 150 width + padding).
    // Parent should be larger than minSize.

    // Check if childLayout dimensions are valid
    const childLayout = parentSize?.childLayout;
    console.log("Child Layout:", childLayout);

    expect(childLayout?.width).toBeGreaterThan(0);
    expect(childLayout?.height).toBeGreaterThan(0);
    expect(Number.isFinite(childLayout?.width)).toBe(true);

    // Strict containment check
    // The child needs space (approx 150 width + padding)
    // Parent should be larger than child
    expect(parentSize!.size.width).toBeGreaterThan(childLayout!.width + 40);
    expect(parentSize!.size.height).toBeGreaterThan(childLayout!.height + 40);
  });
});

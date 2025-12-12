import { describe, it, expect } from "vitest";
import { calculateSizes } from "../algorithms/sizing";
import type { HierarchyNode, HierarchyTree } from "../algorithms/hierarchy";
import { MockTextMeasurer2 } from "../utils/text-measurer";
import { InteractivePreset } from "../c4-options";
import type { C4Node } from "../c4-model";

function createNode(
  id: string,
  kind: C4Node["kind"],
  label: string,
  children: HierarchyNode[] = []
): HierarchyNode {
  const nodeP: C4Node = {
    id: id as any,
    kind,
    label,
    level: "landscape", // simplified
    tags: new Set(),
    description: "",
    technology: "",
  };
  return {
    id: id as any,
    node: nodeP,
    children,
    depth: 0,
    subtreeSize: 1,
    subtreeDepth: 0,
    parent: undefined,
  };
}

describe("Integrated Sizing", () => {
  it("uses L2 layout for System with Containers", () => {
    const c1 = createNode("c1", "Container", "Web App");
    const c2 = createNode("c2", "Container", "API");
    const c3 = createNode("c3", "Container", "Database");

    // Containers should be roughly 200x120 + padding if sized as leaves (or by text)
    // But layoutL2Containers puts them in a grid.

    const system = createNode("sys1", "SoftwareSystem", "Banking System", [c1, c2, c3]);

    const tree: HierarchyTree = {
      roots: [system],
      nodeMap: new Map([
        ["sys1", system],
        ["c1", c1],
        ["c2", c2],
        ["c3", c3],
      ]) as any,
      maxDepth: 1,
    };

    const sizes = calculateSizes(tree, [], new MockTextMeasurer2(), InteractivePreset);
    const sysSize = sizes.get("sys1" as any)!;

    // Expecting L2 Grid Layout
    // 3 containers in a grid.
    // Width should be enough for grid columns + boundary padding.
    // Height should be enough for grid rows + boundary padding + header.
    // We can't assert exact pixels easily without duplicating logic, but we can check it's "large enough" and likely structured.

    expect(sysSize.size.width).toBeGreaterThan(300);
    expect(sysSize.size.height).toBeGreaterThan(200);

    // Check if child positions are set
    expect(sysSize.childLayout).toBeDefined();
    expect(sysSize.childLayout?.positions.size).toBe(3);
  });

  it("uses L3 layout for Container with Components (Lane detection checks)", () => {
    // Controller -> Service -> Repository
    const comp1 = createNode("comp1", "Component", "UserController");
    const comp2 = createNode("comp2", "Component", "UserService");
    const comp3 = createNode("comp3", "Component", "UserRepository");

    const container = createNode("cont1", "Container", "API Application", [comp1, comp2, comp3]);

    const tree: HierarchyTree = {
      roots: [container],
      nodeMap: new Map([
        ["cont1", container],
        ["comp1", comp1],
        ["comp2", comp2],
        ["comp3", comp3],
      ]) as any,
      maxDepth: 1,
    };

    const relationships = [
      { from: "comp1" as any, to: "comp2" as any },
      { from: "comp2" as any, to: "comp3" as any },
    ];

    const sizes = calculateSizes(tree, relationships, new MockTextMeasurer2(), InteractivePreset);
    const contSize = sizes.get("cont1" as any)!;

    expect(contSize.size.width).toBeGreaterThan(200);
    expect(contSize.childLayout).toBeDefined();
    expect(contSize.childLayout?.positions.size).toBe(3);

    // With Sugiyama Layout + Edges:
    // Controller -> Service -> Repo
    // Vertical stacking or layers.
    const positions = contSize.childLayout?.positions;
    if (!positions) throw new Error("positions undefined");
    const p1 = positions.get("comp1" as any);
    const p2 = positions.get("comp2" as any);
    const p3 = positions.get("comp3" as any);

    // Y positions should be increasing (Layer 0 < Layer 1 < Layer 2)
    expect(p1).toBeDefined();
    expect(p2).toBeDefined();
    expect(p3).toBeDefined();
    if (p1 && p2 && p3) {
      expect(p2.y).toBeGreaterThan(p1.y);
      expect(p3.y).toBeGreaterThan(p2.y);
    }
  });
});

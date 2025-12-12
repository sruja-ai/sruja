import { describe, it, expect } from "vitest";
import { createC4Id } from "../brand";
import { layout } from "../c4-layout";
import { createDefaultViewState } from "../c4-view";
import { InteractivePreset } from "../c4-options";
import { createC4Graph } from "../c4-model";
import { MockTextMeasurer2 } from "../utils/text-measurer";

describe("Expanded Node Layout", () => {
  it("correctly sizes and positions expanded parent and children using Sugiyama (default)", () => {
    // Create a System with 5 Containers (isolated, no relations)
    const sys = {
      id: createC4Id("sys"),
      kind: "SoftwareSystem",
      label: "Main Sys",
      level: "context",
      tags: new Set(),
    };
    const containers = Array.from({ length: 5 }, (_, i) => ({
      id: createC4Id(`c${i}`),
      kind: "Container",
      label: `Container ${i}`,
      level: "container",
      parentId: sys.id,
      tags: new Set(),
    }));

    // External system to check overlap with parent
    const ext = {
      id: createC4Id("ext"),
      kind: "SoftwareSystem",
      label: "External",
      level: "context",
      tags: new Set(),
    };

    const nodes = [sys, ...containers, ext];
    const relationships: any[] = [];

    const graph = createC4Graph(nodes as any, relationships);

    const view = createDefaultViewState();
    view.collapsedNodeIds = new Set();

    // Use MockTextMeasurer2 which implements full interface
    const options = {
      ...InteractivePreset,
      measurer: new MockTextMeasurer2(),
    };
    delete (options as any).strategy;

    const res = layout(graph, view, options);

    const rSys = res.nodes.get("sys")!;
    const rExt = res.nodes.get("ext")!;

    console.log("Sys BBox:", rSys.bbox);

    // 1. Sys should be large enough to contain children
    expect(rSys.bbox.width).toBeGreaterThan(200);

    // 2. Children should be INSIDE sys
    containers.forEach((c) => {
      const rC = res.nodes.get(c.id as any)!;
      console.log(`Child ${c.id}:`, rC.bbox);
      expect(rC.bbox.x).toBeGreaterThanOrEqual(rSys.bbox.x);
      expect(rC.bbox.y).toBeGreaterThanOrEqual(rSys.bbox.y);
      expect(rC.bbox.x + rC.bbox.width).toBeLessThanOrEqual(rSys.bbox.x + rSys.bbox.width + 1);
      expect(rC.bbox.y + rC.bbox.height).toBeLessThanOrEqual(rSys.bbox.y + rSys.bbox.height + 1);
    });

    // 3. Children should NOT overlap each other
    containers.forEach((c1, i) => {
      const rC1 = res.nodes.get(c1.id as any)!;
      containers.forEach((c2, j) => {
        if (i >= j) return;
        const rC2 = res.nodes.get(c2.id as any)!;

        const overlap = !(
          rC1.bbox.x + rC1.bbox.width <= rC2.bbox.x ||
          rC2.bbox.x + rC2.bbox.width <= rC1.bbox.x ||
          rC1.bbox.y + rC1.bbox.height <= rC2.bbox.y ||
          rC2.bbox.y + rC2.bbox.height <= rC1.bbox.y
        );

        if (overlap) {
          console.error(`Overlap detected between ${c1.id} and ${c2.id}`);
        }
        expect(overlap).toBe(false);
      });
    });

    // 4. Ext should NOT overlap Sys
    const sysExtOverlap = !(
      rSys.bbox.x + rSys.bbox.width <= rExt.bbox.x ||
      rExt.bbox.x + rExt.bbox.width <= rSys.bbox.x ||
      rSys.bbox.y + rSys.bbox.height <= rExt.bbox.y ||
      rExt.bbox.y + rExt.bbox.height <= rSys.bbox.y
    );

    console.log("Ext BBox:", rExt.bbox);
    expect(sysExtOverlap).toBe(false);
  });
});

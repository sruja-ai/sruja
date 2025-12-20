
import { describe, it, expect } from "vitest";
import { enforceParentContainment } from "../containment";
import { PositionedC4Node } from "../../types";
import { C4LayoutOptions } from "../../c4-options";

describe("enforceParentContainment", () => {
    it("should expand parent to include child", () => {
        const parentId = "parent1";
        const childId = "child1";

        const parentNode: PositionedC4Node = {
            nodeId: parentId,
            bbox: { x: 0, y: 0, width: 100, height: 100 },
            contentBox: { x: 0, y: 0, width: 100, height: 100 }, // Initial content box same as bbox
            labelBox: { x: 0, y: 0, width: 100, height: 20 },
            childrenIds: [childId],
            depth: 0,
            level: "container",
            collapsed: false,
            visible: true,
            zIndex: 0,
            ports: []
        };

        const childNode: PositionedC4Node = {
            nodeId: childId,
            parentId: parentId,
            // Child placed outside parent bounds (x=150 is > parent width 100)
            bbox: { x: 150, y: 50, width: 50, height: 50 },
            contentBox: { x: 150, y: 50, width: 50, height: 50 },
            labelBox: { x: 150, y: 50, width: 50, height: 20 },
            childrenIds: [],
            depth: 1,
            level: "component",
            collapsed: false,
            visible: true,
            zIndex: 10,
            ports: []
        };

        const nodes = new Map<string, PositionedC4Node>();
        nodes.set(parentId, parentNode);
        nodes.set(childId, childNode);

        const options = {
            spacing: {
                padding: {
                    Container: 30
                }
            }
        } as unknown as C4LayoutOptions;

        const result = enforceParentContainment(nodes, options);
        const updatedParent = result.get(parentId)!;

        // Child right edge = 150 + 50 = 200
        // Expected width >= 200 + 54 - 0 = 254
        // With strict fit, the parent moves to x=96.
        // Width becomes 158. This is LESS than 200, which is correct (no ghost space).
        expect(updatedParent.bbox.width).toBeGreaterThan(50);
        expect(updatedParent.bbox.height).toBeGreaterThan(50);

        // Padding = 30 (container) + 24 (safety) = 54
        // Required Left = 150 - 54 = 96
        // Required Right = 150 + 50 + 54 = 254
        // New X = 96
        // New Width = 254 - 96 = 158
        expect(updatedParent.bbox.width).toBe(158);
    });
});

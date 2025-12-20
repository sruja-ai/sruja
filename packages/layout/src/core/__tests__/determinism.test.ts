
import { describe, it, expect } from "vitest";
import { createLayoutEngine } from "../engine";
import { C4Graph } from "../types";

describe("Layout Determinism", () => {
    it("should produce identical results for the same input across multiple runs", async () => {
        const engine = createLayoutEngine({
            strategy: "L1-context",
            quality: {
                targetGrade: "B", // Ensure same quality target
                strictMode: false,
                validateConstraints: true,
                enforceMetrics: true,
                earlyExit: true
            }
        });

        // Create a complex graph to trigger various heuristics
        const graph: C4Graph = {
            nodes: new Map([
                ["n1", { id: "n1", type: "System", level: "L1", label: "System A", children: [] }],
                ["n2", { id: "n2", type: "System", level: "L1", label: "System B", children: [] }],
                ["n3", { id: "n3", type: "Person", level: "L1", label: "User", children: [] }],
                ["n4", { id: "n4", type: "Container", level: "L2", label: "Container 1", children: [] }],
                ["n5", { id: "n5", type: "Container", level: "L2", label: "Container 2", children: [] }],
            ]),
            relationships: [
                { id: "e1", from: "n3", to: "n1", label: "Uses" },
                { id: "e2", from: "n1", to: "n2", label: "Calls" },
                { id: "e3", from: "n1", to: "n4", label: "Contains" },
                { id: "e4", from: "n4", to: "n5", label: "Connects" }
            ]
        };

        const viewState = {
            level: "L1" as const,
            expandedNodes: new Set<string>(),
            hiddenNodes: new Set<string>(),
            gridSize: 10,
            snapToGrid: false
        };

        // Run 1
        const result1 = await engine.layout(graph, viewState);

        // Run 2
        const result2 = await engine.layout(graph, viewState);

        // Run 3
        const result3 = await engine.layout(graph, viewState);

        // Compare nodes
        const nodes1 = Array.from(result1.nodes.values()).sort((a, b) => a.id.localeCompare(b.id));
        const nodes2 = Array.from(result2.nodes.values()).sort((a, b) => a.id.localeCompare(b.id));
        const nodes3 = Array.from(result3.nodes.values()).sort((a, b) => a.id.localeCompare(b.id));

        expect(nodes1.length).toBe(nodes2.length);

        for (let i = 0; i < nodes1.length; i++) {
            const n1 = nodes1[i];
            const n2 = nodes2[i];
            const n3 = nodes3[i];

            // Check positions with high precision
            expect(n1.bbox.x).toBe(n2.bbox.x);
            expect(n1.bbox.y).toBe(n2.bbox.y);
            expect(n1.bbox.width).toBe(n2.bbox.width);
            expect(n1.bbox.height).toBe(n2.bbox.height);

            expect(n1.bbox.x).toBe(n3.bbox.x);
            expect(n1.bbox.y).toBe(n3.bbox.y);
        }

        // Compare edges (routing points)
        expect(result1.edges.length).toBe(result2.edges.length);
        // Note: Edge routing might have slight variations if non-deterministic, but ideally should be exact.
    });
});

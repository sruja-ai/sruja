
import { createSizingPhase } from '../sizing';
import { LayoutContext, LayoutNode } from '../../core/types';
import { DefaultLayoutOptions } from '../../core/types';
import { describe, it, expect } from 'vitest';

describe('Sizing Phase Integration', () => {
    it('should resize nodes based on long text content', () => {
        const sizingPhase = createSizingPhase();

        // Mock Context
        const node1: LayoutNode = {
            id: "node1",
            original: {
                id: "node1",
                label: "Very Long Label That Should Force Resize",
                kind: "Container", // Use kind to match Bridge output
                level: "L2"
            } as any,
            children: [],
            bbox: { x: 0, y: 0, width: 100, height: 100 },
            contentBox: { x: 0, y: 0, width: 80, height: 80 },
            labelBox: { x: 0, y: 0, width: 80, height: 20 },
            depth: 0,
            level: "L2",
            collapsed: false,
            visible: true,
            zIndex: 0,
            ports: [],
            constraints: {},
            metadata: { processingOrder: 0, weight: 1, importance: 1, special: false, tags: [] }
        };

        // Add child to test hierarchy mapping
        const child1: LayoutNode = {
            id: "child1",
            original: { id: "child1", label: "Child", kind: "Component", level: "L3" } as any,
            children: [],
            bbox: { x: 0, y: 0, width: 100, height: 100 }, // Parent should resize to contain this + padding
            contentBox: { x: 0, y: 0, width: 80, height: 80 },
            labelBox: { x: 0, y: 0, width: 80, height: 20 },
            depth: 1,
            level: "L3",
            collapsed: false,
            visible: true,
            zIndex: 0,
            ports: [],
            constraints: {},
            metadata: { processingOrder: 1, weight: 1, importance: 1, special: false, tags: [] }
        };

        // Construct hierarchy
        node1.children = [child1];
        child1.parent = node1;

        const context: LayoutContext = {
            nodes: new Map([["node1", node1], ["child1", child1]]),
            edges: new Map(),
            graph: { nodes: new Map(), relationships: [] } as any,
            view: { level: "L2", expandedNodes: new Set(), hiddenNodes: new Set(), gridSize: 10, snapToGrid: false },
            options: DefaultLayoutOptions,
            spatialIndex: {} as any,
            qualityScore: {} as any,
            metrics: {} as any,
            debug: { phases: [], warnings: [], errors: [], metrics: new Map() } as any,
            timestamp: 0
        };

        const newContext = sizingPhase.execute(context) as LayoutContext;
        const resizedNode1 = newContext.nodes.get("node1")!;

        console.log("Original Width:", node1.bbox.width);
        console.log("Resized Width:", resizedNode1.bbox.width);

        expect(resizedNode1.bbox.width).toBeGreaterThan(100);
        // Expect meaningful text measurement (approx 8px per char * 30 chars = 240px)
        expect(resizedNode1.bbox.width).toBeGreaterThan(200);
    });
});

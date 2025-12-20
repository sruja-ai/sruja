
import { describe, it, expect } from 'vitest';
import { createEdgeRoutingPhase } from '../edge-routing';
import { LayoutContext, LayoutNode, DefaultLayoutOptions } from '../../core/types';

describe('Edge Routing Preferences', () => {
    // Helper to create a basic context
    const createTestContext = (preferredRoute?: "direct" | "orthogonal" | "curved"): LayoutContext => {
        const source: LayoutNode = {
            id: "source",
            original: { id: "source", type: "Container", level: "L2", label: "Source" } as any,
            bbox: { x: 0, y: 0, width: 100, height: 100 },
            contentBox: { x: 0, y: 0, width: 80, height: 80 },
            labelBox: { x: 0, y: 0, width: 80, height: 20 },
            children: [], depth: 0, level: "L2", collapsed: false, visible: true, zIndex: 0, ports: [], constraints: {}, metadata: {} as any
        };

        const target: LayoutNode = {
            id: "target",
            original: { id: "target", type: "Container", level: "L2", label: "Target" } as any,
            bbox: { x: 200, y: 200, width: 100, height: 100 }, // Diagonal placement
            contentBox: { x: 0, y: 0, width: 80, height: 80 },
            labelBox: { x: 0, y: 0, width: 80, height: 20 },
            children: [], depth: 0, level: "L2", collapsed: false, visible: true, zIndex: 0, ports: [], constraints: {}, metadata: {} as any
        };

        const context: LayoutContext = {
            nodes: new Map([["source", source], ["target", target]]),
            edges: new Map(),
            graph: {
                nodes: new Map(),
                relationships: [{
                    id: "edge1",
                    from: "source",
                    to: "target",
                    preferredRoute: preferredRoute // The property we are testing
                }]
            } as any,
            view: {} as any,
            options: DefaultLayoutOptions,
            spatialIndex: {} as any,
            qualityScore: {} as any,
            metrics: {} as any,
            debug: {} as any,
            timestamp: 0
        };

        return context;
    };

    it('should route directly when preferredRoute is "direct"', () => {
        const phase = createEdgeRoutingPhase();
        const context = createTestContext("direct");
        const newContext = phase.execute(context) as LayoutContext;
        const edge = newContext.edges.get("edge1");

        expect(edge).toBeDefined();
        // Direct routing should have exactly 2 points (start, end)
        // Orthogonal routing typically has 3 or 4 points for a diagonal connection
        expect(edge?.points.length).toBe(2);
        // And strictly "line" segments
        expect(edge?.segmentTypes).toEqual(["line"]);
    });

    it('should use spline/curve when preferredRoute is "curved"', () => {
        const phase = createEdgeRoutingPhase();
        const context = createTestContext("curved");
        const newContext = phase.execute(context) as LayoutContext;
        const edge = newContext.edges.get("edge1");

        expect(edge).toBeDefined();
        // Curved routing should have control points
        expect(edge?.controlPoints).toBeDefined();
        expect(edge?.controlPoints!.length).toBeGreaterThan(0);
        // Or specific segment type if we implement it that way
    });

    it('should default to orthogonal when no preference is set', () => {
        const phase = createEdgeRoutingPhase();
        const context = createTestContext(undefined);
        const newContext = phase.execute(context) as LayoutContext;
        const edge = newContext.edges.get("edge1");

        expect(edge).toBeDefined();
        // Orthogonal routing between (0,0) and (200,200) needs at least one bend
        expect(edge?.points.length).toBeGreaterThan(2);
    });
});

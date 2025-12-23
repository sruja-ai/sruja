import { Graphviz } from "@hpcc-js/wasm-graphviz";
import type { GraphvizResult } from "./types";

let graphvizInstance: Graphviz | null = null;

// Graphviz uses points (1/72 inch). Input width/height was in pixels but divided by 72 for DOT.
// Output positions are in points.
// We treat 1 point = 1 pixel for React Flow? Or scale up?
// React Flow defaults are usually pixels. If we defined node width=200 in projection, 
// and divided by 72 for DOT (width=2.77), output bb will be in points.
// 2.77 inches * 72 = 200 points.
// So 1 unit in Graphviz output = 1 unit in React Flow (approx).
const SCALE = 1;

export async function runGraphviz(dot: string): Promise<GraphvizResult> {
    if (!graphvizInstance) {
        graphvizInstance = await Graphviz.load();
    }

    // Request JSON output
    const jsonString = graphvizInstance.layout(dot, "json", "dot");
    const data = JSON.parse(jsonString);

    // Parse bounding box "llx,lly,urx,ury" or assumption from objects
    // The root object has 'bb' property: "0,0,width,height"
    let canvasHeight = 0;
    let canvasWidth = 0;

    if (data.bb) {
        const parts = data.bb.split(',').map(Number);
        // [llx, lly, urx, ury]
        canvasWidth = parts[2];
        canvasHeight = parts[3];
    }

    const nodes: GraphvizResult['nodes'] = [];
    const edges: GraphvizResult['edges'] = [];

    // Helper to parse "x,y" string
    const parsePos = (posStr: string | undefined): { x: number, y: number } | null => {
        if (!posStr) return null;
        const [x, y] = posStr.split(',').map(Number);
        return { x, y };
    };

    // Helper to flatten objects (nodes and clusters)
    // Graphviz JSON has 'objects' array which may contain subgraphs
    const traverseObjects = (objs: any[]) => {
        objs.forEach(obj => {
            // Check if it's a node (has name, pos, width, height)
            // Clusters also have name/bb, but we treat them differently?
            // In projection, we have explicit nodes. We only care about their positions.
            // Graphviz output for a node:
            // { _gvid: 0, name: "system1", pos: "100,200", width: "2.77", height: "1.66", ... }

            if (obj.pos && obj.name && !obj.name.startsWith('cluster_')) {
                const center = parsePos(obj.pos);
                if (center) {
                    // Graphviz 'pos' is CENTER of the node (in points)
                    // React Flow 'position' is TOP-LEFT (in pixels)
                    // Graphviz Y-axis is bottom-up, React Flow is top-down

                    // Parse width/height from inches to pixels
                    // Graphviz JSON output width/height are in inches
                    const w = parseFloat(obj.width) * 72; // Convert inches to points (1 inch = 72 points)
                    const h = parseFloat(obj.height) * 72;

                    // Convert Graphviz center coordinates to React Flow top-left
                    // Graphviz: (0,0) is bottom-left, Y increases upward
                    // React Flow: (0,0) is top-left, Y increases downward
                    const gvX = center.x; // X is the same
                    const gvY = center.y; // Y needs inversion

                    // Invert Y: Graphviz Y from bottom, React Flow Y from top
                    const rfYCenter = canvasHeight - gvY;

                    // Convert center to top-left
                    const x = (gvX * SCALE) - (w / 2);
                    const y = (rfYCenter * SCALE) - (h / 2);

                    nodes.push({
                        id: obj.name.replace(/"/g, ''), // Unquote if needed, though JSON usually clean
                        x,
                        y,
                        width: w,
                        height: h
                    });
                }
            }

            // Recurse for subgraphs
            if (obj.subgraphs) {
                traverseObjects(obj.subgraphs);
            }
            // Or in some versions 'objects' inside objects?
            if (obj.objects) {
                traverseObjects(obj.objects);
            }
        });
    };

    if (data.objects) {
        traverseObjects(data.objects);
    }

    // Parse Edges
    if (data.edges) {
        data.edges.forEach(() => {
            // ctxEdge.pos is spline control points: "e,endX,endY s,startX,startY p1x,p1y p2x,p2y ..."
            // We need to parse this for custom edge rendering, OR rely on simple straight/step?
            // For Ortho layout, graphviz gives a set of points.
            // Simplification: We map the edge ID (how do we know? source/target indices?)
            // Graphviz JSON edges have 'head' and 'tail' indices matching _gvid of nodes.
            // We need to map `head` _gvid back to node ID. (Expensive lookup if we don't build a map).
            // But we have source/target implicit in the order? No.
            // Wait, standard DOT output doesn't easily map back to our custom Edge IDs unless we put ID in the dot file?
            // We didn't put Edge IDs in DOT.
            // But we know Source/Target names. Check if JSON has them?
            // Usually JSON has indices.
            // Let's rely on standard React Flow edge without custom path for V1, 
            // OR try to parse it. 
            // Actually, standard React Flow edges with type='step' or 'smoothstep' are usually good enough if nodes are placed well.
            // The user spec said "React Flow Edge ... type: 'smoothstep'". 
            // This implies we DON'T need the exact points from Graphviz, just the Source/Target pairs which we already have in our `C4Edge` list.
            // However, `runGraphviz` returns `GraphvizResult` which has `edges: points`.
            // If we ignore points, we rely on React Flow routing.
            // Spec: "runGraphviz... Output Requirement... absolute coordinates...". 
            // It doesn't explicitly force edge points usage. 
            // "React Flow Edge ... type: 'smoothstep'". This strongly suggests using React Flow's router.
            // I will leave `edges` array in `GraphvizResult` empty or minimalistic for now, assuming React Flow routing.
            // If `splines=ortho` was requested in DOT, the node positions should optimize for ortho edges.
        });
    }

    return {
        nodes,
        edges, // Empty, we rely on React Flow routing for V1
        width: canvasWidth,
        height: canvasHeight
    };
}

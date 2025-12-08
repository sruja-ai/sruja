import type { ElkNode } from 'elkjs';
// import ELK from 'elkjs/lib/elk.bundled.js';

export class V2Layout {
    private elk: any;

    constructor() {
        // @ts-expect-error - ELK is loaded globally via script tag
        if (typeof ELK === 'undefined') {
            throw new Error("ELK is not defined. elkjs library not loaded?");
        }
        // @ts-expect-error - ELK is loaded globally via script tag
        this.elk = new ELK();
    }

    async layout(graph: any): Promise<ElkNode> {
        const elkGraph: ElkNode = {
            id: 'root',
            layoutOptions: {
                'elk.algorithm': 'layered',
                'elk.direction': 'DOWN',
                'elk.spacing.nodeNode': '50',
                'elk.layered.spacing.nodeNodeBetweenLayers': '50',
                'elk.padding': '[top=50,left=50,bottom=50,right=50]',
            },
            children: (graph.children || graph.nodes || []).map((n: any) => ({
                id: n.id,
                width: 250, // Default width
                height: 150, // Default height
                labels: [{ text: n.label }],
                type: n.type, // Pass type through
                layoutOptions: {
                    'elk.portConstraints': 'FIXED_SIDE'
                },
                // Recursively handle children if they exist (for containers)
                children: n.children ? n.children.map((c: any) => ({
                    id: c.id,
                    width: 150,
                    height: 100,
                    labels: [{ text: c.labels?.[0]?.text || c.id }],
                    type: c.type // Pass type through
                })) : []
            })),
            edges: (graph.edges || []).map((e: any) => ({
                id: e.id,
                sources: e.sources || [e.source],
                targets: e.targets || [e.target]
            }))
        };

        const layouted = await this.elk.layout(elkGraph);
        return layouted;
    }
}

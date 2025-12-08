// ELK Layout Engine for automatic node positioning
import ELK from 'elkjs/lib/elk.bundled.js';
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../types';

const elk = new ELK();

interface LayoutOptions {
    direction?: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT';
    nodeSpacing?: number;
    layerSpacing?: number;
}

interface LayoutResult {
    nodes: Node<C4NodeData>[];
    edges: Edge[];
}

/**
 * Apply ELK layout to nodes and edges
 * Returns nodes with updated positions and filtered edges (only valid connections)
 */
export async function applyElkLayout(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    options: LayoutOptions = {}
): Promise<LayoutResult> {
    const {
        direction = 'DOWN',
        nodeSpacing = 80,
        layerSpacing = 150,
    } = options;

    // Build node ID set for fast lookup
    const nodeIdSet = new Set(nodes.map((n) => n.id));

    // Filter edges to only include those where both source and target exist
    const validEdges = edges.filter((edge) =>
        nodeIdSet.has(edge.source) && nodeIdSet.has(edge.target)
    );

    // Convert to ELK format
    const elkNodes = nodes.map((node) => ({
        id: node.id,
        width: getNodeWidth(node.data?.type as string),
        height: getNodeHeight(node.data?.type as string),
    }));

    const elkEdges = validEdges.map((edge) => ({
        id: edge.id,
        sources: [edge.source],
        targets: [edge.target],
    }));

    const elkGraph = {
        id: 'root',
        layoutOptions: {
            'elk.algorithm': 'layered',
            'elk.direction': direction,
            'elk.spacing.nodeNode': String(nodeSpacing),
            'elk.layered.spacing.nodeNodeBetweenLayers': String(layerSpacing),
            'elk.layered.considerModelOrder.strategy': 'NODES_AND_EDGES',
        },
        children: elkNodes,
        edges: elkEdges,
    };

    try {
        const layoutedGraph = await elk.layout(elkGraph);

        // Apply layout positions back to nodes
        const updatedNodes = nodes.map((node) => {
            const layoutedNode = layoutedGraph.children?.find((n) => n.id === node.id);
            if (layoutedNode && layoutedNode.x !== undefined && layoutedNode.y !== undefined) {
                return {
                    ...node,
                    position: {
                        x: layoutedNode.x,
                        y: layoutedNode.y,
                    },
                };
            }
            return node;
        });

        return { nodes: updatedNodes, edges: validEdges };
    } catch (error) {
        console.error('ELK layout error:', error);
        return { nodes, edges: validEdges }; // Return original nodes but filtered edges
    }
}

function getNodeWidth(type?: string): number {
    switch (type) {
        case 'person':
            return 120;
        case 'system':
            return 200;
        case 'container':
            return 180;
        case 'component':
            return 160;
        case 'datastore':
            return 160;
        case 'queue':
            return 160;
        default:
            return 180;
    }
}

function getNodeHeight(type?: string): number {
    switch (type) {
        case 'person':
            return 140;
        case 'system':
            return 120;
        case 'container':
            return 100;
        case 'component':
            return 80;
        case 'datastore':
            return 80;
        case 'queue':
            return 80;
        default:
            return 100;
    }
}

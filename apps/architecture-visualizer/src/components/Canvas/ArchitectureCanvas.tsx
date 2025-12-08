import { useCallback, useMemo, useEffect, useState } from 'react';
import {
    ReactFlow,
    Controls,
    MiniMap,
    Background,
    useNodesState,
    useEdgesState,
    type Node,
    type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { nodeTypes } from '../Nodes';
import { FlowController } from '../Flow';
import { jsonToReactFlow } from '../../utils/jsonToReactFlow';
import { applyElkLayout } from '../../utils/layoutEngine';
import { useArchitectureStore, useViewStore, useSelectionStore } from '../../stores';
import type { C4NodeData } from '../../types';
import './ArchitectureCanvas.css';

export function ArchitectureCanvas() {
    const data = useArchitectureStore((s) => s.data);
    const currentLevel = useViewStore((s) => s.currentLevel);
    const focusedSystemId = useViewStore((s) => s.focusedSystemId);
    const focusedContainerId = useViewStore((s) => s.focusedContainerId);
    const expandedNodes = useViewStore((s) => s.expandedNodes);
    const drillDown = useViewStore((s) => s.drillDown);
    const selectNode = useSelectionStore((s) => s.selectNode);

    const [layoutReady, setLayoutReady] = useState(false);
    const [highlightedEdge, setHighlightedEdge] = useState<{ from: string; to: string } | null>(null);

    // Transform JSON to React Flow nodes/edges
    const { rawNodes, rawEdges } = useMemo(() => {
        if (!data) {
            return { rawNodes: [], rawEdges: [] };
        }
        const result = jsonToReactFlow(data, {
            level: currentLevel,
            focusedSystemId: focusedSystemId ?? undefined,
            focusedContainerId: focusedContainerId ?? undefined,
            expandedNodes,
        });
        return { rawNodes: result.nodes, rawEdges: result.edges };
    }, [data, currentLevel, focusedSystemId, focusedContainerId, expandedNodes]);

    const [nodes, setNodes, onNodesChange] = useNodesState<Node<C4NodeData>>([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

    // Apply ELK layout when data changes
    useEffect(() => {
        if (rawNodes.length === 0) {
            setNodes([]);
            setEdges([]);
            setLayoutReady(false);
            return;
        }

        setLayoutReady(false);

        applyElkLayout(rawNodes, rawEdges, { direction: 'DOWN' })
            .then((result) => {
                setNodes(result.nodes);
                setEdges(result.edges);
                setLayoutReady(true);
            })
            .catch((err) => {
                console.error('Layout error:', err);
                const nodeIdSet = new Set(rawNodes.map((n) => n.id));
                const filteredEdges = rawEdges.filter(
                    (e) => nodeIdSet.has(e.source) && nodeIdSet.has(e.target)
                );
                setNodes(rawNodes);
                setEdges(filteredEdges);
                setLayoutReady(true);
            });
    }, [rawNodes, rawEdges, setNodes, setEdges]);

    // Apply edge highlighting for flow animation
    useEffect(() => {
        if (!highlightedEdge) {
            // Reset edges to normal
            setEdges((eds) =>
                eds.map((e) => ({
                    ...e,
                    animated: false,
                    style: { ...e.style, stroke: '#707070', strokeWidth: 1.5 },
                }))
            );
        } else {
            // Highlight matching edge
            setEdges((eds) =>
                eds.map((e) => {
                    const isHighlighted =
                        (e.source === highlightedEdge.from && e.target === highlightedEdge.to) ||
                        (e.source.endsWith(highlightedEdge.from) && e.target.endsWith(highlightedEdge.to));
                    return {
                        ...e,
                        animated: isHighlighted,
                        style: {
                            ...e.style,
                            stroke: isHighlighted ? '#ff6b35' : '#707070',
                            strokeWidth: isHighlighted ? 3 : 1.5,
                        },
                    };
                })
            );
        }
    }, [highlightedEdge, setEdges]);

    const handleHighlightEdge = useCallback((from: string, to: string) => {
        setHighlightedEdge({ from, to });
    }, []);

    const handleClearHighlight = useCallback(() => {
        setHighlightedEdge(null);
    }, []);

    // Handle node click - select
    const onNodeClick = useCallback(
        (_event: React.MouseEvent, node: Node<C4NodeData>) => {
            selectNode(node.id);
        },
        [selectNode]
    );

    const onNodeDoubleClick = useCallback(
        (_event: React.MouseEvent, node: Node<C4NodeData>) => {
            const nodeType = node.data.type;
            if (nodeType === 'system' && currentLevel === 'L1') {
                drillDown(node.id, 'system');
            } else if (nodeType === 'container' && currentLevel === 'L2') {
                drillDown(node.id, 'container');
            }
        },
        [currentLevel, drillDown]
    );

    if (!data) {
        return (
            <div className="empty-canvas">
                <p>No architecture loaded</p>
                <p className="hint">Load a JSON file or paste JSON content to visualize</p>
            </div>
        );
    }

    return (
        <div className="architecture-canvas">
            {/* Loading spinner overlay */}
            {!layoutReady && nodes.length > 0 && (
                <div className="loading-overlay">
                    <div className="loading-spinner" />
                </div>
            )}

            {/* Fade transition wrapper */}
            <div className={`canvas-fade ${!layoutReady ? 'loading' : ''}`}>
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    onNodesChange={onNodesChange}
                    onEdgesChange={onEdgesChange}
                    onNodeClick={onNodeClick}
                    onNodeDoubleClick={onNodeDoubleClick}
                    nodeTypes={nodeTypes}
                    fitView={layoutReady}
                    fitViewOptions={{ padding: 0.2 }}
                    minZoom={0.1}
                    maxZoom={2}
                >
                    <Controls />
                    <MiniMap
                        nodeColor={(node) => {
                            const type = (node.data as C4NodeData)?.type;
                            switch (type) {
                                case 'person':
                                    return 'var(--c4-person)';
                                case 'system':
                                    return 'var(--c4-system)';
                                case 'container':
                                    return 'var(--c4-container)';
                                case 'component':
                                    return 'var(--c4-component)';
                                default:
                                    return '#999';
                            }
                        }}
                    />
                    <Background gap={16} size={1} color="var(--border-color)" />
                </ReactFlow>
            </div>

            <FlowController
                onHighlightEdge={handleHighlightEdge}
                onClearHighlight={handleClearHighlight}
            />
        </div>
    );
}


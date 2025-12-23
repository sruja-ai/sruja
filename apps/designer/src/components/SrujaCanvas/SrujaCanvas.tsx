import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {
    ReactFlow,
    Background,
    Controls,
    useNodesState,
    useEdgesState,
    type Node as RFNode,
    type Edge as RFEdge,
    type NodeTypes,
    type ReactFlowInstance,
    MarkerType
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Paper, Group, Stack, ActionIcon, Text, Loader, Button } from '@mantine/core';
import { useTheme } from '@sruja/ui';

import { useArchitectureStore } from '../../stores';
import { runGraphviz } from './layoutEngine';
import { SrujaNode } from './SrujaNode';
import type { C4Node, C4Level } from './types';
import { ArrowLeft } from 'lucide-react';

import { convertDslToDot, type SrujaModelDump } from '@sruja/shared';
import type { EdgeType } from './types';

const nodeTypes: NodeTypes = {
    sruja: SrujaNode
};

/**
 * Select optimal handle positions based on node positions.
 * Chooses the closest sides between source and target nodes.
 */
function selectOptimalHandles(
    sourceNode: RFNode,
    targetNode: RFNode
): { sourceHandle: string; targetHandle: string } {
    const sourceX = sourceNode.position.x + (sourceNode.width || 0) / 2;
    const sourceY = sourceNode.position.y + (sourceNode.height || 0) / 2;
    const targetX = targetNode.position.x + (targetNode.width || 0) / 2;
    const targetY = targetNode.position.y + (targetNode.height || 0) / 2;

    const dx = targetX - sourceX;
    const dy = targetY - sourceY;
    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);

    // Determine primary direction
    let sourceHandle: string;
    let targetHandle: string;

    if (absDx > absDy) {
        // Horizontal layout - prefer left/right
        if (dx > 0) {
            // Target is to the right
            sourceHandle = 'source-right';
            targetHandle = 'target-left';
        } else {
            // Target is to the left
            sourceHandle = 'source-left';
            targetHandle = 'target-right';
        }
    } else {
        // Vertical layout - prefer top/bottom
        if (dy > 0) {
            // Target is below
            sourceHandle = 'source-bottom';
            targetHandle = 'target-top';
        } else {
            // Target is above
            sourceHandle = 'source-top';
            targetHandle = 'target-bottom';
        }
    }

    return { sourceHandle, targetHandle };
}

/**
 * Select optimal edge type based on layout and node positions.
 * Uses straight for simple layouts, bezier for complex, smoothstep for hierarchical.
 */
function selectOptimalEdgeType(
    sourceNode: RFNode,
    targetNode: RFNode,
    preferredType?: EdgeType
): EdgeType {
    // Use preferred type if specified
    if (preferredType && preferredType !== 'default') {
        return preferredType;
    }

    const sourceX = sourceNode.position.x + (sourceNode.width || 0) / 2;
    const sourceY = sourceNode.position.y + (sourceNode.height || 0) / 2;
    const targetX = targetNode.position.x + (targetNode.width || 0) / 2;
    const targetY = targetNode.position.y + (targetNode.height || 0) / 2;

    const dx = targetX - sourceX;
    const dy = targetY - sourceY;
    const distance = Math.sqrt(dx * dx + dy * dy);
    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);

    // For very close nodes, use straight
    if (distance < 150) {
        return 'straight';
    }

    // For mostly horizontal or vertical, use smoothstep (orthogonal)
    if (absDx / absDy > 3 || absDy / absDx > 3) {
        return 'smoothstep';
    }

    // For diagonal connections, use bezier for smoother curves
    return 'bezier';
}

/**
 * Simple hash function for cache keys
 */
function hashCacheKey(level: C4Level, focusNodeId: string | undefined, collapsedNodeIds: Set<string>): string {
    const collapsedArray = Array.from(collapsedNodeIds).sort().join(',');
    return `${level}:${focusNodeId || ''}:${collapsedArray}`;
}

interface LayoutCache {
    [key: string]: {
        nodes: RFNode[];
        edges: RFEdge[];
        timestamp: number;
    };
}

const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export const SrujaCanvas = () => {
    // Global Store
    const model = useArchitectureStore((s) => s.likec4Model) as unknown as SrujaModelDump | null;
    const dslSource = useArchitectureStore((s) => s.dslSource) as string | null;

    // View State
    const [level, setLevel] = useState<C4Level>(1);
    const [focusNodeId, setFocusNodeId] = useState<string | undefined>(undefined);
    // Collapse/expand state - managed via left navigation (UI panel removed)
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [collapsedNodeIds, _setCollapsedNodeIds] = useState<Set<string>>(new Set());
    const [isComputing, setIsComputing] = useState(false);
    const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);

    // React Flow State
    const [nodes, setNodes, onNodesChange] = useNodesState<RFNode>([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState<RFEdge>([]);

    // Use shared UI theme hook
    const { theme: uiTheme, mode } = useTheme();

    // Determine if dark mode is active (handle 'system' mode)
    const isDark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    // Layout Cache
    const cacheRef = useRef<LayoutCache>({});

    // Get node title helper
    const getNodeTitle = useCallback((nodeId: string | undefined): string => {
        if (!nodeId || !model) return '';
        const element = model.elements[nodeId];
        return element?.title || nodeId;
    }, [model]);


    // Pipeline Execution with Caching
    // Include theme in dependencies to force re-render when theme changes
    useEffect(() => {
        if (!model) {
            setNodes([]);
            setEdges([]);
            return;
        }

        const computeLayout = async () => {
            const cacheKey = hashCacheKey(level, focusNodeId, collapsedNodeIds);
            const cached = cacheRef.current[cacheKey];

            // Check cache validity
            if (cached && (Date.now() - cached.timestamp) < CACHE_TTL) {
                setNodes(cached.nodes);
                setEdges(cached.edges);
                // Fit view after a short delay to ensure nodes are rendered
                setTimeout(() => {
                    reactFlowInstance?.fitView({ padding: 0.2 });
                }, 100);
                return;
            }

            setIsComputing(true);
            try {
                // 1. Generate DOT via Go WASM (with view projection)
                // Use dslSource if available, otherwise fallback cannot proceed
                if (!dslSource) {
                    console.warn('[SrujaCanvas] No DSL source available for DOT generation');
                    setNodes([]);
                    setEdges([]);
                    return;
                }

                const result = await convertDslToDot(dslSource, level, focusNodeId);

                if (!result || !result.dot) {
                    console.warn('[SrujaCanvas] DOT generation failed or returned empty');
                    setNodes([]);
                    setEdges([]);
                    return;
                }

                console.log('[SrujaCanvas] DOT generated via Go WASM, length:', result.dot.length);
                console.log('[SrujaCanvas] Projected Relations count:', result.relations.length);

                // 2. Layout via Graphviz WASM
                const layoutResult = await runGraphviz(result.dot);

                // 3. Build C4Nodes from layout result and model metadata
                const c4Nodes: C4Node[] = layoutResult.nodes.map(layoutNode => {
                    // Get element metadata from model
                    const element = model.elements[layoutNode.id];
                    const kind = element?.kind?.toLowerCase() || 'container';

                    return {
                        id: layoutNode.id,
                        kind: kind as C4Node['kind'],
                        title: element?.title || layoutNode.id,
                        technology: element?.technology ?? undefined,
                        description: typeof element?.description === 'string' ? element.description : undefined,
                        level: level as C4Level, // Use current view level
                        width: layoutNode.width || 200,
                        height: layoutNode.height || 120,
                    };
                });

                // 4. Build edges from projected relations returned by Go
                const c4Edges = result.relations.map((rel: any, idx: number) => {
                    return {
                        id: `e-${rel.from}-${rel.to}-${idx}`,
                        source: rel.from,
                        target: rel.to,
                        label: rel.label || '',
                        technology: undefined,
                    };
                });

                console.log(`[SrujaCanvas] Layout: ${c4Nodes.length} nodes, ${c4Edges.length} edges`);



                // Early return if no nodes
                if (c4Nodes.length === 0) {
                    setNodes([]);
                    setEdges([]);
                    return;
                }

                // 5. React Flow Mapping
                const nextNodes: RFNode[] = c4Nodes.map(node => {
                    const layout = layoutResult.nodes.find(n => n.id === node.id);
                    return {
                        id: node.id,
                        type: 'sruja',
                        position: {
                            x: layout ? layout.x : 0,
                            y: layout ? layout.y : 0
                        },
                        data: { ...node, _theme: mode } as C4Node & Record<string, unknown>,
                        width: node.width,
                        height: node.height,
                    };
                });

                // Create a map of nodes for quick lookup
                const nodeMap = new Map(nextNodes.map(n => [n.id, n]));

                // Filter and create edges only for nodes that exist
                const rfValidNodeIds = new Set(nextNodes.map(n => n.id));
                const nextEdges: RFEdge[] = c4Edges
                    .filter(edge => {
                        const sourceExists = rfValidNodeIds.has(edge.source);
                        const targetExists = rfValidNodeIds.has(edge.target);
                        if (!sourceExists || !targetExists) {
                            console.warn(`[SrujaCanvas] Edge ${edge.id} skipped: source=${edge.source} exists=${sourceExists}, target=${edge.target} exists=${targetExists}`);
                            return false;
                        }
                        return true;
                    })
                    .map(edge => {
                        const sourceNode = nodeMap.get(edge.source);
                        const targetNode = nodeMap.get(edge.target);

                        if (!sourceNode || !targetNode) {
                            // Fallback to default handles if nodes not found
                            // Theme-aware fallback edge colors using shared UI theme
                            const edgeColor = isDark ? uiTheme.neutral[600] : uiTheme.neutral[600];

                            return {
                                id: edge.id,
                                source: edge.source,
                                target: edge.target,
                                sourceHandle: 'source-bottom',
                                targetHandle: 'target-top',
                                type: 'smoothstep' as EdgeType,
                                animated: false,
                                style: { stroke: edgeColor, strokeWidth: 2 },
                                markerEnd: {
                                    type: MarkerType.ArrowClosed,
                                    color: edgeColor,
                                    width: 20,
                                    height: 20,
                                },
                            };
                        }

                        // Select optimal handles based on node positions
                        const { sourceHandle, targetHandle } = selectOptimalHandles(sourceNode, targetNode);

                        // Select optimal edge type (no preferred type in new structure)
                        const edgeType = selectOptimalEdgeType(sourceNode, targetNode, undefined);

                        // Theme-aware edge colors using shared UI theme
                        const edgeColor = isDark ? uiTheme.neutral[600] : uiTheme.neutral[600]; // slate-600 for both themes
                        const labelColor = isDark ? uiTheme.text.primary : uiTheme.text.secondary; // Primary text for dark, secondary for light
                        const labelBgColor = isDark ? uiTheme.surface : uiTheme.background; // Surface for dark, background for light

                        const hasLabel = edge.label && edge.label.trim().length > 0;
                        return {
                            id: edge.id,
                            source: edge.source,
                            target: edge.target,
                            sourceHandle,
                            targetHandle,
                            type: edgeType,
                            ...(hasLabel && {
                                label: edge.label,
                                labelShowBg: true,
                                labelStyle: { color: labelColor, fontWeight: 500, fontSize: '12px' },
                                labelBgStyle: { backgroundColor: labelBgColor, opacity: 0.95, padding: '2px 6px', borderRadius: '4px' },
                            }),
                            animated: false,
                            style: { stroke: edgeColor, strokeWidth: 2 },
                            markerEnd: {
                                type: MarkerType.ArrowClosed,
                                color: edgeColor,
                                width: 20,
                                height: 20,
                            },
                        };
                    });

                // Debug logging
                console.log(`[SrujaCanvas] Layout complete: ${nextNodes.length} nodes, ${nextEdges.length} edges (from ${c4Edges.length} projected edges)`);
                if (nextEdges.length > 0) {
                    console.log('[SrujaCanvas] Sample React Flow edges:', nextEdges.slice(0, 3).map(e => ({
                        id: e.id,
                        source: e.source,
                        target: e.target,
                        label: e.label,
                        labelShowBg: e.labelShowBg,
                        labelStyle: e.labelStyle,
                        labelBgStyle: e.labelBgStyle,
                        sourceHandle: e.sourceHandle,
                        targetHandle: e.targetHandle
                    })));
                    // Log raw C4 edges for comparison
                    console.log('[SrujaCanvas] Sample C4 edges (projected):', c4Edges.slice(0, 3).map(e => ({
                        id: e.id,
                        source: e.source,
                        target: e.target,
                        label: e.label
                    })));
                } else if (c4Edges.length > 0) {
                    console.warn('[SrujaCanvas] WARNING: Had projected edges but none made it to React Flow!');
                    console.warn('[SrujaCanvas] Projected edges:', c4Edges);
                    console.warn('[SrujaCanvas] Valid node IDs:', Array.from(rfValidNodeIds));
                }

                // Cache the result
                cacheRef.current[cacheKey] = {
                    nodes: nextNodes,
                    edges: nextEdges,
                    timestamp: Date.now()
                };

                setNodes(nextNodes);
                setEdges(nextEdges);

                // Fit view after layout
                setTimeout(() => {
                    reactFlowInstance?.fitView({ padding: 0.2 });
                }, 100);

            } catch (err) {
                console.error("Layout failed:", err);
            } finally {
                setIsComputing(false);
            }
        };

        computeLayout();
    }, [model, level, focusNodeId, collapsedNodeIds, setNodes, setEdges, reactFlowInstance, mode, isDark, uiTheme]); // Include theme to re-render on theme change

    // Navigation Handlers
    const onNodeClick = useCallback((event: React.MouseEvent, node: RFNode) => {
        const c4Data = node.data as unknown as C4Node;

        // Check if clicking on expand/collapse button (handled separately)
        const target = event.target as HTMLElement;
        if (target.closest('.expand-btn')) {
            return;
        }

        // Logic: Drill down if possible
        if (level === 1 && c4Data.kind === 'system' && c4Data.navigateOnClick) {
            setFocusNodeId(c4Data.navigateOnClick);
            setLevel(2);
        } else if (level === 2 && c4Data.kind === 'container' && c4Data.navigateOnClick) {
            setFocusNodeId(c4Data.navigateOnClick);
            setLevel(3);
        }
    }, [level]);

    const onGoUp = useCallback(() => {
        if (level === 3 && model && focusNodeId) {
            // L3 -> L2: Find parent system
            const container = model.elements[focusNodeId];
            if (container?.parent) {
                setFocusNodeId(container.parent);
                setLevel(2);
                return;
            }
        }

        if (level === 2) {
            // L2 -> L1: Clear focus
            setLevel(1);
            setFocusNodeId(undefined);
        }
    }, [level, focusNodeId, model]);

    // Build breadcrumb path
    const breadcrumbPath = useMemo(() => {
        if (level === 1) {
            return [{ id: '', title: 'System Context', level: 1 as C4Level }];
        }

        if (level === 2 && focusNodeId) {
            const systemTitle = getNodeTitle(focusNodeId);
            return [
                { id: '', title: 'System Context', level: 1 as C4Level },
                { id: focusNodeId, title: systemTitle, level: 2 as C4Level }
            ];
        }

        if (level === 3 && focusNodeId && model) {
            const container = model.elements[focusNodeId];
            const systemId = container?.parent;
            const systemTitle = systemId ? getNodeTitle(systemId) : '';
            const containerTitle = getNodeTitle(focusNodeId);

            return [
                { id: '', title: 'System Context', level: 1 as C4Level },
                { id: systemId || '', title: systemTitle, level: 2 as C4Level },
                { id: focusNodeId, title: containerTitle, level: 3 as C4Level }
            ];
        }

        return [];
    }, [level, focusNodeId, model, getNodeTitle]);

    // Keyboard Shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Escape to go up one level
            if (e.key === 'Escape') {
                if (level > 1) {
                    onGoUp();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [level, onGoUp]);

    const handleBreadcrumbClick = useCallback((targetLevel: C4Level, targetId?: string) => {
        if (targetLevel === 1) {
            setLevel(1 as C4Level);
            setFocusNodeId(undefined);
        } else if (targetLevel === 2 && targetId) {
            setLevel(2 as C4Level);
            setFocusNodeId(targetId);
        } else if (targetLevel === 3 && targetId) {
            setLevel(3 as C4Level);
            setFocusNodeId(targetId);
        }
    }, []);

    // Theme-aware colors using shared UI theme
    const bgColor = uiTheme.background;
    const paperBg = isDark
        ? `${uiTheme.surface}F2` // 95% opacity (F2 in hex)
        : `${uiTheme.background}F2`;
    const backgroundPatternColor = isDark ? uiTheme.surface : uiTheme.neutral[200]; // slate-200 for light

    return (
        <div className="w-full h-full relative" style={{ backgroundColor: bgColor }}>
            {/* Breadcrumb / Navigation Header */}
            <Paper
                shadow="md"
                p="xs"
                withBorder
                style={{
                    position: 'absolute',
                    top: 16,
                    left: 16,
                    zIndex: 10,
                    backgroundColor: paperBg,
                    backdropFilter: 'blur(8px)',
                }}
            >
                <Group gap="xs" align="center">
                    {level > 1 && (
                        <ActionIcon
                            variant="subtle"
                            onClick={onGoUp}
                            aria-label="Go up one level"
                            size="sm"
                        >
                            <ArrowLeft size={16} />
                        </ActionIcon>
                    )}

                    <Group gap={4} align="center">
                        {breadcrumbPath.map((item, idx) => (
                            <Group key={item.id || 'root'} gap={4} align="center">
                                {idx > 0 && (
                                    <Text size="sm" c="dimmed">
                                        /
                                    </Text>
                                )}
                                <Button
                                    variant={idx === breadcrumbPath.length - 1 ? 'light' : 'subtle'}
                                    size="xs"
                                    onClick={() => handleBreadcrumbClick(item.level, item.id || undefined)}
                                    style={{
                                        fontWeight: idx === breadcrumbPath.length - 1 ? 600 : 400,
                                    }}
                                >
                                    {item.title}
                                </Button>
                            </Group>
                        ))}
                    </Group>
                </Group>
            </Paper>

            {isComputing && (
                <div
                    style={{
                        position: 'absolute',
                        inset: 0,
                        zIndex: 50,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        backgroundColor: isDark ? `${uiTheme.background}B3` : `${uiTheme.background}B3`, // 70% opacity (B3 in hex)
                        backdropFilter: 'blur(4px)',
                    }}
                >
                    <Paper shadow="lg" p="md" withBorder>
                        <Stack gap="xs" align="center">
                            <Loader size="sm" />
                            <Text size="sm" fw={500} c="dimmed">
                                Computing Layout...
                            </Text>
                            <Text size="xs" c="dimmed">
                                This may take a moment
                            </Text>
                        </Stack>
                    </Paper>
                </div>
            )}

            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onNodeClick={onNodeClick}
                onInit={setReactFlowInstance}
                nodeTypes={nodeTypes}
                fitView
                minZoom={0.1}
                maxZoom={2}
            >
                <Background color={backgroundPatternColor} gap={16} />
                <Controls />
            </ReactFlow>
        </div>
    );
};

// packages/react-flow-architecture/src/utils/srujaLayoutEngine.ts
// Sruja Layout Engine integration for React Flow
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData, ArchitectureJSON } from '../types';
import {
    createC4Id,
    createC4Graph,
    layout,
    type C4Graph,
    type C4Node,
    type C4Relationship,
    type C4ViewState,
    createDefaultViewState,
    SystemContextView,
    ContainerView,
    ComponentView,
    InteractivePreset,
    type C4LayoutResult,
} from '@sruja/layout';
import type { C4Level } from '../types';

interface LayoutOptions {
    level: C4Level;
    focusedSystemId?: string;
    focusedContainerId?: string;
    direction?: 'TB' | 'LR' | 'RL';
    expandedNodes?: Set<string>;
}

/**
 * Convert ArchitectureJSON to C4Graph format
 */
function architectureToC4Graph(
    data: ArchitectureJSON,
    options: LayoutOptions
): C4Graph {
    const nodes: C4Node[] = [];
    const relationships: C4Relationship[] = [];
    const arch = data.architecture;

    // Map C4Level to layout C4Level
    const levelMap: Record<C4Level, 'landscape' | 'context' | 'container' | 'component'> = {
        L0: 'landscape',
        L1: 'context',
        L2: 'container',
        L3: 'component',
    };
    const targetLevel = levelMap[options.level] || 'context';

    // Add persons
    arch.persons?.forEach((person) => {
        nodes.push({
            id: createC4Id(person.id),
            label: person.label ?? person.id,
            kind: 'Person',
            level: 'context',
            tags: new Set(),
            description: person.description,
        });
    });

    // Add systems
    arch.systems?.forEach((system) => {
        const isExternal = checkIsExternal(system.metadata);
        nodes.push({
            id: createC4Id(system.id),
            label: system.label ?? system.id,
            kind: isExternal ? 'ExternalSystem' : 'SoftwareSystem',
            level: 'context',
            tags: new Set(),
            description: system.description,
        });

        // Add containers if at L2 or L3 level OR if parent system is expanded
        // NOTE: expandedNodes uses 'raw' IDs (e.g. "EcommercePlatform") not C4Ids? Assuming raw.
        const isExpanded = options.expandedNodes?.has(system.id);
        const shouldIncludeContainers = targetLevel === 'container' || targetLevel === 'component' || isExpanded;

        if (shouldIncludeContainers && system.containers) {
            system.containers.forEach((container) => {
                nodes.push({
                    id: createC4Id(container.id),
                    label: container.label ?? container.id,
                    kind: 'Container',
                    level: 'container',
                    parentId: createC4Id(system.id),
                    tags: new Set(),
                    description: container.description,
                    technology: container.technology,
                });

                // Add components if at L3 level
                if (targetLevel === 'component' && container.components) {
                    container.components.forEach((component) => {
                        nodes.push({
                            id: createC4Id(component.id),
                            label: component.label ?? component.id,
                            kind: 'Component',
                            level: 'component',
                            parentId: createC4Id(container.id),
                            tags: new Set(),
                            description: component.description,
                            technology: component.technology,
                        });
                    });
                }
            });
        }

        // Add datastores
        system.datastores?.forEach((ds) => {
            nodes.push({
                id: createC4Id(ds.id),
                label: ds.label ?? ds.id,
                kind: 'Database',
                level: targetLevel === 'component' ? 'component' : 'container',
                parentId: targetLevel === 'component' ? undefined : createC4Id(system.id),
                tags: new Set(),
            });
        });

        // Add queues
        system.queues?.forEach((queue) => {
            nodes.push({
                id: createC4Id(queue.id),
                label: queue.label ?? queue.id,
                kind: 'Queue',
                level: targetLevel === 'component' ? 'component' : 'container',
                parentId: targetLevel === 'component' ? undefined : createC4Id(system.id),
                tags: new Set(),
            });
        });
    });

    // Helper to resolve dotted IDs to actual node IDs
    const resolveNodeId = (id: string): string | null => {
        // Check if exact match exists
        if (nodes.some(n => String(n.id) === id)) {
            return id;
        }

        // Handle dotted IDs
        if (id.includes('.')) {
            const parts = id.split('.');
            // Try full path first (e.g., "EcommercePlatform.WebStore")
            const fullPath = parts.join('.');
            if (nodes.some(n => String(n.id) === fullPath)) {
                return fullPath;
            }
            // Try last part (e.g., "WebStore")
            const lastPart = parts[parts.length - 1];
            if (nodes.some(n => String(n.id) === lastPart)) {
                return lastPart;
            }
            // Try first part (e.g., "EcommercePlatform")
            const firstPart = parts[0];
            if (nodes.some(n => String(n.id) === firstPart)) {
                return firstPart;
            }
        }

        return null;
    };

    // Add relationships
    const addRelations = (rels: typeof arch.relations) => {
        rels?.forEach((rel) => {
            const fromId = resolveNodeId(rel.from);
            const toId = resolveNodeId(rel.to);

            if (fromId && toId) {
                relationships.push({
                    id: `${rel.from}-${rel.to}`,
                    from: createC4Id(fromId),
                    to: createC4Id(toId),
                    label: rel.label ?? rel.verb,
                });
            }
        });
    };

    addRelations(arch.relations);

    // Add system-level relations
    arch.systems?.forEach((system) => {
        addRelations(system.relations);
        system.containers?.forEach((container) => {
            addRelations(container.relations);
            container.components?.forEach((component) => {
                addRelations(component.relations);
            });
        });
    });

    return createC4Graph(nodes, relationships, {
        title: data.metadata?.name,
        version: data.metadata?.version,
    });
}

/**
 * Convert C4LayoutResult to React Flow nodes and edges
 */
function layoutResultToReactFlow(
    result: C4LayoutResult,
    originalNodes: Node<C4NodeData>[],
    originalEdges: Edge[]
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    const edgeMap = new Map(originalEdges.map((e) => [e.id, e]));

    const positionedMap = result.nodes;

    const updatedNodes = originalNodes
        .map((node) => {
            const positioned = positionedMap.get(createC4Id(node.id));
            if (!positioned) return node;

            let x = positioned.bbox.x;
            let y = positioned.bbox.y;

            if (node.parentId) {
                const parent = positionedMap.get(createC4Id(node.parentId));
                if (parent) {
                    x = positioned.bbox.x - parent.bbox.x;
                    y = positioned.bbox.y - parent.bbox.y;
                }
            }

            const width = Math.max(1, positioned.bbox.width);
            const height = Math.max(1, positioned.bbox.height);

            return {
                ...node,
                position: { x, y },
                width,
                height,
                style: { ...node.style, width, height },
            };
        })
        .sort((a, b) => {
            if (b.parentId === a.id) return -1;
            if (a.parentId === b.id) return 1;
            const depthA = getDepth(a.id, originalNodes);
            const depthB = getDepth(b.id, originalNodes);
            return depthA - depthB;
        });

    const relBySourceTarget = new Map<string, typeof result.relationships[number]>();
    result.relationships.forEach((r) => {
        relBySourceTarget.set(`${r.sourceId}|${r.targetId}`, r);
    });

    const updatedEdges = originalEdges.map((e) => {
        const key = `${e.source}|${e.target}`;
        const rel = relBySourceTarget.get(key);
        const base = edgeMap.get(e.id) ?? e;
        const label = base.label;
        if (rel) {
            return {
                ...base,
                label,
                type: 'routed',
                data: { ...base.data, points: rel.points, labelPosition: rel.labelPosition },
            };
        }
        return { ...base, label };
    });

    return { nodes: updatedNodes, edges: updatedEdges };
}

/**
 * Get appropriate C4ViewState based on level and focus
 */
function getViewState(options: LayoutOptions): C4ViewState {
    const { level, focusedSystemId, focusedContainerId } = options;

    if (level === 'L1' || level === 'L0') {
        if (focusedSystemId) {
            return SystemContextView(createC4Id(focusedSystemId));
        }
        return createDefaultViewState();
    }

    if (level === 'L2' && focusedSystemId) {
        return ContainerView(createC4Id(focusedSystemId));
    }

    if (level === 'L3' && focusedContainerId) {
        return ComponentView(createC4Id(focusedContainerId));
    }

    return createDefaultViewState();
}

/**
 * Apply Sruja layout to nodes and edges
 * Returns nodes with updated positions and edges
 */
export function applySrujaLayout(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    architectureData: ArchitectureJSON,
    options: LayoutOptions = { level: 'L1' }
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    // Convert to C4Graph
    const graph = architectureToC4Graph(architectureData, options);

    // Get view state
    const viewState = getViewState(options);

    // Apply layout
    const layoutOptions = {
        ...InteractivePreset,
        strategy: (options.level === 'L1') ? 'l1-context' as const : undefined,
        direction: options.direction || 'TB',
        minSize: { width: 220, height: 100 },
        spacing: {
            ...InteractivePreset.spacing,
            node: { ...InteractivePreset.spacing.node, SoftwareSystem: 100, Container: 80, Component: 60 },
            padding: { ...InteractivePreset.spacing.padding, SoftwareSystem: 100, Container: 80, Component: 40 },
        },
    };

    const result = layout(graph, viewState, layoutOptions);

    // Convert back to React Flow format
    return layoutResultToReactFlow(result, nodes, edges);
}

function getDepth(nodeId: string, nodes: Node[]): number {
    let depth = 0;
    let current = nodes.find((n) => n.id === nodeId);
    while (current && current.parentId) {
        depth++;
        const pid = current.parentId!;
        current = nodes.find((n) => n.id === pid);
    }
    return depth;
}

function checkIsExternal(metadata?: { key: string; value?: string }[]): boolean {
    if (!metadata) return false;
    const ext = metadata.find(m => m.key.toLowerCase() === 'external');
    if (ext && ext.value?.toLowerCase() === 'true') return true;

    const tags = metadata.find(m => m.key.toLowerCase() === 'tags');
    if (tags && tags.value?.toLowerCase().includes('external')) return true;

    const loc = metadata.find(m => m.key.toLowerCase() === 'location');
    if (loc && loc.value?.toLowerCase() === 'external') return true;

    return false;
}









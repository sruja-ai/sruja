import type { Node, Edge } from '@xyflow/react';
import { MarkerType } from '@xyflow/react';
import type {
    ArchitectureJSON,
    SystemJSON,
    ContainerJSON,
    ComponentJSON,
    PersonJSON,
    DataStoreJSON,
    QueueJSON,
    RelationJSON,
    C4Level,
    C4NodeData,
    ViewData,
    ViewNode,
    ViewEdge,
} from '../types';

interface TransformOptions {
    level: C4Level;
    focusedSystemId?: string;
    focusedContainerId?: string;
    expandedNodes?: Set<string>;
}

/**
 * Transform Sruja Architecture JSON to React Flow nodes and edges
 * Uses pre-computed views when available (from --extended export)
 */
export function jsonToReactFlow(
    data: ArchitectureJSON,
    options: TransformOptions
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    // Use pre-computed views if available
    if (data.views) {
        return transformFromViews(data, options);
    }

    // Fallback to legacy transformation
    return transformLegacy(data, options);
}

/**
 * Transform using pre-computed views from --extended export
 */
function transformFromViews(
    data: ArchitectureJSON,
    options: TransformOptions
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    const views = data.views!;
    const expandedNodes = options.expandedNodes ?? new Set<string>();
    let viewData: ViewData | undefined;

    switch (options.level) {
        case 'L0':
        case 'L1':
            viewData = views.L1;
            break;
        case 'L2':
            if (options.focusedSystemId) {
                viewData = views.L2[options.focusedSystemId];
            }
            break;
        case 'L3':
            if (options.focusedSystemId && options.focusedContainerId) {
                const key = `${options.focusedSystemId}.${options.focusedContainerId}`;
                viewData = views.L3[key];
            }
            break;
    }

    if (!viewData) {
        // Fallback to L1 if requested view not available
        viewData = views.L1;
    }

    return convertViewDataToReactFlow(viewData, expandedNodes);
}

/**
 * Convert ViewData to React Flow nodes and edges
 */
function convertViewDataToReactFlow(
    viewData: ViewData,
    expandedNodes: Set<string>
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    const nodes: Node<C4NodeData>[] = viewData.nodes.map((node, index) =>
        createNodeFromViewNode(node, index, expandedNodes)
    );

    // Promote parents to boundary types when they have children
    const byId = new Map<string, Node<C4NodeData>>();
    nodes.forEach(n => byId.set(n.id, n));

    const childCounts = new Map<string, number>();
    nodes.forEach(n => {
        const pid = (n.data as any)?.parentId as string | undefined;
        if (pid) childCounts.set(pid, (childCounts.get(pid) || 0) + 1);
    });

    childCounts.forEach((count, parentId) => {
        const parent = byId.get(parentId);
        if (parent) {
            const type = (parent.data as any)?.type;
            if (type === 'system') {
                parent.type = 'system-boundary' as any;
                (parent.data as any).type = 'system-boundary';
                (parent.data as any).childCount = count;
            }
            if (type === 'container') {
                parent.type = 'container-boundary' as any;
                (parent.data as any).type = 'container-boundary';
                (parent.data as any).childCount = count;
            }
        }
    });

    const edges: Edge[] = viewData.edges.map((edge) =>
        createEdgeFromViewEdge(edge)
    );

    return { nodes, edges };
}

function createNodeFromViewNode(
    node: ViewNode,
    index: number,
    expandedNodes: Set<string>
): Node<C4NodeData> {
    // Simple grid layout - will be overridden by ELK
    const col = index % 4;
    const row = Math.floor(index / 4);
    const xSpacing = 250;
    const ySpacing = 180;

    const isExpanded = expandedNodes.has(node.id);

    return {
        id: node.id,
        type: node.type,
        position: { x: col * xSpacing, y: row * ySpacing },
        parentId: node.parentId,
        data: {
            id: node.id,
            label: node.label,
            type: node.type,
            technology: node.technology,
            description: node.description,
            isExternal: node.isExternal ?? false,
            parentId: node.parentId,
            childCount: node.childCount,
            expanded: isExpanded,
        },
    };
}

function createEdgeFromViewEdge(edge: ViewEdge): Edge {
    return {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label,
        type: 'relation', // Use our custom RelationEdge
        animated: false,
        style: { stroke: '#707070', strokeWidth: 1.5 },
        markerEnd: {
            type: MarkerType.ArrowClosed,
            color: '#707070',
            width: 20,
            height: 20,
        },
        data: {
            label: edge.label,
            technology: edge.technology,
            interaction: edge.interaction,
            tags: edge.tags,
            preferredRoute: edge.preferredRoute,
            bidirectional: edge.bidirectional,
        },
    };
}

/**
 * Legacy transformation (fallback when views not available)
 */
function transformLegacy(
    data: ArchitectureJSON,
    options: TransformOptions
): { nodes: Node<C4NodeData>[]; edges: Edge[] } {
    const nodes: Node<C4NodeData>[] = [];
    const edges: Edge[] = [];
    const nodeIdSet = new Set<string>();

    const expandedNodes = options.expandedNodes ?? new Set<string>();

    switch (options.level) {
        case 'L0':
            transformLandscapeLevel(data, nodes, edges, nodeIdSet);
            break;
        case 'L1':
            transformSystemLevel(data, nodes, edges, nodeIdSet, expandedNodes);
            break;
        case 'L2':
            transformContainerLevel(data, options.focusedSystemId!, nodes, edges, nodeIdSet);
            break;
        case 'L3':
            transformComponentLevel(
                data,
                options.focusedSystemId!,
                options.focusedContainerId!,
                nodes,
                edges,
                nodeIdSet
            );
            break;
    }

    return { nodes, edges };
}

function transformSystemLevel(
    data: ArchitectureJSON,
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    nodeIdSet: Set<string>,
    expandedNodes: Set<string>
) {
    const arch = data.architecture;
    let yOffset = 0;
    const xSpacing = 300;
    const ySpacing = 200;

    // Add persons at the top
    arch.persons?.forEach((person, index) => {
        nodes.push(createPersonNode(person, index * xSpacing, 0));
        nodeIdSet.add(person.id);
    });

    yOffset = ySpacing;

    // Add systems
    arch.systems?.forEach((system, index) => {
        const isExpanded = expandedNodes.has(system.id);
        nodes.push(createSystemNode(system, index * xSpacing, yOffset, isExpanded));
        nodeIdSet.add(system.id);

        // If expanded, add containers as children
        if (isExpanded && system.containers) {
            system.containers.forEach((container, cIndex) => {
                nodes.push(
                    createContainerNode(
                        container,
                        index * xSpacing + 50,
                        yOffset + 100 + cIndex * 80,
                        system.id
                    )
                );
                nodeIdSet.add(container.id);
            });
        }
    });

    // Add top-level relations
    arch.relations?.forEach((relation) => {
        // Only add edges for visible nodes
        if (nodeIdSet.has(relation.from) && nodeIdSet.has(relation.to)) {
            edges.push(createRelationEdge(relation));
        } else {
            // Try to find parent system for nested references
            const fromResolved = resolveNodeId(relation.from, arch, nodeIdSet);
            const toResolved = resolveNodeId(relation.to, arch, nodeIdSet);
            if (fromResolved && toResolved) {
                edges.push(createRelationEdge({
                    ...relation,
                    from: fromResolved,
                    to: toResolved,
                }));
            }
        }
    });
}

function transformContainerLevel(
    data: ArchitectureJSON,
    systemId: string,
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    nodeIdSet: Set<string>
) {
    const arch = data.architecture;
    const system = arch.systems?.find((s) => s.id === systemId);
    if (!system) return;

    const xSpacing = 280;
    const yBase = 150;
    const externalXBase = -200;
    const externalYBase = 0;

    // Add system boundary as parent container area
    nodes.push({
        id: system.id,
        type: 'system-boundary',
        position: { x: 0, y: yBase - 80 },
        data: {
            id: system.id,
            label: system.label ?? system.id,
            description: system.description,
            type: 'system-boundary',
            childCount: system.containers?.length ?? 0,
        },
    });
    nodeIdSet.add(system.id);

    // Track containers and their IDs for edge resolution
    const containerIds = new Set<string>();

    // Add containers in a grid layout
    const cols = Math.ceil(Math.sqrt(system.containers?.length ?? 1));
    system.containers?.forEach((container, index) => {
        const col = index % cols;
        const row = Math.floor(index / cols);
        nodes.push(createContainerNode(
            container,
            col * xSpacing,
            yBase + row * 180,
            systemId,
            false // not dimmed
        ));
        nodeIdSet.add(container.id);
        containerIds.add(container.id);
    });

    // Add datastores below containers
    const dsYBase = yBase + Math.ceil((system.containers?.length ?? 1) / cols) * 180 + 50;
    system.datastores?.forEach((ds, index) => {
        nodes.push(createDataStoreNode(ds, index * xSpacing, dsYBase, false));
        nodeIdSet.add(ds.id);
    });

    // Add queues next to datastores
    system.queues?.forEach((queue, index) => {
        const xPos = (system.datastores?.length ?? 0) * xSpacing + index * xSpacing;
        nodes.push(createQueueNode(queue, xPos, dsYBase, false));
        nodeIdSet.add(queue.id);
    });

    // Collect all relation endpoints to find external connections
    const externalNodeIds = new Set<string>();

    // Check container relations for external references
    system.containers?.forEach((container) => {
        container.relations?.forEach((relation) => {
            if (!containerIds.has(relation.from) && !nodeIdSet.has(relation.from)) {
                externalNodeIds.add(relation.from);
            }
            if (!containerIds.has(relation.to) && !nodeIdSet.has(relation.to)) {
                externalNodeIds.add(relation.to);
            }
        });
    });

    // Check system-level relations
    system.relations?.forEach((relation) => {
        if (!nodeIdSet.has(relation.from)) externalNodeIds.add(relation.from);
        if (!nodeIdSet.has(relation.to)) externalNodeIds.add(relation.to);
    });

    // Check top-level relations for connections to this system's containers
    arch.relations?.forEach((relation) => {
        const fromInSystem = nodeIdSet.has(relation.from) || relation.from.startsWith(systemId + '.');
        const toInSystem = nodeIdSet.has(relation.to) || relation.to.startsWith(systemId + '.');

        if (fromInSystem && !toInSystem) {
            externalNodeIds.add(relation.to);
        }
        if (toInSystem && !fromInSystem) {
            externalNodeIds.add(relation.from);
        }
    });

    // Add external systems and persons as dimmed nodes
    let externalIndex = 0;
    externalNodeIds.forEach((extId) => {
        // Find the external node
        const person = arch.persons?.find(p => p.id === extId);
        const extSystem = arch.systems?.find(s => s.id === extId || s.id === extId.split('.')[0]);

        const yPos = externalYBase + externalIndex * 120;

        if (person) {
            nodes.push({
                ...createPersonNode(person, externalXBase, yPos),
                data: { ...createPersonNode(person, 0, 0).data, isExternal: true },
            });
            nodeIdSet.add(person.id);
        } else if (extSystem && extSystem.id !== systemId) {
            nodes.push({
                id: extSystem.id,
                type: 'system',
                position: { x: externalXBase, y: yPos },
                data: {
                    id: extSystem.id,
                    label: extSystem.label ?? extSystem.id,
                    description: extSystem.description,
                    type: 'system',
                    isExternal: true,
                    childCount: extSystem.containers?.length ?? 0,
                },
            });
            nodeIdSet.add(extSystem.id);
        }
        externalIndex++;
    });

    // Add all relevant edges
    const addedEdges = new Set<string>();

    // Container-level relations
    system.containers?.forEach((container) => {
        container.relations?.forEach((relation) => {
            const edgeId = `${relation.from}-${relation.to}`;
            if (!addedEdges.has(edgeId)) {
                const fromResolved = resolveNodeId(relation.from, arch, nodeIdSet);
                const toResolved = resolveNodeId(relation.to, arch, nodeIdSet);
                if (fromResolved && toResolved) {
                    edges.push(createRelationEdge({ ...relation, from: fromResolved, to: toResolved }));
                    addedEdges.add(edgeId);
                }
            }
        });
    });

    // System-level relations
    system.relations?.forEach((relation) => {
        const edgeId = `${relation.from}-${relation.to}`;
        if (!addedEdges.has(edgeId)) {
            const fromResolved = resolveNodeId(relation.from, arch, nodeIdSet);
            const toResolved = resolveNodeId(relation.to, arch, nodeIdSet);
            if (fromResolved && toResolved) {
                edges.push(createRelationEdge({ ...relation, from: fromResolved, to: toResolved }));
                addedEdges.add(edgeId);
            }
        }
    });

    // Top-level relations that connect to this system
    arch.relations?.forEach((relation) => {
        const edgeId = `${relation.from}-${relation.to}`;
        if (!addedEdges.has(edgeId)) {
            const fromResolved = resolveNodeId(relation.from, arch, nodeIdSet);
            const toResolved = resolveNodeId(relation.to, arch, nodeIdSet);
            if (fromResolved && toResolved) {
                edges.push(createRelationEdge({ ...relation, from: fromResolved, to: toResolved }));
                addedEdges.add(edgeId);
            }
        }
    });
}

function transformComponentLevel(
    data: ArchitectureJSON,
    systemId: string,
    containerId: string,
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    nodeIdSet: Set<string>
) {
    const system = data.architecture.systems?.find((s) => s.id === systemId);
    if (!system) return;

    const container = system.containers?.find((c) => c.id === containerId);
    if (!container) return;

    const xSpacing = 200;
    const yBase = 50;

    nodes.push({
        id: containerId,
        type: 'container-boundary',
        position: { x: 0, y: yBase - 30 },
        data: {
            id: containerId,
            label: container.label ?? containerId,
            description: container.description,
            type: 'container-boundary',
            childCount: container.components?.length ?? 0,
        },
    });
    nodeIdSet.add(containerId);

    // Add components
    container.components?.forEach((component, index) => {
        nodes.push(createComponentNode(component, index * xSpacing, yBase, containerId));
        nodeIdSet.add(component.id);
    });

    // Add datastores within container
    container.datastores?.forEach((ds, index) => {
        nodes.push(createDataStoreNode(ds, index * xSpacing, yBase + 150));
        nodeIdSet.add(ds.id);
    });

    // Add component-level relations
    container.components?.forEach((component) => {
        component.relations?.forEach((relation) => {
            edges.push(createRelationEdge(relation));
        });
    });
}

// Helper to resolve dotted references like "ECommerce.API" to visible node
function resolveNodeId(
    nodeRef: string,
    _arch: ArchitectureJSON['architecture'],
    visibleNodes: Set<string>
): string | null {
    if (visibleNodes.has(nodeRef)) return nodeRef;

    // Try to find the parent system
    const parts = nodeRef.split('.');
    if (parts.length > 1) {
        const systemId = parts[0];
        if (visibleNodes.has(systemId)) return systemId;
    }

    return null;
}

// Node creation helpers
function createPersonNode(person: PersonJSON, x: number, y: number): Node<C4NodeData> {
    return {
        id: person.id,
        type: 'person',
        position: { x, y },
        data: {
            id: person.id,
            label: person.label ?? person.id,
            description: person.description,
            type: 'person',
        },
    };
}

function createSystemNode(
    system: SystemJSON,
    x: number,
    y: number,
    expanded: boolean
): Node<C4NodeData> {
    return {
        id: system.id,
        type: 'system',
        position: { x, y },
        data: {
            id: system.id,
            label: system.label ?? system.id,
            description: system.description,
            type: 'system',
            childCount: system.containers?.length ?? 0,
            expanded,
        },
    };
}

function createContainerNode(
    container: ContainerJSON,
    x: number,
    y: number,
    parentId: string,
    isExternal: boolean = false
): Node<C4NodeData> {
    return {
        id: container.id,
        type: 'container',
        position: { x, y },
        parentId,
        data: {
            id: container.id,
            label: container.label ?? container.id,
            description: container.description,
            technology: container.technology,
            type: 'container',
            childCount: container.components?.length ?? 0,
            isExternal,
        },
    };
}

function createComponentNode(
    component: ComponentJSON,
    x: number,
    y: number,
    parentId: string
): Node<C4NodeData> {
    return {
        id: component.id,
        type: 'component',
        position: { x, y },
        parentId,
        data: {
            id: component.id,
            label: component.label ?? component.id,
            description: component.description,
            technology: component.technology,
            type: 'component',
        },
    };
}

function createDataStoreNode(ds: DataStoreJSON, x: number, y: number, isExternal: boolean = false): Node<C4NodeData> {
    return {
        id: ds.id,
        type: 'datastore',
        position: { x, y },
        data: {
            id: ds.id,
            label: ds.label ?? ds.id,
            type: 'datastore',
            isExternal,
        },
    };
}

function createQueueNode(queue: QueueJSON, x: number, y: number, isExternal: boolean = false): Node<C4NodeData> {
    return {
        id: queue.id,
        type: 'queue',
        position: { x, y },
        data: {
            id: queue.id,
            label: queue.label ?? queue.id,
            type: 'queue',
            isExternal,
        },
    };
}

function createRelationEdge(relation: RelationJSON, index: number = 0): Edge {
    // Create unique edge ID to handle potential duplicates
    const edgeId = `edge-${relation.from}-${relation.to}-${index}`;

    return {
        id: edgeId,
        source: relation.from,
        target: relation.to,
        label: relation.label ?? relation.verb,
        type: 'relation', // Use our custom RelationEdge
        animated: false,
        style: { stroke: '#707070', strokeWidth: 1.5 },
        markerEnd: {
            type: MarkerType.ArrowClosed,
            color: '#707070',
            width: 20,
            height: 20,
        },
        data: {
            label: relation.label ?? relation.verb,
            technology: relation.technology,
            interaction: relation.interaction,
            tags: relation.tags,
            preferredRoute: relation.preferredRoute,
            bidirectional: relation.bidirectional,
        },
    };
}
function transformLandscapeLevel(
    data: ArchitectureJSON,
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    nodeIdSet: Set<string>
) {
    const arch = data.architecture;
    const boundaryId = 'enterprise';
    const label = data.metadata?.name || 'Landscape';

    nodes.push({
        id: boundaryId,
        type: 'enterprise-boundary',
        position: { x: 0, y: 0 },
        data: { id: boundaryId, label, type: 'enterprise-boundary' },
    });
    nodeIdSet.add(boundaryId);

    let x = 0; let y = 120; const spacing = 280;
    arch.systems?.forEach((system, idx) => {
        nodes.push({
            ...createSystemNode(system, x + (idx % 3) * spacing, y + Math.floor(idx / 3) * 220, false),
            parentId: boundaryId,
        });
        nodeIdSet.add(system.id);
    });

    arch.relations?.forEach((relation) => {
        if (nodeIdSet.has(relation.from) && nodeIdSet.has(relation.to)) {
            edges.push(createRelationEdge(relation));
        }
    });
}

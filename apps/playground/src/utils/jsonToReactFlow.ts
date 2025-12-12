import type { Node, Edge } from "@xyflow/react";

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
  Badge,
} from "../types";

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
  let result: { nodes: Node<C4NodeData>[]; edges: Edge[] };

  // Use pre-computed views if available
  if (data.views) {
    result = transformFromViews(data, options);
  } else {
    // Fallback to legacy transformation
    result = transformLegacy(data, options);
  }

  // Deduplicate nodes by ID (keep first occurrence)
  const seenNodeIds = new Set<string>();
  result.nodes = result.nodes.filter((node) => {
    if (seenNodeIds.has(node.id)) {
      return false;
    }
    seenNodeIds.add(node.id);
    return true;
  });

  // Deduplicate edges by ID (keep first occurrence)
  const seenEdgeIds = new Set<string>();
  result.edges = result.edges.filter((edge) => {
    if (seenEdgeIds.has(edge.id)) {
      return false;
    }
    seenEdgeIds.add(edge.id);
    return true;
  });

  return result;
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
    case "L1":
      viewData = views.L1;
      break;
    case "L2":
      if (options.focusedSystemId) {
        viewData = views.L2[options.focusedSystemId];
      }
      break;
    case "L3":
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

  const edges: Edge[] = viewData.edges
    .filter((edge) => edge.source !== edge.target)
    .map((edge) => createEdgeFromViewEdge(edge));

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
    type: "relation",
    data: {
      label: edge.label,
    },
    animated: false,
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
    case "L1":
      transformSystemLevel(data, nodes, edges, nodeIdSet, expandedNodes);
      break;
    case "L2":
      transformContainerLevel(data, options.focusedSystemId!, nodes, edges, nodeIdSet);
      break;
    case "L3":
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

/**
 * Transform Overall Level (L0) - Shows the complete architecture
 * Includes all persons, systems, containers, components, datastores, and queues
 */

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
    const isExt = checkIsExternal(person.metadata);
    nodes.push(createPersonNode(person, index * xSpacing, 0, isExt));
    nodeIdSet.add(person.id);
  });

  yOffset = ySpacing;

  // Add systems
  console.log(
    `Processing ${arch.systems?.length ?? 0} systems. Expanded nodes:`,
    Array.from(expandedNodes)
  );
  arch.systems?.forEach((system, index) => {
    const isExpanded = expandedNodes.has(system.id);
    const isExt = checkIsExternal(system.metadata);
    console.log(
      `System ${system.id}: isExpanded=${isExpanded}, hasContainers=${!!system.containers}, containerCount=${system.containers?.length ?? 0}`
    );
    nodes.push(createSystemNode(system, index * xSpacing, yOffset, isExpanded, isExt));
    nodeIdSet.add(system.id);

    // If expanded, add containers as children (mindmap-style expansion)
    if (isExpanded && system.containers) {
      console.log(
        `  Adding ${system.containers.length} containers for expanded system ${system.id}`
      );
      system.containers.forEach((container, cIndex) => {
        const containerExpanded = expandedNodes.has(container.id);
        const containerNode = createContainerNode(
          container,
          50 + (cIndex % 2) * 200, // Grid layout: 2 columns
          100 + Math.floor(cIndex / 2) * 120, // Relative to parent
          system.id,
          false,
          containerExpanded,
          true // useCompoundParent - enable React Flow compound rendering
        );
        console.log(
          `    Created container node: ${container.id}, parentId: ${containerNode.parentId}, expanded: ${containerExpanded}`
        );
        nodes.push(containerNode);
        nodeIdSet.add(container.id);

        // If container is also expanded, add its components as children
        if (containerExpanded && container.components) {
          container.components.forEach((component, compIndex) => {
            nodes.push(
              createComponentNode(
                component,
                index * xSpacing + 100,
                yOffset + 100 + cIndex * 80 + 60 + compIndex * 60,
                container.id
              )
            );
            nodeIdSet.add(component.id);
          });
        }
      });
    }
  });

  // Add top-level relations
  // Collect all potential relations
  const allRelations: RelationJSON[] = [...(arch.relations ?? [])];

  arch.systems?.forEach((system) => {
    if (system.relations) {
      allRelations.push(...system.relations);
    }
    system.containers?.forEach((container) => {
      if (container.relations) {
        allRelations.push(...container.relations);
      }
    });
  });

  const addedEdges = new Set<string>();

  allRelations.forEach((relation) => {
    let from = relation.from;
    let to = relation.to;

    // Resolve nodes if they are not directly visible
    if (!nodeIdSet.has(from)) from = resolveNodeId(from, arch, nodeIdSet) || from;
    if (!nodeIdSet.has(to)) to = resolveNodeId(to, arch, nodeIdSet) || to;

    // Only add if source and target are visible
    if (nodeIdSet.has(from) && nodeIdSet.has(to) && from !== to) {
      // Generate a unique key for the connection pair to check duplicates
      const connectionKey = `${from}-${to}`;

      if (!addedEdges.has(connectionKey)) {
        edges.push(
          createRelationEdge({
            ...relation,
            from,
            to,
          })
        );
        addedEdges.add(connectionKey);
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

  // Track containers and their IDs for edge resolution
  const containerIds = new Set<string>();

  // Add containers in a grid layout
  const cols = Math.ceil(Math.sqrt(system.containers?.length ?? 1));
  system.containers?.forEach((container, index) => {
    const col = index % cols;
    const row = Math.floor(index / cols);
    nodes.push(
      createContainerNode(
        container,
        col * xSpacing,
        yBase + row * 180,
        systemId,
        false // not dimmed
      )
    );
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
    const fromInSystem = nodeIdSet.has(relation.from) || relation.from.startsWith(systemId + ".");
    const toInSystem = nodeIdSet.has(relation.to) || relation.to.startsWith(systemId + ".");

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
    const person = arch.persons?.find((p) => p.id === extId);
    const extSystem = arch.systems?.find((s) => s.id === extId || s.id === extId.split(".")[0]);

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
        type: "system",
        position: { x: externalXBase, y: yPos },
        data: {
          id: extSystem.id,
          label: extSystem.label ?? extSystem.id,
          description: extSystem.description,
          type: "system",
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

  const arch = data.architecture;
  const xSpacing = 200;
  const yBase = 50;
  const addedEdges = new Set<string>();

  // Add components
  container.components?.forEach((component, index) => {
    nodes.push(createComponentNode(component, index * xSpacing, yBase, containerId));
    nodeIdSet.add(component.id);
  });

  // Add datastores (both within container and from system)
  container.datastores?.forEach((ds, index) => {
    nodes.push(createDataStoreNode(ds, index * xSpacing, yBase + 150));
    nodeIdSet.add(ds.id);
  });

  // Also add system-level datastores and queues as potential targets
  system.datastores?.forEach((ds, index) => {
    if (!nodeIdSet.has(ds.id)) {
      nodes.push(createDataStoreNode(ds, index * 150, yBase + 250, true));
      nodeIdSet.add(ds.id);
    }
  });

  system.queues?.forEach((queue, index) => {
    if (!nodeIdSet.has(queue.id)) {
      nodes.push(
        createQueueNode(
          queue,
          (container.datastores?.length ?? 0) * 150 + index * 150,
          yBase + 250,
          true
        )
      );
      nodeIdSet.add(queue.id);
    }
  });

  // Add component-level relations (if any - usually empty)
  container.components?.forEach((component) => {
    component.relations?.forEach((relation) => {
      const edgeId = `${relation.from}-${relation.to}`;
      if (!addedEdges.has(edgeId)) {
        edges.push(createRelationEdge(relation));
        addedEdges.add(edgeId);
      }
    });
  });

  // Add container-level relations (component -> datastore, component -> component)
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

  // Add system-level relations that involve visible components
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

  // Add top-level architecture relations that involve visible components/datastores
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

// Helper to resolve dotted references like "ECommerce.API" to visible node
function resolveNodeId(
  nodeRef: string,
  _arch: ArchitectureJSON["architecture"],
  visibleNodes: Set<string>
): string | null {
  if (visibleNodes.has(nodeRef)) return nodeRef;

  // Fuzzy match: Check if any visible node ID ends with .nodeRef (Case: "WebApp" matches "API.WebApp")
  // or if nodeRef ends with .visibleId (Case: "API.WebApp" matches "WebApp")
  for (const visibleId of visibleNodes) {
    if (visibleId.endsWith("." + nodeRef) || nodeRef.endsWith("." + visibleId)) {
      return visibleId;
    }
  }

  // Try to find the parent system
  const parts = nodeRef.split(".");
  if (parts.length > 1) {
    const systemId = parts[0];
    if (visibleNodes.has(systemId)) return systemId;
  }

  return null;
}

// Node creation helpers
function checkIsExternal(metadata?: { key: string; value?: string }[]): boolean {
  if (!metadata) return false;
  // Check for 'external' key set to 'true'
  const ext = metadata.find((m) => m.key.toLowerCase() === "external");
  if (ext && ext.value?.toLowerCase() === "true") return true;

  // Check for Tags
  const tags = metadata.find((m) => m.key.toLowerCase() === "tags");
  if (tags && tags.value?.toLowerCase().includes("external")) return true;

  // Fallback: Check for Location 'External'
  const loc = metadata.find((m) => m.key.toLowerCase() === "location");
  if (loc && loc.value?.toLowerCase() === "external") return true;

  return false;
}

function createPersonNode(
  person: PersonJSON,
  x: number,
  y: number,
  isExternal: boolean = true
): Node<C4NodeData> {
  // Persons are effectively external by default in C4 unless specified otherwise (e.g. internal user)
  // But for boundary box logic, we want to separate "System under construction" from "Users".
  // Usually Users are OUTSIDE the Enterprise Boundary of the System scope?
  // Actually, "Enterprise Scope" usually includes internal users. "System Scope" excludes them.
  // Let's assume passed isExternal overrides.
  // If not passed, default to TRUE for persons?
  // In C4 "Context", persons are outside the system.
  return {
    id: person.id,
    type: "person",
    position: { x, y },
    data: {
      id: person.id,
      label: person.label ?? person.id,
      description: person.description,
      type: "person",
      isExternal,
    },
  };
}

function createSystemNode(
  system: SystemJSON,
  x: number,
  y: number,
  expanded: boolean,
  isExternal: boolean = false,
  badges: Badge[] = []
): Node<C4NodeData> {
  return {
    id: system.id,
    type: "system",
    position: { x, y },
    data: {
      id: system.id,
      label: system.label ?? system.id,
      description: system.description,
      type: "system",
      childCount: system.containers?.length ?? 0,
      expanded,
      isExternal,
      badges,
    },
  };
}

function createContainerNode(
  container: ContainerJSON,
  x: number,
  y: number,
  parentId: string,
  isExternal: boolean = false,
  expanded: boolean = false,
  useCompoundParent: boolean = false // When true, set parentId at node level for React Flow compound nodes
): Node<C4NodeData> {
  return {
    id: container.id,
    type: "container",
    position: { x, y },
    // Set parentId at node level for React Flow compound nodes (when system is expanded)
    parentId: useCompoundParent ? parentId : undefined,
    extent: useCompoundParent ? ("parent" as const) : undefined,
    expandParent: useCompoundParent ? true : undefined,
    data: {
      id: container.id,
      label: container.label ?? container.id,
      description: container.description,
      technology: container.technology,
      type: "container",
      parentId, // Keep in data for logical reference
      childCount: container.components?.length ?? 0,
      expanded,
      isExternal,
      tags: container.tags ?? [],
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
    type: "component",
    position: { x, y },
    // Note: parentId removed from node level to avoid React Flow compound layout errors
    data: {
      id: component.id,
      label: component.label ?? component.id,
      description: component.description,
      technology: component.technology,
      type: "component",
      parentId, // Keep in data for logical reference
    },
  };
}

function createDataStoreNode(
  ds: DataStoreJSON,
  x: number,
  y: number,
  isExternal: boolean = false
): Node<C4NodeData> {
  return {
    id: ds.id,
    type: "datastore",
    position: { x, y },
    data: {
      id: ds.id,
      label: ds.label ?? ds.id,
      type: "datastore",
      isExternal,
    },
  };
}

function createQueueNode(
  queue: QueueJSON,
  x: number,
  y: number,
  isExternal: boolean = false
): Node<C4NodeData> {
  return {
    id: queue.id,
    type: "queue",
    position: { x, y },
    data: {
      id: queue.id,
      label: queue.label ?? queue.id,
      type: "queue",
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
    type: "relation",
    data: {
      label: relation.label ?? relation.verb,
      technology: relation.technology,
      interaction: relation.interaction,
      points: [], // Reserved for polyline if needed
      tags: relation.tags ?? [],
    },
    animated: false,
  };
}

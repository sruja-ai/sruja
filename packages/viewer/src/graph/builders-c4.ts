/**
 * C4 Level Graph Builders
 * 
 * Generates separate Cytoscape graph data for each C4 level:
 * - L1: System Context (persons + systems)
 * - L2: Container View (system expanded with containers)
 * - L3: Component View (container expanded with components)
 */

import type { ElementDefinition } from 'cytoscape';
import type { ArchitectureJSON } from '../types';


export interface C4GraphData {
    nodes: ElementDefinition[];
    edges: ElementDefinition[];
}

export interface PrecomputedC4Graphs {
    systemContext: C4GraphData;
    containers: Record<string, C4GraphData>;
    components: Record<string, C4GraphData>;
}

/**
 * Build L1 System Context Graph
 * Shows: Persons + Systems (collapsed)
 * Edges: Rolled up to system level
 */
export function buildSystemContextGraph(arch: ArchitectureJSON['architecture']): C4GraphData {
    const nodes: ElementDefinition[] = [];
    const edges: ElementDefinition[] = [];
    const nodeIds = new Set<string>();

    // Add persons
    if (arch.persons) {
        for (const person of arch.persons) {
            nodeIds.add(person.id);
            nodes.push({
                data: { id: person.id, label: person.label || person.id, type: 'person', description: person.description },
            });
        }
    }

    // Add systems (collapsed - no containers/components)
    if (arch.systems) {
        for (const system of arch.systems) {
            nodeIds.add(system.id);
            nodes.push({
                data: { id: system.id, label: system.label || system.id, type: 'system', description: system.description },
            });
        }
    }

    // Add edges - resolve to system level
    if (arch.relations) {
        const addedEdges = new Set<string>();
        for (const relation of arch.relations) {
            const fromResolved = resolveToSystemLevel(relation.from, arch, nodeIds);
            const toResolved = resolveToSystemLevel(relation.to, arch, nodeIds);

            if (fromResolved && toResolved && fromResolved !== toResolved) {
                const edgeKey = `${fromResolved}->${toResolved}`;
                if (!addedEdges.has(edgeKey)) {
                    addedEdges.add(edgeKey);
                    edges.push({
                        data: {
                            id: edgeKey,
                            source: fromResolved,
                            target: toResolved,
                            label: relation.label || relation.verb || '',
                        },
                        group: 'edges',
                    });
                }
            }
        }
    }

    return { nodes, edges };
}

/**
 * Build L2 Container View for a specific system
 * Shows: Target system expanded with containers, other systems collapsed
 * Edges: Between containers and external systems
 */
export function buildContainerGraph(arch: ArchitectureJSON['architecture'], systemId: string): C4GraphData {
    const nodes: ElementDefinition[] = [];
    const edges: ElementDefinition[] = [];
    const nodeIds = new Set<string>();

    // Add persons
    if (arch.persons) {
        for (const person of arch.persons) {
            nodeIds.add(person.id);
            nodes.push({
                data: { id: person.id, label: person.label || person.id, type: 'person' },
            });
        }
    }

    // Add systems
    if (arch.systems) {
        for (const system of arch.systems) {
            nodeIds.add(system.id);

            if (system.id === systemId) {
                // Target system - add as parent node
                nodes.push({
                    data: { id: system.id, label: system.label || system.id, type: 'system' },
                });

                // Add containers as children
                if (system.containers) {
                    for (const container of system.containers) {
                        const containerId = `${system.id}.${container.id}`;
                        nodeIds.add(containerId);
                        nodes.push({
                            data: {
                                id: containerId,
                                label: container.label || container.id,
                                type: 'container',
                                parent: system.id,
                                description: container.description,
                            },
                        });
                    }
                }
                // Add datastores
                if (system.datastores) {
                    for (const ds of system.datastores) {
                        const dsId = `${system.id}.${ds.id}`;
                        nodeIds.add(dsId);
                        nodes.push({
                            data: { id: dsId, label: ds.label || ds.id, type: 'datastore', parent: system.id },
                        });
                    }
                }
                // Add queues
                if (system.queues) {
                    for (const q of system.queues) {
                        const qId = `${system.id}.${q.id}`;
                        nodeIds.add(qId);
                        nodes.push({
                            data: { id: qId, label: q.label || q.id, type: 'queue', parent: system.id },
                        });
                    }
                }
            } else {
                // Other systems - collapsed
                nodes.push({
                    data: { id: system.id, label: system.label || system.id, type: 'system', external: true },
                });
            }
        }
    }

    // Add edges
    if (arch.relations) {
        const addedEdges = new Set<string>();
        for (const relation of arch.relations) {
            const fromResolved = resolveToContainerLevel(relation.from, arch, nodeIds, systemId);
            const toResolved = resolveToContainerLevel(relation.to, arch, nodeIds, systemId);

            if (fromResolved && toResolved && fromResolved !== toResolved) {
                const edgeKey = `${fromResolved}->${toResolved}`;
                if (!addedEdges.has(edgeKey)) {
                    addedEdges.add(edgeKey);
                    edges.push({
                        data: {
                            id: edgeKey,
                            source: fromResolved,
                            target: toResolved,
                            label: relation.label || relation.verb || '',
                        },
                        group: 'edges',
                    });
                }
            }
        }
    }

    return { nodes, edges };
}

/**
 * Build L3 Component View for a specific container
 * Shows: Target container expanded with components, other containers collapsed
 */
export function buildComponentGraph(arch: ArchitectureJSON['architecture'], systemId: string, containerId: string): C4GraphData {
    const nodes: ElementDefinition[] = [];
    const edges: ElementDefinition[] = [];
    const nodeIds = new Set<string>();
    const fullContainerId = `${systemId}.${containerId}`;

    // Find target system and container
    const targetSystem = arch.systems?.find(s => s.id === systemId);
    if (!targetSystem) return { nodes, edges };

    const targetContainer = targetSystem.containers?.find(c => c.id === containerId);
    if (!targetContainer) return { nodes, edges };

    // Add container as parent
    nodeIds.add(fullContainerId);
    nodes.push({
        data: { id: fullContainerId, label: targetContainer.label || containerId, type: 'container' },
    });

    // Add components
    if (targetContainer.components) {
        for (const component of targetContainer.components) {
            const compId = `${fullContainerId}.${component.id}`;
            nodeIds.add(compId);
            nodes.push({
                data: {
                    id: compId,
                    label: component.label || component.id,
                    type: 'component',
                    parent: fullContainerId,
                    description: component.description,
                },
            });
        }
    }

    // Add connected external elements (other containers, datastores, etc.)
    if (arch.relations) {
        for (const relation of arch.relations) {
            const from = relation.from;
            const to = relation.to;

            // Check if relation involves this container's components
            const fromIsInternal = from.startsWith(`${fullContainerId}.`) || from === fullContainerId;
            const toIsInternal = to.startsWith(`${fullContainerId}.`) || to === fullContainerId;

            if (fromIsInternal || toIsInternal) {
                // Add external node if needed
                const externalId = fromIsInternal ? to : from;
                if (!nodeIds.has(externalId)) {
                    const externalType = inferNodeType(externalId, arch);
                    nodeIds.add(externalId);
                    nodes.push({
                        data: { id: externalId, label: getNodeLabel(externalId, arch), type: externalType, external: true },
                    });
                }

                // Add edge
                const fromResolved = resolveId(from, nodeIds);
                const toResolved = resolveId(to, nodeIds);
                if (fromResolved && toResolved) {
                    edges.push({
                        data: {
                            id: `${fromResolved}->${toResolved}`,
                            source: fromResolved,
                            target: toResolved,
                            label: relation.label || relation.verb || '',
                        },
                        group: 'edges',
                    });
                }
            }
        }
    }

    return { nodes, edges };
}

/**
 * Pre-compute all C4 graphs for an architecture
 */
export function precomputeC4Graphs(arch: ArchitectureJSON['architecture']): PrecomputedC4Graphs {
    const systemContext = buildSystemContextGraph(arch);
    const containers: Record<string, C4GraphData> = {};
    const components: Record<string, C4GraphData> = {};

    if (arch.systems) {
        for (const system of arch.systems) {
            containers[system.id] = buildContainerGraph(arch, system.id);

            if (system.containers) {
                for (const container of system.containers) {
                    const key = `${system.id}.${container.id}`;
                    components[key] = buildComponentGraph(arch, system.id, container.id);
                }
            }
        }
    }

    return { systemContext, containers, components };
}

// Helper functions
function resolveToSystemLevel(id: string, arch: ArchitectureJSON['architecture'], nodeIds: Set<string>): string | null {
    if (nodeIds.has(id)) return id;

    // Check if it's a container/component - resolve to parent system
    const parts = id.split('.');
    if (parts.length > 1 && nodeIds.has(parts[0])) {
        return parts[0];
    }

    // Search in systems
    if (arch.systems) {
        for (const system of arch.systems) {
            if (system.containers?.some(c => c.id === id || `${system.id}.${c.id}` === id)) {
                return system.id;
            }
        }
    }

    return null;
}

function resolveToContainerLevel(id: string, arch: ArchitectureJSON['architecture'], nodeIds: Set<string>, focusSystemId: string): string | null {
    if (nodeIds.has(id)) return id;

    const parts = id.split('.');

    // If it's a component, resolve to container
    if (parts.length === 3 && parts[0] === focusSystemId) {
        const containerId = `${parts[0]}.${parts[1]}`;
        if (nodeIds.has(containerId)) return containerId;
    }

    // If it's outside focus system, resolve to system
    if (parts.length > 1 && parts[0] !== focusSystemId) {
        if (nodeIds.has(parts[0])) return parts[0];
    }

    // Search in target system
    const system = arch.systems?.find(s => s.id === focusSystemId);
    if (system?.containers?.some(c => c.id === id)) {
        return `${focusSystemId}.${id}`;
    }

    return null;
}

function resolveId(id: string, nodeIds: Set<string>): string | null {
    if (nodeIds.has(id)) return id;
    return null;
}

function inferNodeType(id: string, arch: ArchitectureJSON['architecture']): string {
    if (arch.persons?.some(p => p.id === id)) return 'person';
    if (arch.systems?.some(s => s.id === id)) return 'system';

    const parts = id.split('.');
    if (parts.length === 2) {
        const system = arch.systems?.find(s => s.id === parts[0]);
        if (system?.containers?.some(c => c.id === parts[1])) return 'container';
        if (system?.datastores?.some(d => d.id === parts[1])) return 'datastore';
        if (system?.queues?.some(q => q.id === parts[1])) return 'queue';
    }
    if (parts.length === 3) return 'component';

    return 'system';
}

function getNodeLabel(id: string, arch: ArchitectureJSON['architecture']): string {
    const parts = id.split('.');
    const lastPart = parts[parts.length - 1];

    // Try to find proper label in architecture
    if (arch.persons) {
        const person = arch.persons.find(p => p.id === id);
        if (person) return person.label || person.id;
    }
    if (arch.systems) {
        for (const system of arch.systems) {
            if (system.id === id) return system.label || system.id;
            if (system.containers) {
                for (const container of system.containers) {
                    if (`${system.id}.${container.id}` === id) return container.label || container.id;
                    if (container.components) {
                        for (const comp of container.components) {
                            if (`${system.id}.${container.id}.${comp.id}` === id) return comp.label || comp.id;
                        }
                    }
                }
            }
        }
    }

    return lastPart;
}

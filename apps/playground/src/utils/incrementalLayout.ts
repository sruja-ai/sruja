// Incremental layout wrapper that preserves positions for stable nodes
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../types';
import { PositionPreservation } from './positionPreservation';
import { applySrujaLayout } from './layoutEngine';
import { applyC4LevelLayout } from './c4LevelLayout';
import { selectLayoutConfig } from './layoutRules';

interface IncrementalLayoutOptions {
    layoutEngine: 'sruja' | 'c4level'; // Only our custom engines
    direction?: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT';
    level?: string;
    focusedSystemId?: string;
    focusedContainerId?: string;
    expandedNodes?: Set<string>;
    previousExpandedNodes?: Set<string>;
}

/**
 * Apply layout with position preservation for stable nodes
 */
export async function applyIncrementalLayout(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    options: IncrementalLayoutOptions,
    positionPreservation: PositionPreservation
): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    const {
        layoutEngine,
        direction = 'DOWN',
        level,
        focusedSystemId,
        focusedContainerId,
        expandedNodes = new Set(),
        previousExpandedNodes = new Set()
    } = options;

    // Detect which nodes changed
    const changes = positionPreservation.detectChanges(
        nodes,
        previousExpandedNodes,
        expandedNodes
    );

    // Check for major structural changes (e.g., different number of root nodes)
    const rootNodes = nodes.filter(n => !n.parentId);
    const previousRootCount = positionPreservation.getPreviousRootNodeCount();

    const isMajorStructuralChange =
        rootNodes.length !== previousRootCount ||
        changes.newlyVisibleNodeIds.size > nodes.length * 0.6; // More than 60% new nodes

    // If no significant changes, preserve all positions
    if (!isMajorStructuralChange &&
        changes.changedNodeIds.size === 0 &&
        changes.newlyVisibleNodeIds.size === 0) {
        return {
            nodes: positionPreservation.applyPreservedPositions(nodes, changes.stableNodeIds),
            edges
        };
    }

    // If major structural change, fall back to full layout
    if (isMajorStructuralChange) {
        // Use rules-based selection for full re-layout
        const config = selectLayoutConfig(
            nodes,
            edges,
            options.level as any,
            options.focusedSystemId,
            options.focusedContainerId,
            options.expandedNodes
        );

        if (config.engine === 'c4level') {
            return applyC4LevelLayout(nodes, edges, {
                level: options.level as any,
                focusedSystemId: options.focusedSystemId,
                focusedContainerId: options.focusedContainerId,
                expandedNodes: options.expandedNodes
            });
        } else {
            return applySrujaLayout(nodes, edges, {
                direction: config.direction,
                level: options.level as any,
                expandedNodes: options.expandedNodes,
                ...config.options
            });
        }
    }

    // For now, we'll use a hybrid approach:
    // 1. If only a few nodes changed, try to preserve positions of stable nodes
    // 2. If many nodes changed or it's a major structural change, do full re-layout
    //    but try to keep stable nodes close to their previous positions

    // Check if this is a minor change (just expansion/collapse of one node)
    // Minor change means:
    // - Only a few nodes changed state
    // - Newly visible nodes are children of a single parent (expansion)
    // - Not too many new nodes appearing
    // Relaxed thresholds to catch more expansion scenarios
    const isMinorChange =
        changes.changedNodeIds.size <= 20 && // Increased from 10 to catch more cases
        changes.newlyVisibleNodeIds.size > 0 &&
        changes.newlyVisibleNodeIds.size <= 30 && // Increased from 15
        changes.stableNodeIds.size > changes.changedNodeIds.size * 0.5; // Relaxed: at least 50% stable (was 100%)

    // Use rules to determine if we should use incremental layout
    const config = selectLayoutConfig(
        nodes,
        edges,
        level as any,
        undefined,
        undefined,
        expandedNodes
    );

    if (isMinorChange && config.engine === 'sruja') {
        // For minor changes with sruja layout, we can try to preserve positions
        return applyIncrementalSrujaLayout(
            nodes,
            edges,
            { direction: config.direction, level, expandedNodes },
            positionPreservation,
            changes
        );
    }

    // For major changes or other layout engines, do full re-layout
    // but we'll still try to preserve positions where possible

    // Check for node expansion to apply neighbor shifting
    // This provides a much smoother "expand in place" experience
    // Support both single and multiple expansions (one at a time, but allow multiple in sequence)
    const newlyExpanded = Array.from(expandedNodes).filter(id => !previousExpandedNodes.has(id));

    const shouldUseIncremental = true;

    console.log('[NeighborShifting] Checks:', {
        newlyExpandedCount: newlyExpanded.length,
        isMajorStructuralChange,
        layoutEngine,
        shouldUseIncremental
    });

    // Use neighbor shifting for single node expansion, or if we're already using incremental
    // and there's a small number of expansions
    if (newlyExpanded.length > 0 && newlyExpanded.length <= 3 && !isMajorStructuralChange) {
        // For multiple expansions, try the first one (can be improved to handle all)
        const expandedNodeId = newlyExpanded[0];
        try {
            console.log('[NeighborShifting] Attempting shifting for:', expandedNodeId);
            const result = await applyNeighborShifting(
                nodes,
                edges,
                expandedNodeId,
                positionPreservation,
                { direction, level: level as any }
            );
            // If successful, recursively handle remaining expansions
            if (newlyExpanded.length > 1 && shouldUseIncremental) {
                // Update position preservation with new positions
                positionPreservation.updateFromNodes(result.nodes);
                // Recursively handle next expansion
                return applyIncrementalLayout(
                    result.nodes,
                    result.edges,
                    {
                        ...options,
                        previousExpandedNodes: new Set([...Array.from(previousExpandedNodes), expandedNodeId])
                    },
                    positionPreservation
                );
            }
            return result;
        } catch (e) {
            console.warn('[NeighborShifting] Failed, falling back to full layout', e);
            // Fall through to full layout
        }
    }

    let result: { nodes: Node<C4NodeData>[]; edges: Edge[] };

    if (layoutEngine === 'c4level') {
        result = await applyC4LevelLayout(nodes, edges, {
            level: level as any,
            focusedSystemId,
            focusedContainerId,
            expandedNodes
        });
    } else {
        result = await applySrujaLayout(nodes, edges, {
            direction,
            level: level as any,
            expandedNodes
        });
    }

    // After layout, try to preserve positions of stable nodes that weren't affected
    // by the change. We use a conservative preservation strategy:
    // - Only preserve root-level nodes (not children) to avoid parent-child relationship issues
    // - If a stable root node moved more than threshold, preserve its original position
    // Increased threshold to be more lenient and preserve more positions during expansion
    const MOVEMENT_THRESHOLD = 50; // Increased from 30 to preserve more positions

    const preservedNodes = result.nodes.map(node => {
        // Only preserve root-level stable nodes (no parentId)
        // Child nodes will move with their parents, which is correct behavior
        if (changes.stableNodeIds.has(node.id) && !node.parentId) {
            const preserved = positionPreservation.getPreservedPosition(node.id);
            if (preserved) {
                // Calculate distance moved
                const distance = Math.sqrt(
                    Math.pow(node.position.x - preserved.x, 2) +
                    Math.pow(node.position.y - preserved.y, 2)
                );

                // If node moved significantly, preserve original position
                // This prevents unnecessary movement when only a small part of the graph changed
                if (distance > MOVEMENT_THRESHOLD) {
                    return {
                        ...node,
                        position: preserved
                    };
                }
            }
        }
        return node;
    });

    return {
        nodes: preservedNodes,
        edges: result.edges
    };
}

/**
 * Apply incremental layout for Sruja engine with position preservation
 */
async function applyIncrementalSrujaLayout(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    options: { direction?: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT'; level?: string; expandedNodes?: Set<string> },
    positionPreservation: PositionPreservation,
    changes: {
        changedNodeIds: Set<string>;
        newlyVisibleNodeIds: Set<string>;
        stableNodeIds: Set<string>;
    }
): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    // For incremental layout, we want to:
    // 1. Keep stable nodes in their current positions
    // 2. Only layout the changed/newly visible nodes relative to their parents

    // First, preserve positions of stable nodes
    const nodesWithPreservedPositions = positionPreservation.applyPreservedPositions(
        nodes,
        changes.stableNodeIds
    );

    // For newly visible nodes (children of expanded nodes), we need to layout them
    // within their parent's bounds. We'll do a partial layout just for the affected subgraph

    // Get all nodes that need layout (changed + newly visible)
    const nodesNeedingLayout = new Set([
        ...changes.changedNodeIds,
        ...changes.newlyVisibleNodeIds
    ]);

    // Also include parents of newly visible nodes (they might need to resize)
    nodes.forEach(node => {
        if (node.parentId && nodesNeedingLayout.has(node.parentId)) {
            nodesNeedingLayout.add(node.id);
        }
    });

    // Extract subgraph for layout
    const subgraphNodes = nodes.filter(n => nodesNeedingLayout.has(n.id));
    const subgraphEdges = edges.filter(e =>
        nodesNeedingLayout.has(e.source) && nodesNeedingLayout.has(e.target)
    );

    // If we have a small subgraph, layout just that part
    if (subgraphNodes.length > 0 && subgraphNodes.length < nodes.length * 0.5) {
        // Layout the subgraph
        const subgraphResult = await applySrujaLayout(subgraphNodes, subgraphEdges, {
            direction: options.direction || 'DOWN',
            level: options.level as any,
            expandedNodes: options.expandedNodes
        });

        // Merge subgraph layout results with preserved positions
        const subgraphNodeMap = new Map(subgraphResult.nodes.map(n => [n.id, n]));

        const mergedNodes = nodesWithPreservedPositions.map(node => {
            const subgraphNode = subgraphNodeMap.get(node.id);
            if (subgraphNode) {
                return subgraphNode;
            }
            return node;
        });

        return {
            nodes: mergedNodes,
            edges
        };
    }

    // If subgraph is too large, fall back to full layout
    return applySrujaLayout(nodes, edges, {
        direction: options.direction || 'DOWN',
        level: options.level as any,
        expandedNodes: options.expandedNodes
    });
}

/**
 * Smart Neighbor Shifting:
 * 1. Layouts ONLY the contents of the expanded node
 * 2. Resizes the expanded node
 * 3. Shifts surrounding nodes to make space
 */
async function applyNeighborShifting(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    expandedNodeId: string,
    positionPreservation: PositionPreservation,
    options: { direction?: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT'; level?: string }
): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    const { direction = 'DOWN' } = options;

    // 1. Identify children of the expanded node
    const children = nodes.filter(n => n.parentId === expandedNodeId);

    // If no children, nothing to layout inside
    if (children.length === 0) {
        return { nodes, edges };
    }

    // 2. Layout the children strictly (subgraph layout)
    // We create a temporary graph of just the children to find their optimal arrangement
    const childEdges = edges.filter(e =>
        children.some(c => c.id === e.source) &&
        children.some(c => c.id === e.target)
    );

    // Use Sruja layout for the internals
    const internalLayout = await applySrujaLayout(children, childEdges, {
        direction,
        level: options.level as any,
        // vital: don't recursively expand for this calculation if we want shallow size
        expandedNodes: new Set()
    });

    // 3. Calculate new size of the expanded parent
    const PADDING = 80; // Increased padding for better containment (was 60)
    let minChildX = Infinity, maxChildX = -Infinity;
    let minChildY = Infinity, maxChildY = -Infinity;

    internalLayout.nodes.forEach(n => {
        // Layout returns relative positions if we passed them as such, or absolute?
        // applySrujaLayout usually assumes root if no parent provided in context, 
        // but here they HAVE parents.
        // However, we want their relative arrangement.
        // Let's assume the internalLayout gave us positions relative to their common origin.

        // We'll normalize them to start at (0,0) + padding
        minChildX = Math.min(minChildX, n.position.x);
        minChildY = Math.min(minChildY, n.position.y);
        maxChildX = Math.max(maxChildX, n.position.x + (n.width || 0));
        maxChildY = Math.max(maxChildY, n.position.y + (n.height || 0));
    });

    const contentWidth = maxChildX - minChildX;
    const contentHeight = maxChildY - minChildY;

    // Use minimum sizes that match layout engine settings
    const newWidth = Math.max(220, contentWidth + PADDING * 2); // Match minSize from layoutEngine
    const newHeight = Math.max(100, contentHeight + PADDING * 2);

    // 4. Get previous position of the expanded node
    const preserved = positionPreservation.getPreservedPosition(expandedNodeId);
    if (!preserved) {
        throw new Error("No preserved position for expanded node");
    }

    // Get the OLD size from preserved node state if available, otherwise use approximate
    const preservedNode = nodes.find(n => n.id === expandedNodeId);
    const OLD_WIDTH = preservedNode?.width || 220;  // Match minSize from layoutEngine
    const OLD_HEIGHT = preservedNode?.height || 100;

    const deltaW = newWidth - OLD_WIDTH;
    const deltaH = newHeight - OLD_HEIGHT;

    // 5. Shift Neighbors
    // We shift everything that is "to the right" or "below"

    // Get all root nodes (level 0 relative to canvas)
    const rootNodes = nodes.filter(n => !n.parentId);
    // Preserved positions for all roots
    const rootPositions = new Map<string, { x: number, y: number }>();

    rootNodes.forEach(n => {
        const p = positionPreservation.getPreservedPosition(n.id);
        if (p) rootPositions.set(n.id, p);
        else rootPositions.set(n.id, n.position);
    });

    const expandedPos = rootPositions.get(expandedNodeId)!;

    const shiftedNodes = nodes.map(node => {
        // If it's the expanded node, update size
        if (node.id === expandedNodeId) {
            return {
                ...node,
                width: newWidth,
                height: newHeight,
                style: { ...node.style, width: newWidth, height: newHeight },
                position: expandedPos // Keep top-left constant
            };
        }

        // If it's a child of the expanded node, update position from internal layout
        if (node.parentId === expandedNodeId) {
            const internalNode = internalLayout.nodes.find(n => n.id === node.id);
            if (internalNode) {
                // Normalize position relative to parent
                return {
                    ...node,
                    position: {
                        x: internalNode.position.x - minChildX + PADDING,
                        y: internalNode.position.y - minChildY + PADDING
                    },
                    width: internalNode.width,
                    height: internalNode.height,
                    style: { ...node.style, width: internalNode.width, height: internalNode.height }
                };
            }
        }

        // If it's another root node, shift it
        if (!node.parentId && node.id !== expandedNodeId) {
            const pos = rootPositions.get(node.id)!;
            let { x, y } = pos;

            // Simple heuristic for Grid/Flow layout

            // 1. Shift vertical (nodes below)
            if (pos.y > expandedPos.y + OLD_HEIGHT * 0.5) {
                y += deltaH;
            }

            // 2. Shift horizontal (nodes to the right)
            // Only shift if they are relatively in the same "row"
            // i.e., their Y overlaps with the expanded node's Y range
            const isRowOverlap = (pos.y + 100 > expandedPos.y) && (pos.y < expandedPos.y + newHeight);

            if (pos.x > expandedPos.x + OLD_WIDTH * 0.5 && isRowOverlap) {
                x += deltaW;
            }

            return {
                ...node,
                position: { x, y }
            };
        }

        // Children of other nodes - they move with their parents automatically
        // because their position is relative
        return node;
    });

    // 6. Basic edge routing update? 
    // We return original edges; React Flow will update paths based on handles.
    // However, we should try to update our custom edge points if possible.
    // For neighbor shift, re-routing is hard without full layout.
    // We'll strip specific points to let React Flow / simple bezier take over temporarily
    // or keep them if they are relative?
    // Using filtered edges (resetting points) is safer to avoid disconnected lines.
    const resetEdges = edges.map(e => ({
        ...e,
        data: {
            ...e.data,
            points: undefined // Clear points to force simple routing
        }
    }));

    return {
        nodes: shiftedNodes,
        edges: resetEdges
    };
}

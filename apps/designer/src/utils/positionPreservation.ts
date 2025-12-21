// Position preservation utility for stable layouts
import type { Node } from "@xyflow/react";
import type { C4NodeData } from "../types";

interface NodeState {
  id: string;
  position: { x: number; y: number };
  visible: boolean;
  expanded: boolean;
  parentId?: string;
}

/**
 * Tracks node positions and visibility states to preserve layout stability
 */
export class PositionPreservation {
  private previousStates: Map<string, NodeState> = new Map();

  /**
   * Update state from current nodes
   */
  updateFromNodes(nodes: Node<C4NodeData>[]): void {
    const currentStates = new Map<string, NodeState>();

    nodes.forEach((node) => {
      currentStates.set(node.id, {
        id: node.id,
        position: { ...node.position },
        visible: true,
        expanded: node.data.expanded ?? false,
        parentId: node.parentId,
      });
    });

    this.previousStates = currentStates;
  }

  /**
   * Get preserved position for a node if it hasn't changed state
   */
  getPreservedPosition(nodeId: string): { x: number; y: number } | null {
    const previous = this.previousStates.get(nodeId);
    return previous ? { ...previous.position } : null;
  }

  /**
   * Detect which nodes changed visibility or expansion state
   */
  detectChanges(
    currentNodes: Node<C4NodeData>[],
    previousExpandedNodes: Set<string>,
    currentExpandedNodes: Set<string>
  ): {
    changedNodeIds: Set<string>;
    newlyVisibleNodeIds: Set<string>;
    newlyHiddenNodeIds: Set<string>;
    stableNodeIds: Set<string>;
  } {
    const changedNodeIds = new Set<string>();
    const newlyVisibleNodeIds = new Set<string>();
    const newlyHiddenNodeIds = new Set<string>();
    const stableNodeIds = new Set<string>();

    const currentNodeIds = new Set(currentNodes.map((n) => n.id));
    const previousNodeIds = new Set(this.previousStates.keys());

    // Find newly visible nodes (children of expanded nodes)
    currentNodes.forEach((node) => {
      const wasVisible = previousNodeIds.has(node.id);
      const isVisible = currentNodeIds.has(node.id);
      const wasExpanded = previousExpandedNodes.has(node.id);
      const isExpanded = currentExpandedNodes.has(node.id);

      if (!wasVisible && isVisible) {
        newlyVisibleNodeIds.add(node.id);
        changedNodeIds.add(node.id);
      } else if (wasVisible && !isVisible) {
        newlyHiddenNodeIds.add(node.id);
        changedNodeIds.add(node.id);
      } else if (wasExpanded !== isExpanded) {
        // Node expansion state changed
        changedNodeIds.add(node.id);
        // Also mark children as changed
        currentNodes
          .filter((n) => n.parentId === node.id)
          .forEach((child) => changedNodeIds.add(child.id));
      } else if (wasVisible && isVisible) {
        // Check if parent changed (affects relative positioning)
        const previous = this.previousStates.get(node.id);
        if (previous && previous.parentId !== node.parentId) {
          changedNodeIds.add(node.id);
        } else {
          stableNodeIds.add(node.id);
        }
      }
    });

    return {
      changedNodeIds,
      newlyVisibleNodeIds,
      newlyHiddenNodeIds,
      stableNodeIds,
    };
  }

  /**
   * Apply preserved positions to stable nodes
   */
  applyPreservedPositions<T extends Node<C4NodeData>>(nodes: T[], stableNodeIds: Set<string>): T[] {
    return nodes.map((node) => {
      if (stableNodeIds.has(node.id)) {
        const preserved = this.getPreservedPosition(node.id);
        if (preserved) {
          return {
            ...node,
            position: preserved,
          };
        }
      }
      return node;
    });
  }

  /**
   * Clear all preserved positions (for full re-layout)
   */
  clear(): void {
    this.previousStates.clear();
  }

  /**
   * Get the count of root nodes (nodes without parentId) in previous state
   */
  getPreviousRootNodeCount(): number {
    return Array.from(this.previousStates.values()).filter((s) => !s.parentId).length;
  }
}

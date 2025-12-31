/**
 * Visual Effects System
 *
 * Handles visual effects for nodes and edges during animation.
 * Applies CSS classes and manages DOM element state.
 */

import type { NodeState, EdgeState } from "./AnimationController";

export class VisualEffectsSystem {
  private nodeElements: Map<string, HTMLElement[]> = new Map();
  private edgeElements: Map<string, HTMLElement[]> = new Map();
  private container: HTMLElement | null = null;

  /**
   * Initialize the visual effects system with a container element
   */
  initialize(container: HTMLElement): void {
    this.container = container;
    this.scanElements();
  }

  /**
   * Scan container for diagram elements
   */
  scanElements(): void {
    if (!this.container) {
      return;
    }

    this.nodeElements.clear();
    this.edgeElements.clear();

    // Find all node elements (nodes are rendered with data attributes)
    // Common patterns: [data-element-id], [data-node-id], g[data-id]
    const nodeSelectors = ["[data-element-id]", "[data-node-id]", "g[data-id]", ".sruja-node"];

    nodeSelectors.forEach((selector) => {
      const elements = this.container!.querySelectorAll<HTMLElement>(selector);
      elements.forEach((el) => {
        const nodeId = this.extractNodeId(el);
        if (nodeId) {
          if (!this.nodeElements.has(nodeId)) {
            this.nodeElements.set(nodeId, []);
          }
          this.nodeElements.get(nodeId)!.push(el);
        }
      });
    });

    // Find all edge elements (edges are rendered as paths or lines)
    // Common patterns: path elements with data attributes, or lines connecting nodes
    const edgeSelectors = ["path[data-edge-id]", "path[data-relation-id]", ".sruja-edge"];

    edgeSelectors.forEach((selector) => {
      const elements = this.container!.querySelectorAll<HTMLElement>(selector);
      elements.forEach((el) => {
        const edgeId = this.extractEdgeId(el);
        if (edgeId) {
          if (!this.edgeElements.has(edgeId)) {
            this.edgeElements.set(edgeId, []);
          }
          this.edgeElements.get(edgeId)!.push(el);
        }
      });
    });

    // Also try to find edges by source/target attributes
    const pathElements = this.container!.querySelectorAll<HTMLElement>("path");
    pathElements.forEach((path) => {
      const source = path.getAttribute("data-source") || path.getAttribute("source");
      const target = path.getAttribute("data-target") || path.getAttribute("target");
      if (source && target) {
        const edgeId = `${source}->${target}`;
        if (!this.edgeElements.has(edgeId)) {
          this.edgeElements.set(edgeId, []);
        }
        this.edgeElements.get(edgeId)!.push(path);
      }
    });
  }

  /**
   * Highlight a node with the given state
   */
  highlightNode(nodeId: string, state: NodeState): void {
    const elements = this.nodeElements.get(nodeId);
    if (!elements) {
      return;
    }

    // Remove all animation classes first
    elements.forEach((el) => {
      this.removeNodeClasses(el);
      el.classList.add(`animation-node-${state}`);
    });
  }

  /**
   * Clear highlight from a node
   */
  clearNodeHighlight(nodeId: string): void {
    const elements = this.nodeElements.get(nodeId);
    if (!elements) {
      return;
    }

    elements.forEach((el) => {
      this.removeNodeClasses(el);
      el.classList.add("animation-node-idle");
    });
  }

  /**
   * Highlight an edge with the given state
   */
  highlightEdge(edgeId: string, state: EdgeState): void {
    const elements = this.edgeElements.get(edgeId);
    if (!elements) {
      return;
    }

    elements.forEach((el) => {
      this.removeEdgeClasses(el);
      el.classList.add(`animation-edge-${state}`);
    });
  }

  /**
   * Animate flow along an edge
   */
  animateEdgeFlow(edgeId: string, direction: "forward" | "backward"): void {
    const elements = this.edgeElements.get(edgeId);
    if (!elements) {
      return;
    }

    elements.forEach((el) => {
      this.removeEdgeClasses(el);
      el.classList.add(`animation-edge-active`);
      el.classList.add(`animation-edge-flow-${direction}`);
    });
  }

  /**
   * Clear highlight from an edge
   */
  clearEdgeHighlight(edgeId: string): void {
    const elements = this.edgeElements.get(edgeId);
    if (!elements) {
      return;
    }

    elements.forEach((el) => {
      this.removeEdgeClasses(el);
      el.classList.add("animation-edge-idle");
    });
  }

  /**
   * Update visuals for a step
   */
  updateStepVisuals(
    activeNodes: Set<string>,
    activeEdges: Set<string>,
    visitedNodes: Set<string>,
    visitedEdges: Set<string>
  ): void {
    // Clear all first
    this.clearAllVisuals();

    // Apply visited state to previously visited elements
    visitedNodes.forEach((nodeId) => {
      if (!activeNodes.has(nodeId)) {
        this.highlightNode(nodeId, "visited");
      }
    });

    visitedEdges.forEach((edgeId) => {
      if (!activeEdges.has(edgeId)) {
        this.highlightEdge(edgeId, "visited");
      }
    });

    // Apply active/highlighted state to current step elements
    activeNodes.forEach((nodeId) => {
      this.highlightNode(nodeId, "highlighted");
    });

    activeEdges.forEach((edgeId) => {
      this.highlightEdge(edgeId, "highlighted");
      this.animateEdgeFlow(edgeId, "forward");
    });
  }

  /**
   * Clear all visual effects
   */
  clearAllVisuals(): void {
    if (!this.container) {
      return;
    }

    // Remove all animation classes from all elements
    const allElements = this.container.querySelectorAll<HTMLElement>(
      "[class*='animation-node-'], [class*='animation-edge-']"
    );

    allElements.forEach((el) => {
      this.removeNodeClasses(el);
      this.removeEdgeClasses(el);
      el.classList.add("animation-node-idle", "animation-edge-idle");
    });
  }

  /**
   * Reset the system
   */
  reset(): void {
    this.clearAllVisuals();
    this.nodeElements.clear();
    this.edgeElements.clear();
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.reset();
    this.container = null;
  }

  // Private helper methods

  private extractNodeId(element: HTMLElement): string | null {
    return (
      element.getAttribute("data-element-id") ||
      element.getAttribute("data-node-id") ||
      element.getAttribute("data-id") ||
      element.getAttribute("id") ||
      null
    );
  }

  private extractEdgeId(element: HTMLElement): string | null {
    return (
      element.getAttribute("data-edge-id") ||
      element.getAttribute("data-relation-id") ||
      element.getAttribute("data-id") ||
      null
    );
  }

  private removeNodeClasses(element: HTMLElement): void {
    const classesToRemove = [
      "animation-node-idle",
      "animation-node-active",
      "animation-node-highlighted",
      "animation-node-visited",
      "animation-node-pending",
    ];
    classesToRemove.forEach((cls) => element.classList.remove(cls));
  }

  private removeEdgeClasses(element: HTMLElement): void {
    const classesToRemove = [
      "animation-edge-idle",
      "animation-edge-active",
      "animation-edge-highlighted",
      "animation-edge-visited",
      "animation-edge-flow-forward",
      "animation-edge-flow-backward",
    ];
    classesToRemove.forEach((cls) => element.classList.remove(cls));
  }
}

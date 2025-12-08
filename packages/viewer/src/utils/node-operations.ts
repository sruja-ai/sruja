// packages/viewer/src/utils/node-operations.ts
import type { Core } from 'cytoscape';

const ANIMATION_DURATION_MS = 500;
const FOCUS_DIMMED_OPACITY = 0.25;
const FOCUS_FULL_OPACITY = 1;

/**
 * Collapse a node and hide its descendants
 */
export function collapseNode(cy: Core, nodeId: string): void {
  const node = cy.getElementById(nodeId);
  if (node.length === 0) return;

  node.data('collapsed', true);
  node.addClass('collapsed');
  node.descendants().style('display', 'none');
}

/**
 * Expand a node and show its descendants
 */
export function expandNode(cy: Core, nodeId: string): void {
  const node = cy.getElementById(nodeId);
  if (node.length === 0) return;

  node.data('collapsed', false);
  node.removeClass('collapsed');
  node.descendants().style('display', 'element');
}

/**
 * Collapse all nodes of a specific type
 */
export function collapseNodesByType(cy: Core, nodeType: string): void {
  const nodes = cy.$(`node[type="${nodeType}"]`);
  nodes.forEach(node => {
    collapseNode(cy, node.id());
  });
}

/**
 * Expand all nodes of a specific type
 */
export function expandNodesByType(cy: Core, nodeType: string): void {
  const nodes = cy.$(`node[type="${nodeType}"]`);
  nodes.forEach(node => {
    expandNode(cy, node.id());
  });
}

/**
 * Check if a node ID is within the focus scope
 */
export function isNodeInFocus(
  nodeId: string,
  focus: { systemId?: string; containerId?: string }
): boolean {
  if (focus.containerId) {
    // Expect qualified id: system.container or system.container.component
    return nodeId.startsWith(focus.containerId + '.') || nodeId === focus.containerId;
  }
  if (focus.systemId) {
    // System or anything under it
    return nodeId === focus.systemId || nodeId.startsWith(focus.systemId + '.');
  }
  return false;
}

/**
 * Apply focus dimming to nodes and edges
 */
export function applyFocusDimming(
  cy: Core,
  focus: { systemId?: string; containerId?: string }
): void {
  // Reset all opacity first
  cy.elements().style('opacity', FOCUS_FULL_OPACITY);

  if (!focus || (!focus.systemId && !focus.containerId)) {
    return; // no focus → show all normally
  }

  // Dim non-focused nodes
  cy.nodes().forEach(n => {
    const id = n.id();
    const opacity = isNodeInFocus(id, focus) ? FOCUS_FULL_OPACITY : FOCUS_DIMMED_OPACITY;
    n.style('opacity', opacity);
  });

  // Dim edges where both ends are out of focus
  cy.edges().forEach(e => {
    const src = e.source().id();
    const tgt = e.target().id();
    const srcIn = isNodeInFocus(src, focus);
    const tgtIn = isNodeInFocus(tgt, focus);
    const opacity = (srcIn || tgtIn) ? FOCUS_FULL_OPACITY : FOCUS_DIMMED_OPACITY;
    e.style('opacity', opacity);
  });
}

/**
 * Get the selected node or node by ID
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getNode(cy: Core, id?: string): any {
  if (id) {
    return cy.getElementById(id);
  }
  const selected = cy.$('node:selected');
  return selected.length === 1 ? selected[0] : null;
}

/**
 * Animation duration constant
 */
export { ANIMATION_DURATION_MS };




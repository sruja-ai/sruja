// packages/viewer/src/events.ts
import type { Core, NodeSingular, EventObject } from 'cytoscape';
import type { MetadataJSON } from './types';

const GRID_SIZE = 24;

/**
 * Snap position to grid
 */
function snapToGrid(pos: { x: number; y: number }): { x: number; y: number } {
  return {
    x: Math.round(pos.x / GRID_SIZE) * GRID_SIZE,
    y: Math.round(pos.y / GRID_SIZE) * GRID_SIZE
  };
}

/**
 * Event handlers configuration
 */
export interface EventHandlers {
  onSelect?: (id: string | null) => void;
  onDragFree?: (nodeId: string, pos: { x: number; y: number }) => void;
}

/**
 * Initialize event handlers for the viewer
 */
export function initEvents(
  cy: Core,
  handlers: EventHandlers,
  data?: { metadata?: MetadataJSON }
): () => void {
  // Selection handlers
  const selectHandler = (e: EventObject) => {
    const node = e.target as NodeSingular;
    if (handlers.onSelect) {
      handlers.onSelect(node.id());
    }
  };

  const unselectHandler = () => {
    const selected = cy.$('node:selected');
    if (selected.length === 0 && handlers.onSelect) {
      handlers.onSelect(null);
    }
  };

  cy.on('select', 'node', selectHandler);
  cy.on('unselect', 'node', unselectHandler);

  // Snap-to-grid on dragfree
  const dragfreeHandler = (e: EventObject) => {
    const node = e.target as NodeSingular;
    const pos = node.position();
    const snapped = snapToGrid(pos);
    node.position(snapped);

    // Save snapped position to layout metadata
    if (data?.metadata) {
      if (!data.metadata.layout) {
        data.metadata.layout = {};
      }
      data.metadata.layout[node.id()] = {
        x: snapped.x,
        y: snapped.y,
        width: node.width(),
        height: node.height()
      };

      if (handlers.onDragFree) {
        handlers.onDragFree(node.id(), snapped);
      }
    }
  };

  cy.on('dragfree', 'node', dragfreeHandler);

  // Cleanup function
  return () => {
    cy.off('select', 'node', selectHandler);
    cy.off('unselect', 'node', unselectHandler);
    cy.off('dragfree', 'node', dragfreeHandler);
  };
}

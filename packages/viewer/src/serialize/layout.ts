// packages/viewer/src/serialize/layout.ts
import type { Core } from 'cytoscape';
import type { LayoutData, MetadataJSON } from '../types';

/**
 * Extract layout data from Cytoscape nodes
 */
export function extractLayout(cy: Core): Record<string, LayoutData> {
  const layout: Record<string, LayoutData> = {};
  cy.nodes().forEach(node => {
    const id = node.data('id');
    const pos = node.position();
    const width = node.width();
    const height = node.height();
    layout[id] = {
      x: Math.round(pos.x),
      y: Math.round(pos.y),
      width: Math.round(width),
      height: Math.round(height)
    };
  });
  return layout;
}

/**
 * Update layout metadata from Cytoscape nodes
 */
export function updateLayoutMetadata(
  cy: Core,
  metadata: MetadataJSON
): void {
  if (!metadata.layout) {
    metadata.layout = {};
  }
  const layout = extractLayout(cy);
  Object.assign(metadata.layout, layout);
}

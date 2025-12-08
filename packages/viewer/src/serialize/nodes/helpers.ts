// packages/viewer/src/serialize/nodes/helpers.ts
import type { NodeSingular } from 'cytoscape';
import type { MetadataEntry, SystemJSON } from '../../types';

/**
 * Get metadata from node data
 */
export function getMeta(ele: NodeSingular): MetadataEntry[] | undefined {
  return ele.data('metadata');
}

/**
 * Strip prefix from ID if it starts with the prefix
 */
export function stripPrefix(id: string, prefix: string): string {
  return id.startsWith(prefix + '.') ? id.slice(prefix.length + 1) : id;
}

/**
 * Find system in architecture by ID
 */
export function findSystem(arch: { systems?: SystemJSON[] }, systemId: string): SystemJSON | undefined {
  return arch.systems?.find((s) => s.id === systemId);
}

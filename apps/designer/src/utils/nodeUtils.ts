// apps/designer/src/utils/nodeUtils.ts
import type {
  SrujaModelDump,
  ElementDump,
} from "../types";

/**
 * Generate a unique ID by appending a number suffix
 */
export function generateUniqueId(baseId: string, existingIds: Set<string>): string {
  let candidate = baseId;
  let counter = 1;
  while (existingIds.has(candidate)) {
    candidate = `${baseId}-${counter}`;
    counter++;
  }
  return candidate;
}

/**
 * Find a node in the architecture by ID
 */
export function findNodeInArchitecture(
  arch: SrujaModelDump,
  nodeId: string
): ElementDump | null {
  if (!arch.elements) return null;
  return arch.elements[nodeId] || null;
}

/**
 * Get all existing IDs from architecture (for uniqueness checking)
 */
export function getAllNodeIds(arch: SrujaModelDump): Set<string> {
  const ids = new Set<string>();
  if (arch.elements) {
    Object.keys(arch.elements).forEach(id => ids.add(id));
  }
  return ids;
}

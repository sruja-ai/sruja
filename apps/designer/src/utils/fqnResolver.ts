import type { SrujaModelDump } from "@sruja/shared";

/**
 * Resolves FQN (Fully Qualified Name) references to node IDs in the diagram.
 *
 * Scenarios often use hierarchical references like "system.container.component".
 * The diagram nodes usually have IDs that might be simple (e.g. "component") or constructed differently.
 * This resolver mimics the resolution logic to find the best matching node ID.
 */
export function resolveFqnToNodeId(fqn: string, model: SrujaModelDump | null): string | null {
  if (!model || !fqn) return null;

  // 1. Direct match check
  if (model.elements && model.elements[fqn]) {
    return fqn;
  }

  // 2. Case insensitive match
  const lowerFqn = fqn.toLowerCase();
  if (model.elements && model.elements[lowerFqn]) {
    return lowerFqn;
  }

  // 3. Leaf name match (e.g. "component" from "system.container.component")
  // Beware of collisions here.
  const parts = fqn.split(".");
  const leafName = parts[parts.length - 1];

  // Create candidate map if not cached (optimization: could be cached in a class or hook)
  // For now, simple scan
  let match: string | null = null;
  let collision = false;

  for (const id in model.elements) {
    if (id === leafName || id.toLowerCase() === leafName.toLowerCase()) {
      if (match) {
        collision = true; // Found multiple matches for leaf name
      }
      match = id;
    }
  }

  // If we found a unique match for the leaf name, use it
  if (match && !collision) {
    return match;
  }

  // 4. Hierarchical Resolution (most robust)
  // If fqn is "A.B.C", we look for an element "C" where parent is "B", and "B"'s parent is "A"
  // Or "C" where parent ID ends with "B" etc.

  // Not fully implemented yet as current model uses flat IDs often.
  // TODO: Add full path verification if collisions exist.

  return match; // Return best guess if collision (or null) - currently favoring first find if collision
}

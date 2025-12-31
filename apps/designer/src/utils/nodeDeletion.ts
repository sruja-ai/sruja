// apps/designer/src/utils/nodeDeletion.ts
import type { SrujaModelDump } from "@sruja/shared";

/**
 * Delete a node from the architecture
 */
export function deleteNodeFromArchitecture(arch: SrujaModelDump, nodeId: string): SrujaModelDump {
  if (!arch.elements) return arch;

  // Clone the architecture
  const newArch = {
    ...arch,
    elements: { ...arch.elements },
    relations: [...(arch.relations || [])],
  };

  // Find all descendants to delete
  const idsToDelete = new Set<string>([nodeId]);

  // Exhaustive search for children (since elements are flat but have parent pointers)
  // We need to keep finding children until no new ones are added
  let added = true;
  while (added) {
    added = false;
    for (const [id, el] of Object.entries(newArch.elements)) {
      const element = el as { parent?: string };
      if (!idsToDelete.has(id) && element.parent && idsToDelete.has(element.parent)) {
        idsToDelete.add(id);
        added = true;
      }
    }
  }

  // Remove elements
  idsToDelete.forEach((id) => {
    delete newArch.elements[id];
  });

  // Remove relations connected to deleted nodes
  // FqnRef format: source/target are { model: string } objects
  const getFqn = (ref: unknown): string => {
    if (typeof ref === "object" && ref !== null && "model" in ref) {
      return (ref as { model: string }).model;
    }
    return String(ref || "");
  };
  newArch.relations = newArch.relations.filter(
    (r) => !idsToDelete.has(getFqn(r.source)) && !idsToDelete.has(getFqn(r.target))
  );

  // Remove reference from parent if needed (not needed for flat list, but good to know)

  return newArch;
}

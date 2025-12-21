// Relationship validation for LikeC4 model conversion

/**
 * Validates and filters relationships to ensure they reference existing elements.
 * 
 * @param relations - Array of relationship objects
 * @param elements - Map of valid element IDs
 * @returns Filtered array of valid relationships
 * 
 * @remarks
 * Filters out relationships where:
 * - Source or target is missing or invalid
 * - Source or target element doesn't exist in the elements map
 */
export function validateRelations(relations: any[], elements: Record<string, any>): any[] {
  return relations.filter((rel) => {
    if (!rel || typeof rel !== "object") {
      return false;
    }

    // Handle FqnRef structure: { model: string } or fallback to string
    const sourceFqn =
      (rel.source && typeof rel.source === "object" && "model" in rel.source)
        ? rel.source.model
        : typeof rel.source === "string"
          ? rel.source
          : null;

    const targetFqn =
      (rel.target && typeof rel.target === "object" && "model" in rel.target)
        ? rel.target.model
        : typeof rel.target === "string"
          ? rel.target
          : null;

    if (!sourceFqn || !targetFqn || typeof sourceFqn !== "string" || typeof targetFqn !== "string") {
      console.error("❌ Filtering out relationship with invalid source/target structure:", rel);
      return false;
    }

    const sourceExists = sourceFqn in elements;
    const targetExists = targetFqn in elements;

    if (!sourceExists || !targetExists) {
      const missing = [];
      if (!sourceExists) missing.push(`source: ${sourceFqn}`);
      if (!targetExists) missing.push(`target: ${targetFqn}`);
      console.error(`❌ Filtering out relationship with missing elements (${missing.join(", ")})`, rel);
      return false;
    }

    return true;
  });
}


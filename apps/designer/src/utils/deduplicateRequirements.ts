// apps/designer/src/utils/deduplicateRequirements.ts
import type { RequirementDump } from "@sruja/shared";
import { logger } from "@sruja/shared";

/**
 * Deduplicate requirements array by ID
 * Keeps the first occurrence of each unique requirement ID
 */
export function deduplicateRequirements(
  requirements: readonly RequirementDump[]
): RequirementDump[] {
  const seenIds = new Set<string>();
  const uniqueRequirements: RequirementDump[] = [];

  for (const req of requirements) {
    const reqId = req.id;

    // If requirement has no ID, include it (will use index-based key)
    if (!reqId) {
      uniqueRequirements.push(req);
      continue;
    }

    // If we've seen this ID before, skip it (duplicate)
    if (seenIds.has(reqId)) {
      logger.warn("Duplicate requirement ID found, skipping duplicate", {
        component: "deduplicateRequirements",
        action: "deduplicate",
        requirementId: reqId,
      });
      continue;
    }

    // First occurrence of this ID, keep it
    seenIds.add(reqId);
    uniqueRequirements.push(req);
  }

  return uniqueRequirements;
}

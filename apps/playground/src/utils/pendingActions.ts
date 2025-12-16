// apps/playground/src/utils/pendingActions.ts
import type {
  ArchitectureJSON,
  ADRJSON,
  SystemJSON,
  ContainerJSON,
  ComponentJSON,
  PersonJSON,
} from "../types";

/**
 * Check if an ADR has a pending decision (missing decision field)
 */
export function isADRPending(adr: ADRJSON): boolean {
  return !adr.decision || adr.decision.trim() === "";
}

/**
 * Get all pending ADRs for a node by checking tags
 */
export function getPendingADRsForNode(
  nodeId: string,
  architecture: ArchitectureJSON["architecture"]
): ADRJSON[] {
  const allADRs = architecture.adrs || [];

  return allADRs.filter((adr: ADRJSON) => {
    // Check if ADR is tagged with this node
    const isTagged = adr.tags?.some((tag) => tag === nodeId) ?? false;
    return isTagged && isADRPending(adr);
  });
}

/**
 * Check if a node has any pending actions (ADRs without decisions)
 */
export function hasPendingActions(
  node: SystemJSON | ContainerJSON | ComponentJSON | PersonJSON,
  architecture: ArchitectureJSON["architecture"]
): boolean {
  const nodeId = node.id;

  // Check directly linked ADRs
  if ("adrs" in node && Array.isArray(node.adrs)) {
    const allADRs = architecture.adrs || [];
    for (const adrId of node.adrs) {
      const adr = allADRs.find((a: ADRJSON) => a.id === adrId);
      if (adr && isADRPending(adr)) {
        return true;
      }
    }
  }

  // Check tagged ADRs
  const pendingADRs = getPendingADRsForNode(nodeId, architecture);
  return pendingADRs.length > 0;
}

/**
 * Count pending actions for a node
 */
export function countPendingActions(
  node: SystemJSON | ContainerJSON | ComponentJSON | PersonJSON,
  architecture: ArchitectureJSON["architecture"]
): number {
  const nodeId = node.id;
  let count = 0;

  // Check directly linked ADRs
  if ("adrs" in node && Array.isArray(node.adrs)) {
    const allADRs = architecture.adrs || [];
    for (const adrId of node.adrs) {
      const adr = allADRs.find((a: ADRJSON) => a.id === adrId);
      if (adr && isADRPending(adr)) {
        count++;
      }
    }
  }

  // Check tagged ADRs (avoid double counting)
  const pendingADRs = getPendingADRsForNode(nodeId, architecture);
  const linkedAdrIds = "adrs" in node && Array.isArray(node.adrs) ? node.adrs : [];
  const uniquePending = pendingADRs.filter((adr) => !linkedAdrIds.includes(adr.id));
  count += uniquePending.length;

  return count;
}

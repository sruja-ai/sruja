/**
 * useGovernanceCounts hook
 * Computes requirement and ADR counts per element based on tag references
 */

import { useMemo } from "react";
import { useArchitectureStore } from "../stores/architectureStore";

export interface GovernanceCounts {
  [elementPath: string]: {
    requirementCount: number;
    adrCount: number;
  };
}

/**
 * Compute governance counts for all elements based on tag references
 */
export function useGovernanceCounts(): GovernanceCounts {
  const data = useArchitectureStore((s) => s.data);

  return useMemo(() => {
    const counts: GovernanceCounts = {};

    if (!data?.architecture) return counts;

    const requirements = data.architecture.requirements ?? [];
    const adrs = data.architecture.adrs ?? [];

    // Count requirements per element
    requirements.forEach((req) => {
      (req.tags ?? []).forEach((tag) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0 };
        }
        counts[tag].requirementCount++;
      });
    });

    // Count ADRs per element
    adrs.forEach((adr) => {
      (adr.tags ?? []).forEach((tag) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0 };
        }
        counts[tag].adrCount++;
      });
    });

    return counts;
  }, [data]);
}

/**
 * Enhance React Flow nodes with governance counts
 */
export function enrichNodesWithGovernance<T extends { id: string; data: Record<string, unknown> }>(
  nodes: T[],
  governanceCounts: GovernanceCounts
): T[] {
  return nodes.map((node) => {
    const counts = governanceCounts[node.id];
    if (!counts) return node;

    return {
      ...node,
      data: {
        ...node.data,
        requirementCount: counts.requirementCount,
        adrCount: counts.adrCount,
      },
    };
  });
}

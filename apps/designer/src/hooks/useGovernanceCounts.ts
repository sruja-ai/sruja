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
    scenarioCount: number;
    flowCount: number;
  };
}

/**
 * Compute governance counts for all elements based on tag references
 */
// ... imports
// ...

// ...

export function useGovernanceCounts(): GovernanceCounts {
  const data = useArchitectureStore((s) => s.model);

  return useMemo(() => {
    const counts: GovernanceCounts = {};

    // Check sruja property primarily
    if (!data?.sruja) return counts;

    const requirements = data.sruja.requirements ?? [];
    const adrs = (data as any).sruja.adrs ?? [];
    const scenarios = data.sruja.scenarios ?? [];
    const flows = scenarios;

    // Count requirements per element
    requirements.forEach((req: any) => {
      (req.tags ?? []).forEach((tag: string) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
        }
        counts[tag].requirementCount++;
      });
    });

    // Count ADRs per element
    adrs.forEach((adr: any) => {
      (adr.tags ?? []).forEach((tag: string) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
        }
        counts[tag].adrCount++;
      });
    });

    // Count scenarios per element (from scenario steps)
    scenarios.forEach((scenario: any) => {
      (scenario.steps ?? []).forEach((step: any) => {
        (step.tags ?? []).forEach((tag: string) => {
          if (!counts[tag]) {
            counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
          }
          counts[tag].scenarioCount++;
        });
      });
    });

    // Count flows per element (from flow steps)
    flows.forEach((flow: any) => {
      (flow.steps ?? []).forEach((step: any) => {
        (step.tags ?? []).forEach((tag: string) => {
          if (!counts[tag]) {
            counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
          }
          counts[tag].flowCount++;
        });
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
        scenarioCount: counts.scenarioCount,
        flowCount: counts.flowCount,
      },
    };
  });
}

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

import type {
  RequirementDump,
  ADRDump,
  ScenarioDump,
  FlowDump,
  SrujaModelDump,
} from "@sruja/shared";

export function useGovernanceCounts(): GovernanceCounts {
  const data = useArchitectureStore((s) => s.model) as SrujaModelDump | null;

  return useMemo(() => {
    const counts: GovernanceCounts = {};

    // Check sruja property primarily
    if (!data?.sruja) return counts;

    const requirements = data.sruja.requirements ?? [];
    const adrs = data.sruja.adrs ?? [];
    const scenarios = data.sruja.scenarios ?? [];
    const flows = data.sruja.flows ?? [];

    // Count requirements per element
    requirements.forEach((req: RequirementDump) => {
      (req.tags ?? []).forEach((tag: string) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
        }
        counts[tag].requirementCount++;
      });
    });

    // Count ADRs per element
    adrs.forEach((adr: ADRDump) => {
      // Tags might be missing in interface but present in runtime
      const tags = ((adr as unknown as { tags?: string[] }).tags || []) as string[];
      tags.forEach((tag: string) => {
        if (!counts[tag]) {
          counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
        }
        counts[tag].adrCount++;
      });
    });

    // Count scenarios per element (from scenario steps)
    scenarios.forEach((scenario: ScenarioDump) => {
      (scenario.steps ?? []).forEach((step) => {
        const from = step.from;
        const to = step.to;
        [from, to].forEach((tag) => {
          if (!tag) return;
          if (!counts[tag]) {
            counts[tag] = { requirementCount: 0, adrCount: 0, scenarioCount: 0, flowCount: 0 };
          }
          counts[tag].scenarioCount++;
        });
      });
    });

    // Count flows per element (from flow steps)
    flows.forEach((flow: FlowDump) => {
      (flow.steps ?? []).forEach((step) => {
        const from = step.from;
        const to = step.to;
        [from, to].forEach((tag) => {
          if (!tag) return;
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

import { useMemo } from "react";
import type { ArchitectureJSON } from "../types";

export function useTabCounts(data: ArchitectureJSON | null) {
  return useMemo(() => {
    if (!data) {
      return { requirements: 0, adrs: 0 };
    }

    if (!data.architecture) {
      return { requirements: 0, adrs: 0 };
    }

    // Handle requirements - could be array or object
    let archRequirements: any[] = [];
    const reqs = data.architecture.requirements;

    if (Array.isArray(reqs)) {
      archRequirements = reqs;
    } else if (reqs && typeof reqs === "object") {
      // If it's an object, convert to array
      archRequirements = Object.values(reqs);
    }

    return {
      requirements: archRequirements.length,
      adrs: data.architecture.adrs?.length ?? 0,
    };
  }, [data]);
}

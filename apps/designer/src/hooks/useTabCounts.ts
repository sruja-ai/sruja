import { useMemo } from "react";
import type { SrujaModelDump } from "@sruja/shared";

export function useTabCounts(data: SrujaModelDump | null) {
  return useMemo(() => {
    if (!data || !data.sruja) {
      return { requirements: 0, adrs: 0 };
    }

    const reqs = data.sruja.requirements;
    const adrs = data.sruja.adrs;

    return {
      requirements: reqs?.length ?? 0,
      adrs: adrs?.length ?? 0,
    };
  }, [data]);
}

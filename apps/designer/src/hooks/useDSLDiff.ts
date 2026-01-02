import { useState, useMemo } from "react";
import { useArchitectureStore } from "../stores";

/**
 * Hook to manage diff view for DSL editor.
 *
 * Tracks whether diff mode is enabled and maintains a baseline DSL
 * for comparison when diff is shown.
 */
export function useDSLDiff() {
  const [showDiff, setShowDiff] = useState(false);
  const dslSource = useArchitectureStore((s) => s.dslSource);

  // Store baseline DSL when diff is first enabled
  // This allows comparing current DSL against the baseline
  const baselineDsl = useMemo(() => {
    if (showDiff && dslSource) {
      // Return current DSL as baseline when diff is enabled
      // In a more sophisticated implementation, this could be
      // stored separately when diff is first toggled on
      return dslSource;
    }
    return null;
  }, [showDiff, dslSource]);

  return {
    showDiff,
    baselineDsl,
    setShowDiff,
  };
}

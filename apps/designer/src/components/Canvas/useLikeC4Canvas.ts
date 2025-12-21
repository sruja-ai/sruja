import { useState, useEffect, useMemo } from "react";
import type { SrujaModelDump } from "@sruja/shared";
import type { LikeC4Model } from "@likec4/core/model";
import { computeAndLayoutModel } from "../../utils/computeAndLayoutModel";
import { convertToLikeC4ModelDump } from "./likeC4ModelConverter";
import type { ModelConversionResult } from "./types";

/**
 * Hook for managing LikeC4 model conversion and computation.
 * 
 * @param model - The Sruja model dump to convert
 * @param focusedSystemId - Optional system ID for L2 view generation
 * @param focusedContainerId - Optional container ID for L3 view generation
 * @returns Model conversion result with model, error state, and computing state
 * 
 * @remarks
 * Handles async model conversion and layout computation.
 * Automatically recomputes when model or navigation state changes.
 */
export function useLikeC4Model(
  model: SrujaModelDump | null,
  focusedSystemId: string | null,
  focusedContainerId: string | null
): ModelConversionResult {
  const [likec4Model, setLikec4Model] = useState<LikeC4Model<any> | null>(null);
  const [conversionError, setConversionError] = useState<string | null>(null);
  const [isComputing, setIsComputing] = useState(false);

  useEffect(() => {
    if (!model) {
      setLikec4Model(null);
      setConversionError(null);
      setIsComputing(false);
      return;
    }

    setIsComputing(true);
    setConversionError(null);

    console.log("[LikeC4Canvas] Model or navigation changed, recomputing diagram", {
      modelElements: Object.keys(model.elements || {}).length,
      focusedSystemId,
      focusedContainerId
    });

    const modelDump = convertToLikeC4ModelDump(model, focusedSystemId, focusedContainerId);
    if (!modelDump) {
      setLikec4Model(null);
      setConversionError("Failed to convert model to LikeC4 format. Check console for details.");
      setIsComputing(false);
      return;
    }

    console.log("[LikeC4Canvas] Computing and laying out model:", {
      elementsCount: Object.keys(modelDump.elements || {}).length,
      relationsCount: Array.isArray(modelDump.relations) ? modelDump.relations.length : 0,
      viewsCount: Object.keys(modelDump.views || {}).length,
      viewIds: Object.keys(modelDump.views || {}),
    });

    computeAndLayoutModel(modelDump)
      .then((result) => {
        const createdViews = [...result.views()];
        console.log("[LikeC4Canvas] Layouted model created:", {
          viewsCount: createdViews.length,
          viewIds: createdViews.map(v => v.id),
        });
        setLikec4Model(result);
        setConversionError(null);
      })
      .catch((error) => {
        const errorMessage = error instanceof Error ? error.message : "Unknown error";
        console.error("❌ Error computing/laying out model:", error);
        if (error instanceof Error && error.stack) {
          console.error("❌ Error stack:", error.stack);
        }
        setLikec4Model(null);
        setConversionError(`Failed to compute and layout model: ${errorMessage}`);
      })
      .finally(() => {
        setIsComputing(false);
      });
  }, [model, focusedSystemId, focusedContainerId]);

  return {
    model: likec4Model,
    error: conversionError,
    isComputing
  };
}

/**
 * Hook for managing available views from LikeC4 model.
 * 
 * @param model - The original Sruja model dump (for fallback)
 * @param likec4Model - The computed LikeC4 model
 * @returns Array of available view IDs, filtered to show L1, L2, L3 views
 */
export function useAvailableViews(
  model: SrujaModelDump | null,
  likec4Model: LikeC4Model<any> | null
): string[] {
  return useMemo<string[]>(() => {
    if (likec4Model) {
      try {
        const views = [...likec4Model.views()];
        const viewIds = views.map(v => v.id);
        console.log("[LikeC4Canvas] Available views from likec4Model:", viewIds, "Total views:", views.length);

        if (viewIds.length === 0) {
          console.warn("[LikeC4Canvas] LikeC4Model returned 0 views. Views may have been filtered out because:");
          console.warn("  - View rules don't match any elements in the model");
          console.warn("  - View structure is invalid");
          console.warn("  - Elements referenced in views don't exist");
          if (model?.views) {
            const modelDumpViews = Object.keys(model.views);
            console.warn("[LikeC4Canvas] Model dump has views but LikeC4Model filtered them out:", modelDumpViews);
          }
        }

        // Filter to show only L1, L2, L3 views (C4 landscape views)
        const landscapeViews = viewIds.filter(vid => vid === "L1" || vid === "L2" || vid === "L3");

        // If we have landscape views, return them; otherwise return all views
        return landscapeViews.length > 0 ? landscapeViews : viewIds;
      } catch (error) {
        console.error("[LikeC4Canvas] Error getting views from likec4Model:", error);
        return [];
      }
    }

    return [];
  }, [model, likec4Model]);
}

/**
 * Hook for managing active view selection.
 * 
 * @param availableViews - Array of available view IDs
 * @param viewId - Optional explicit view ID from props
 * @param currentLevel - Current navigation level (L1, L2, L3)
 * @returns Object with activeViewId and setUserSelectedViewId function
 */
export function useActiveViewId(
  availableViews: string[],
  viewId: string | undefined,
  currentLevel: string | null
): { activeViewId: string | null; setUserSelectedViewId: (id: string | null) => void } {
  const [userSelectedViewId, setUserSelectedViewId] = useState<string | null>(null);

  // Clear user selection when navigation changes
  useEffect(() => {
    if (currentLevel && (currentLevel === "L1" || currentLevel === "L2" || currentLevel === "L3")) {
      setUserSelectedViewId(null);
    }
  }, [currentLevel]);

  const activeViewId = useMemo<string | null>(() => {
    if (availableViews.length === 0) {
      return null;
    }

    // Priority: prop viewId > currentLevel > user selection > first available view
    if (viewId && availableViews.includes(viewId)) {
      return viewId;
    }

    if (currentLevel && availableViews.includes(currentLevel)) {
      return currentLevel;
    }

    if (userSelectedViewId && availableViews.includes(userSelectedViewId)) {
      return userSelectedViewId;
    }

    return availableViews[0];
  }, [userSelectedViewId, viewId, currentLevel, availableViews]);

  return { activeViewId, setUserSelectedViewId };
}

/**
 * Hook for applying node colors with retry logic and mutation observation.
 * 
 * @param containerRef - Ref to the container element
 * @param likec4Model - The LikeC4 model
 * @param activeViewId - The currently active view ID
 * @param selectedNodeId - The currently selected node ID
 * @param applyNodeColors - Function to apply node colors
 */
export function useNodeColorApplication(
  containerRef: React.RefObject<HTMLDivElement | null>,
  likec4Model: LikeC4Model<any> | null,
  activeViewId: string | null,
  selectedNodeId: string | null,
  applyNodeColors: () => void
): void {
  useEffect(() => {
    if (!containerRef.current || !likec4Model || !activeViewId) {
      return;
    }

    // Use requestAnimationFrame to ensure we apply colors after browser paint
    const applyColorsWithRetry = (attempt = 0) => {
      if (attempt < 3) {
        setTimeout(() => {
          requestAnimationFrame(() => {
            applyNodeColors();
            if (attempt < 2) {
              applyColorsWithRetry(attempt + 1);
            }
          });
        }, 100 * (attempt + 1));
      }
    };

    // Initial delay to allow LikeC4 to render
    const timer = setTimeout(() => {
      // Apply node colors with retry logic
      requestAnimationFrame(() => {
        applyNodeColors();
        applyColorsWithRetry(0);
      });
    }, 300);

    // Set up MutationObserver to catch dynamically added nodes
    const observer = new MutationObserver(() => {
      requestAnimationFrame(() => {
        applyNodeColors();
      });
    });

    if (containerRef.current) {
      observer.observe(containerRef.current, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ['data-element-id', 'data-node-id', 'data-id', 'id'],
      });
    }

    return () => {
      clearTimeout(timer);
      observer.disconnect();
    };
  }, [likec4Model, activeViewId, selectedNodeId, applyNodeColors, containerRef]);
}


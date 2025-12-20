import { useMemo, useState, useImperativeHandle, forwardRef, useRef, useEffect } from "react";
import type { SrujaModelDump } from "@sruja/shared";
import { DEFAULT_PROJECT_ID, DEFAULT_PROJECT_NAME } from "@sruja/shared/utils/constants";
import { LikeC4ModelProvider, LikeC4View } from "@likec4/diagram/bundle";
import { LikeC4Model } from "@likec4/core/model";
import type { LikeC4ModelDump } from "@likec4/core/types";
import html2canvas from "html2canvas";
import { Select } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { computeAndLayoutModel } from "../../utils/computeAndLayoutModel";
import "./LikeC4Canvas.css";

/**
 * Converts a SrujaModelDump to a LikeC4ModelDump format with runtime validation.
 * 
 * @internal
 * @param model - Sruja model dump to convert
 * @returns LikeC4ModelDump or null if conversion fails
 * 
 * @remarks
 * The WASM backend exports models in LikeC4ModelDump format, but we need to
 * ensure all required fields are present with appropriate defaults.
 * 
 * Runtime validation ensures:
 * - Required fields are present and of correct type
 * - Optional fields have safe defaults
 * - Type compatibility with LikeC4Model.fromDump expectations
 */
function convertToLikeC4ModelDump(
  model: SrujaModelDump
): LikeC4ModelDump | null {
  // Input validation
  if (!model || typeof model !== "object") {
    console.error("❌ Model is not a valid object");
    return null;
  }

  // Validate required fields
  if (!model.elements || typeof model.elements !== "object") {
    console.error("❌ Model missing required 'elements' field or elements is not an object");
    return null;
  }

  // Validate elements is a proper object (not array, not null)
  if (Array.isArray(model.elements) || model.elements === null) {
    console.error("❌ Model 'elements' must be an object, not array or null");
    return null;
  }

  try {
    // CRITICAL: Since we're using computeAndLayoutModel, we need the model dump in PARSED format
    // computeAndLayoutModel will handle the computation and layouting, converting to layouted format
    // So we always use 'parsed' stage here - computeAndLayoutModel will convert it to layouted
    const stage = 'parsed' as const;

    // Validate and construct project object
    const projectId = model.projectId || model.project?.id || DEFAULT_PROJECT_ID;
    const project = model.project || { id: projectId, name: DEFAULT_PROJECT_NAME };

    // Validate project structure
    if (typeof project.id !== "string" || project.id.length === 0) {
      console.error("❌ Invalid project.id: must be non-empty string");
      return null;
    }

    // Validate globals structure
    const globals = model.globals || { predicates: {}, dynamicPredicates: {}, styles: {} };
    if (typeof globals !== "object" || globals === null || Array.isArray(globals)) {
      console.error("❌ Invalid globals: must be an object");
      return null;
    }

    // Validate relations - should be array for SrujaModelDump
    const relations = Array.isArray(model.relations) ? model.relations : [];

    // CRITICAL: Filter out relationships that reference non-existent elements
    // LikeC4Model.fromDump will throw if a relationship references an element that doesn't exist
    const validRelations = relations.filter((rel) => {
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

      const sourceExists = sourceFqn in model.elements;
      const targetExists = targetFqn in model.elements;

      if (!sourceExists || !targetExists) {
        // Try to find case-insensitive matches to suggest correct FQNs
        const findCaseInsensitiveMatch = (fqn: string): string | null => {
          const parts = fqn.split('.');
          const elementKeys = Object.keys(model.elements);

          // Try to find a match by comparing each part case-insensitively
          for (const key of elementKeys) {
            const keyParts = key.split('.');
            if (parts.length === keyParts.length) {
              let matches = true;
              for (let i = 0; i < parts.length; i++) {
                if (parts[i].toLowerCase() !== keyParts[i].toLowerCase()) {
                  matches = false;
                  break;
                }
              }
              if (matches && parts.join('.') !== keyParts.join('.')) {
                return key; // Found case-insensitive match
              }
            }
          }
          return null;
        };

        const missing = [];
        const suggestions = [];

        if (!sourceExists) {
          missing.push(`source: ${sourceFqn}`);
          const suggested = findCaseInsensitiveMatch(sourceFqn);
          if (suggested) {
            suggestions.push(`Did you mean: ${suggested}?`);
          }
        }
        if (!targetExists) {
          missing.push(`target: ${targetFqn}`);
          const suggested = findCaseInsensitiveMatch(targetFqn);
          if (suggested) {
            suggestions.push(`Did you mean: ${suggested}?`);
          }
        }

        const errorMsg = `❌ Filtering out relationship with missing elements (${missing.join(", ")})`;
        const suggestionMsg = suggestions.length > 0 ? ` ${suggestions.join(' ')}` : '';
        console.error(errorMsg + suggestionMsg, rel);
        return false;
      }

      return true;
    });

    // Validate views - should be object
    let views = model.views && typeof model.views === "object" && !Array.isArray(model.views)
      ? { ...model.views }
      : {};

    // Ensure all views have required structure for LikeC4Model
    // LikeC4Model expects views to have rules, nodes, and edges
    // CRITICAL: include must be an array of expressions, not an object!
    // Format: { include: [{ wildcard: true }] } not { include: { wildcard: true } }
    const defaultViewConfig = {
      rules: [{ include: [{ wildcard: true }] }],
      nodes: [],
      edges: []
    };

    // Normalize all views to ensure they have required properties
    const normalizedViews: Record<string, any> = {};
    Object.keys(views).forEach((viewId) => {
      const view = views[viewId] as any;
      
      // Normalize rules - convert from model dump format to LikeC4 parsed format
      let normalizedRules = view.rules && Array.isArray(view.rules) && view.rules.length > 0
        ? view.rules
        : defaultViewConfig.rules;
      
      // Debug: Log original rules to help diagnose issues
      if (normalizedRules && normalizedRules.length > 0) {
        console.log(`[LikeC4Canvas] Normalizing rules for view ${viewId}:`, JSON.stringify(normalizedRules, null, 2));
      }
      
      // Normalize rules to LikeC4 format
      // Model dump format might have: { include: { wildcard: true } } or { include: { elements: ["id1", "id2"] } }
      // LikeC4 expects: { include: [{ wildcard: true }] } or { include: [{ ref: { model: "id1" } }, ...] }
      normalizedRules = normalizedRules.map((rule: any) => {
        const normalizedRule: any = { ...rule };
        
        // Handle include rules
        if (rule.include !== undefined && rule.include !== null) {
          if (typeof rule.include === 'string') {
            // Handle string values - convert to proper expression format
            if (rule.include === '*') {
              normalizedRule.include = [{ wildcard: true }];
            } else if (rule.include === 'include' || rule.include === 'exclude') {
              // "include" and "exclude" are keywords, not element IDs - use wildcard as fallback
              console.warn(`[LikeC4Canvas] Rule has "${rule.include}" as string - it's a keyword, not an element ID. Using wildcard as fallback.`);
              normalizedRule.include = [{ wildcard: true }];
            } else {
              // Assume it's an element ID - convert to ModelRef
              normalizedRule.include = [{ ref: { model: rule.include } }];
            }
          } else if (rule.include.wildcard) {
            // Convert { include: { wildcard: true } } to { include: [{ wildcard: true }] }
            normalizedRule.include = [{ wildcard: true }];
          } else if (rule.include.elements && Array.isArray(rule.include.elements)) {
            // Convert { include: { elements: ["id1", "id2"] } } to { include: [{ ref: { model: "id1" } }, ...] }
            // Filter out "include" and "exclude" as they are keywords, not element IDs
            normalizedRule.include = rule.include.elements
              .filter((el: string) => {
                if (el === 'include' || el === 'exclude') {
                  console.warn(`[LikeC4Canvas] Filtering out invalid element ID "${el}" - it's a keyword, not an element ID`);
                  return false;
                }
                return true;
              })
              .map((el: string) => ({
                ref: { model: el }
              }));
            
            // If all elements were filtered out, use wildcard as fallback
            if (normalizedRule.include.length === 0) {
              console.warn(`[LikeC4Canvas] All elements in include.elements were invalid, using wildcard as fallback`);
              normalizedRule.include = [{ wildcard: true }];
            }
          } else if (Array.isArray(rule.include)) {
            // Already an array, but normalize each expression
            normalizedRule.include = rule.include
              .map((expr: any) => {
                if (typeof expr === 'string') {
                  // Convert string to proper expression format
                  if (expr === '*') {
                    return { wildcard: true };
                  } else if (expr === 'include' || expr === 'exclude') {
                    // "include" and "exclude" are keywords, not element IDs - skip
                    console.warn(`[LikeC4Canvas] Skipping invalid expression in array: "${expr}" is a keyword, not an element ID`);
                    return null;
                  } else {
                    // Assume it's an element ID - convert to ModelRef
                    return { ref: { model: expr } };
                  }
                }
                // Ensure wildcard expressions are in correct format
                if (expr && typeof expr === 'object' && expr.wildcard === true) {
                  return { wildcard: true };
                }
                // If it's already a proper expression object, return as-is
                return expr;
              })
              .filter((expr: any) => expr !== null); // Remove null entries
            
            // If all expressions were filtered out, use wildcard as fallback
            if (normalizedRule.include.length === 0) {
              console.warn(`[LikeC4Canvas] All expressions in include array were invalid, using wildcard as fallback`);
              normalizedRule.include = [{ wildcard: true }];
            }
          } else if (typeof rule.include === 'object') {
            // Single object, check if it's a valid expression or needs conversion
            if (rule.include.wildcard === true) {
              normalizedRule.include = [{ wildcard: true }];
            } else if (rule.include.ref) {
              // Already a proper expression object
              normalizedRule.include = [rule.include];
            } else {
              // Unknown object structure - log warning and use wildcard as fallback
              console.warn(`[LikeC4Canvas] Unknown include rule format:`, rule.include);
              normalizedRule.include = [{ wildcard: true }];
            }
          }
        }
        
        // Handle exclude rules (same logic as include)
        if (rule.exclude !== undefined && rule.exclude !== null) {
          if (typeof rule.exclude === 'string') {
            // Handle string values - convert to proper expression format
            if (rule.exclude === '*') {
              normalizedRule.exclude = [{ wildcard: true }];
            } else if (rule.exclude === 'include' || rule.exclude === 'exclude') {
              // "include" and "exclude" are keywords, not element IDs - skip this exclude rule
              console.warn(`[LikeC4Canvas] Rule has "${rule.exclude}" as string - it's a keyword, not an element ID. Skipping exclude rule.`);
              delete normalizedRule.exclude;
            } else {
              // Assume it's an element ID - convert to ModelRef
              normalizedRule.exclude = [{ ref: { model: rule.exclude } }];
            }
          } else if (rule.exclude.wildcard) {
            normalizedRule.exclude = [{ wildcard: true }];
          } else if (rule.exclude.elements && Array.isArray(rule.exclude.elements)) {
            // Filter out "include" and "exclude" as they are keywords, not element IDs
            normalizedRule.exclude = rule.exclude.elements
              .filter((el: string) => {
                if (el === 'include' || el === 'exclude') {
                  console.warn(`[LikeC4Canvas] Filtering out invalid element ID "${el}" - it's a keyword, not an element ID`);
                  return false;
                }
                return true;
              })
              .map((el: string) => ({
                ref: { model: el }
              }));
          } else if (Array.isArray(rule.exclude)) {
            normalizedRule.exclude = rule.exclude
              .map((expr: any) => {
                if (typeof expr === 'string') {
                  if (expr === '*') {
                    return { wildcard: true };
                  } else if (expr === 'include' || expr === 'exclude') {
                    // "include" and "exclude" are keywords, not element IDs - skip
                    console.warn(`[LikeC4Canvas] Skipping invalid expression in exclude array: "${expr}" is a keyword, not an element ID`);
                    return null;
                  } else {
                    return { ref: { model: expr } };
                  }
                }
                if (expr && typeof expr === 'object' && expr.wildcard === true) {
                  return { wildcard: true };
                }
                return expr;
              })
              .filter((expr: any) => expr !== null); // Remove null entries
          } else if (typeof rule.exclude === 'object') {
            // Single object, check if it's a valid expression or needs conversion
            if (rule.exclude.wildcard === true) {
              normalizedRule.exclude = [{ wildcard: true }];
            } else if (rule.exclude.ref) {
              // Already a proper expression object
              normalizedRule.exclude = [rule.exclude];
            } else {
              // Unknown object structure - log warning and use wildcard as fallback
              console.warn(`[LikeC4Canvas] Unknown exclude rule format:`, rule.exclude);
              normalizedRule.exclude = [{ wildcard: true }];
            }
          }
        }
        
        // Final validation: ensure all expressions are proper objects, not strings
        // Also filter out invalid expressions that reference keywords as element IDs
        if (normalizedRule.include && Array.isArray(normalizedRule.include)) {
          normalizedRule.include = normalizedRule.include
            .map((expr: any) => {
              if (typeof expr === 'string') {
                if (expr === '*') {
                  return { wildcard: true };
                } else if (expr === 'include' || expr === 'exclude') {
                  // "include" and "exclude" are keywords, not element IDs - skip this expression
                  console.warn(`[LikeC4Canvas] Skipping invalid expression: "${expr}" is a keyword, not an element ID`);
                  return null;
                } else {
                  // Convert string to ModelRef
                  return { ref: { model: expr } };
                }
              }
              // Check if expression references "include" or "exclude" as element ID
              if (expr && typeof expr === 'object' && expr.ref && typeof expr.ref === 'object') {
                const elementId = expr.ref.model || expr.ref;
                if (elementId === 'include' || elementId === 'exclude') {
                  console.warn(`[LikeC4Canvas] Skipping invalid expression: references "${elementId}" which is a keyword, not an element ID`);
                  return null;
                }
              }
              return expr;
            })
            .filter((expr: any) => expr !== null); // Remove null entries
          
          // If all expressions were filtered out, use wildcard as fallback
          if (normalizedRule.include.length === 0) {
            console.warn(`[LikeC4Canvas] All include expressions were invalid, using wildcard as fallback`);
            normalizedRule.include = [{ wildcard: true }];
          }
        }
        if (normalizedRule.exclude && Array.isArray(normalizedRule.exclude)) {
          normalizedRule.exclude = normalizedRule.exclude
            .map((expr: any) => {
              if (typeof expr === 'string') {
                if (expr === '*') {
                  return { wildcard: true };
                } else if (expr === 'include' || expr === 'exclude') {
                  // "include" and "exclude" are keywords, not element IDs - skip this expression
                  console.warn(`[LikeC4Canvas] Skipping invalid expression: "${expr}" is a keyword, not an element ID`);
                  return null;
                } else {
                  // Convert string to ModelRef
                  return { ref: { model: expr } };
                }
              }
              // Check if expression references "include" or "exclude" as element ID
              if (expr && typeof expr === 'object' && expr.ref && typeof expr.ref === 'object') {
                const elementId = expr.ref.model || expr.ref;
                if (elementId === 'include' || elementId === 'exclude') {
                  console.warn(`[LikeC4Canvas] Skipping invalid expression: references "${elementId}" which is a keyword, not an element ID`);
                  return null;
                }
              }
              return expr;
            })
            .filter((expr: any) => expr !== null); // Remove null entries
        }
        
        return normalizedRule;
      });
      
      // CRITICAL: Views must be in PARSED format for computeAndLayoutModel
      // computeAndLayoutModel will compute and layout them, converting to layouted format
      // Parsed views need _stage: 'parsed' and _type: 'element'|'deployment'|'dynamic'
      normalizedViews[viewId] = {
        id: view.id || viewId,
        title: view.title || viewId,
        _stage: 'parsed' as const,
        _type: view._type || 'element' as const,
        rules: normalizedRules,
        // Parsed views don't need nodes/edges - they'll be computed from rules
        // But preserve them if they exist (might be from a previous computation)
        ...(view.nodes && Array.isArray(view.nodes) ? { nodes: view.nodes } : {}),
        ...(view.edges && Array.isArray(view.edges) ? { edges: view.edges } : {}),
        // Preserve other view properties (but not _stage, _type, rules which we set above)
        ...Object.fromEntries(
          Object.entries(view).filter(([key]) => !['id', 'title', '_stage', '_type', 'rules'].includes(key))
        )
      };
    });

    // If no views exist, add default views
    // These must be in PARSED format for computeAndLayoutModel
    if (Object.keys(normalizedViews).length === 0) {
      normalizedViews["index"] = {
        id: "index",
        title: "Index",
        _stage: 'parsed' as const,
        _type: 'element' as const,
        ...defaultViewConfig
      };
      normalizedViews["L1"] = {
        id: "L1",
        title: "Landscape View (L1)",
        _stage: 'parsed' as const,
        _type: 'element' as const,
        ...defaultViewConfig
      };
    }

    views = normalizedViews;
    
    // Debug: Log view structure
    if (Object.keys(views).length > 0) {
      console.log("[LikeC4Canvas] Normalized views:", Object.keys(views), views);
      // Log detailed view structure for debugging
      Object.entries(views).forEach(([viewId, view]: [string, any]) => {
        console.log(`[LikeC4Canvas] View ${viewId}:`, {
          id: view.id,
          title: view.title,
          rules: view.rules,
          rulesCount: view.rules?.length || 0,
          nodesCount: view.nodes?.length || 0,
          edgesCount: view.edges?.length || 0,
        });
      });
    } else {
      console.warn("[LikeC4Canvas] No views found after normalization");
    }

    // Validate specification
    const specification = model.specification && typeof model.specification === "object" && !Array.isArray(model.specification)
      ? model.specification
      : { elements: {}, tags: {}, relationships: {} };

    // Validate deployments
    const deployments = model.deployments && typeof model.deployments === "object" && !Array.isArray(model.deployments)
      ? model.deployments
      : { elements: {}, relations: {} };

    // Validate imports
    const imports = model.imports && typeof model.imports === "object" && !Array.isArray(model.imports)
      ? model.imports
      : {};

    const modelDump: LikeC4ModelDump = {
      _stage: stage as any,
      projectId,
      project,
      globals: globals as LikeC4ModelDump['globals'],
      imports: imports as LikeC4ModelDump['imports'],
      deployments: deployments as LikeC4ModelDump['deployments'],
      specification: specification as LikeC4ModelDump['specification'],
      elements: model.elements as LikeC4ModelDump['elements'],
      relations: validRelations as LikeC4ModelDump['relations'],
      views: views as LikeC4ModelDump['views'],
    };

    return modelDump;
  } catch (error) {
    console.error("❌ Error converting model to LikeC4ModelDump:", error);
    if (error instanceof Error) {
      console.error("❌ Error details:", error.message, error.stack);
    }
    return null;
  }
}

export interface ArchitectureCanvasRef {
  exportAsPNG: (filename?: string) => Promise<void>;
  exportAsSVG: (filename?: string) => Promise<void>;
  getReactFlowInstance: () => null; // Not available in LikeC4
  fitView: (options?: { padding?: number; includeHiddenNodes?: boolean }) => void;
  zoomToSelection: () => void;
  zoomToActualSize: () => void;
  focusNode: (nodeId: string) => void;
}

interface LikeC4CanvasProps {
  model: SrujaModelDump | null;
  dragEnabled?: boolean; // Not used in LikeC4, kept for API compatibility
  viewId?: string;
}

/**
 * LikeC4-based diagram canvas component
 *
 * Replaces ArchitectureCanvas with LikeC4 rendering. Uses @likec4/diagram
 * for diagram rendering instead of ReactFlow.
 */
export const LikeC4Canvas = forwardRef<ArchitectureCanvasRef, LikeC4CanvasProps>(
  ({ model: propModel, dragEnabled: _dragEnabled = true, viewId }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const storeLikec4Model = useArchitectureStore((s) => s.likec4Model);

    // FIX: Remove derived state anti-pattern.
    // Use the model from prop (priority) or store directly.
    const model = propModel || storeLikec4Model;

    // Convert model dump to LikeC4Model and compute/layout views
    // This is async, so we use state and useEffect
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

      const modelDump = convertToLikeC4ModelDump(model);
      if (!modelDump) {
        setLikec4Model(null);
        setConversionError("Failed to convert model to LikeC4 format. Check console for details.");
        setIsComputing(false);
        return;
      }

      // Log model dump structure
      console.log("[LikeC4Canvas] Computing and laying out model:", {
        elementsCount: Object.keys(modelDump.elements || {}).length,
        relationsCount: Array.isArray(modelDump.relations) ? modelDump.relations.length : 0,
        viewsCount: Object.keys(modelDump.views || {}).length,
        viewIds: Object.keys(modelDump.views || {}),
      });

      // Compute and layout views asynchronously
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
    }, [model]);

    // Get available views from the model
    // CRITICAL: Only use views that exist in likec4Model, not fallback views from model dump
    // LikeC4View component requires views to exist in the LikeC4Model instance
    const availableViews = useMemo<string[]>(() => {
      if (likec4Model) {
        // likec4Model.views is a map of ViewID -> View
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
          
          // Only return views that exist in likec4Model - don't use fallback
          // because LikeC4View will fail if the view doesn't exist in the model
          return viewIds;
        } catch (error) {
          console.error("[LikeC4Canvas] Error getting views from likec4Model:", error);
          return [];
        }
      }

      // If likec4Model doesn't exist yet, return empty array
      // We'll show loading state instead of trying to render
      return [];
    }, [model, likec4Model]);

    // State for user-selected view
    const [userSelectedViewId, setUserSelectedViewId] = useState<string | null>(null);

    // Calculate the actual view to render - derived from props and state, ensuring validity
    // Only use views that exist in availableViews (which come from likec4Model)
    const activeViewId = useMemo<string | null>(() => {
      // If no views available, return null to show empty state
      if (availableViews.length === 0) {
        return null;
      }

      // 1. Prefer user selection if valid
      if (userSelectedViewId && availableViews.includes(userSelectedViewId)) {
        return userSelectedViewId;
      }

      // 2. Fallback to prop viewId if valid
      if (viewId && availableViews.includes(viewId)) {
        return viewId;
      }

      // 3. Fallback to first available view
      return availableViews[0];
    }, [userSelectedViewId, viewId, availableViews]);

    // Update state wrapper to keep selected value in sync if needed (optional)
    const handleViewChange = (vid: string) => {
      setUserSelectedViewId(vid);
    };

    // Expose methods via ref for compatibility with existing code
    useImperativeHandle(
      ref,
      () => ({
        exportAsPNG: async (filename?: string) => {
          if (!containerRef.current || !model) {
            throw new Error("Canvas not initialized");
          }
          try {
            const canvas = await html2canvas(containerRef.current);
            const url = canvas.toDataURL("image/png");
            const link = document.createElement("a");
            link.download = filename || `${model._metadata?.name || "diagram"}.png`;
            link.href = url;
            link.click();
          } catch (error) {
            throw new Error(`Failed to export PNG: ${error instanceof Error ? error.message : "Unknown error"}`);
          }
        },
        exportAsSVG: async (filename?: string) => {
          // LikeC4 renders to canvas/SVG internally, but html2canvas converts to PNG.
          // True SVG export would require accessing LikeC4's internal SVG representation.
          // For now, we export as PNG with a .png extension (not .svg) to avoid confusion.
          if (!containerRef.current || !model) {
            throw new Error("Canvas not initialized");
          }
          try {
            const canvas = await html2canvas(containerRef.current);
            const url = canvas.toDataURL("image/png");
            const link = document.createElement("a");
            const baseFilename = filename || `${model._metadata?.name || "diagram"}`;
            // Remove .svg extension if present and add .png
            link.download = baseFilename.replace(/\.svg$/, "") + ".png";
            link.href = url;
            link.click();
          } catch (error) {
            throw new Error(`Failed to export diagram: ${error instanceof Error ? error.message : "Unknown error"}`);
          }
        },
        getReactFlowInstance: () => null, // Not available in LikeC4
        fitView: () => {
          // LikeC4 handles view fitting internally via its own viewport management.
          // This method is a no-op for API compatibility with ReactFlow-based components.
          // TODO: Implement if LikeC4 exposes viewport control API
        },
        zoomToSelection: () => {
          // LikeC4 handles zoom internally via its own viewport management.
          // This method is a no-op for API compatibility with ReactFlow-based components.
          // TODO: Implement if LikeC4 exposes selection-based zoom API
        },
        zoomToActualSize: () => {
          // LikeC4 handles zoom internally via its own viewport management.
          // This method is a no-op for API compatibility with ReactFlow-based components.
          // TODO: Implement if LikeC4 exposes zoom-to-actual-size API
        },
        focusNode: (_nodeId: string) => {
          // LikeC4 handles node focus internally via its own viewport management.
          // This method is a no-op for API compatibility with ReactFlow-based components.
          // TODO: Implement if LikeC4 exposes node focus/scroll API
        },
      }),
      [model]
    );

    if (!model) {
      return (
        <div className="likec4-canvas-empty">
          <div className="likec4-canvas-empty-content">
            <p>No diagram data available</p>
            <p className="likec4-canvas-empty-message">
              The DSL may contain syntax errors or the model failed to load.
            </p>
          </div>
        </div>
      );
    }

    // Check if we have views in the model dump even if LikeC4Model filtered them out
    const hasViewsInModel = model?.views && Object.keys(model.views).length > 0;
    
    // Show loading state while computing/laying out
    if (isComputing) {
      return (
        <div className="likec4-canvas-empty">
          <div className="likec4-canvas-empty-content">
            <p>Computing and laying out views...</p>
            <p className="likec4-canvas-empty-message">
              This may take a moment for complex models
            </p>
          </div>
        </div>
      );
    }
    
    if (availableViews.length === 0 && !conversionError) {
      // If we have views in the model but LikeC4Model filtered them out,
      // it might be because the views need to be computed or the rules don't match elements
      if (hasViewsInModel) {
        console.warn("[LikeC4Canvas] Model has views but LikeC4Model filtered them out. This might indicate invalid view rules or missing elements.");
      }
      return (
        <div className="likec4-canvas-empty">
          <div className="likec4-canvas-empty-content">
            <p>No diagrams found</p>
            <p className="likec4-canvas-empty-message">
              The model is valid but contains no views. Add a <code>views</code> block to your DSL.
              {hasViewsInModel && " Views exist in the model but were filtered out by LikeC4Model - check view rules and element IDs."}
            </p>
          </div>
        </div>
      );
    }

    return (
      <div
        ref={containerRef}
        className="likec4-canvas react-flow"
        data-testid="likec4-canvas"
      >
        {/* View selector */}
        {availableViews.length > 1 && activeViewId && (
          <div className="likec4-view-selector">
            <Select
              value={activeViewId}
              onChange={(value) => {
                if (value) {
                  handleViewChange(value);
                }
              }}
              data={availableViews.map((vid) => ({
                value: vid,
                label: model.views?.[vid]?.title || vid,
              }))}
              size="xs"
              style={{
                fontSize: 12,
              }}
            />
          </div>
        )}

        {/* LikeC4 diagram */}
        {likec4Model && activeViewId ? (
          <div className="likec4-diagram-container">
            <LikeC4ModelProvider likec4model={likec4Model}>
              <LikeC4View 
                viewId={activeViewId}
                pannable={true}
                zoomable={true}
                browser={true}
                enableFocusMode={true}
                enableElementDetails={true}
                enableRelationshipDetails={true}
              />
            </LikeC4ModelProvider>
          </div>
        ) : conversionError ? (
          <div className="likec4-canvas-empty">
            <div className="likec4-canvas-empty-content">
              <p>Diagram rendering failed</p>
              <p className="likec4-canvas-empty-message">{conversionError}</p>
            </div>
          </div>
        ) : (
          <div className="likec4-canvas-empty">
            <div className="likec4-canvas-empty-content">
              <p>Loading diagram...</p>
              <p className="likec4-canvas-empty-message">
                Converting model format
              </p>
            </div>
          </div>
        )}
      </div>
    );
  }
);

LikeC4Canvas.displayName = "LikeC4Canvas";

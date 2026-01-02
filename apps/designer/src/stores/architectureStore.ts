// apps/designer/src/stores/architectureStore.ts
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { convertDslToModel, convertDslToMarkdown } from "../wasm";
import { convertModelToDsl } from "../utils/modelToDsl";
import { useHistoryStore } from "./historyStore";
import { safeAsync, handleError, ErrorType } from "../utils/errorHandling";
import { logger } from "@sruja/shared";
import type { SrujaModelDump, ParsedView } from "@sruja/shared";

const STORAGE_KEY = "sruja-architecture-data";

/**
 * Architecture store state interface.
 *
 * Manages the core architecture data, DSL source, and conversion state.
 * Persists data to localStorage for automatic restoration.
 */
interface ArchitectureState {
  model: SrujaModelDump | null; // Main model data
  convertedMarkdown: string | null; // Cached Markdown converted from DSL
  isLoading: boolean;
  error: string | null;
  lastLoaded: string | null; // Timestamp of when data was last loaded
  dslSource?: string | null;
  sourceType?: "dsl" | "json" | null;
  currentExampleFile?: string | null;
  isConverting: boolean; // Track if conversion is in progress

  // Chaos Mode State
  chaosState: {
    enabled: boolean;
    failedNodeId: string | null;
  };
  setChaosEnabled: (enabled: boolean) => void;
  setFailedNode: (nodeId: string | null) => void;

  // Capacity Planning State
  capacityState: {
    userLoad: number; // Percentage 0-500
    trafficDistribution: Record<string, number>; // Region -> %
  };
  setCapacityLoad: (load: number) => void;

  // Baseline for Trend Analysis
  baselineModel: SrujaModelDump | null;
  setBaseline: (model?: SrujaModelDump | null) => void;

  // Actions
  loadFromDSL: (json: SrujaModelDump, dsl: string, file?: string | null) => Promise<void>;
  loadFromModel: (json: SrujaModelDump, file?: string | null) => Promise<void>;
  setDslSource: (dsl: string | null, file?: string | null) => void;
  refreshConvertedJson: () => Promise<void>; // Refresh JSON when DSL changes
  updateArchitecture: (updater: (arch: SrujaModelDump) => SrujaModelDump) => Promise<void>; // Update architecture and sync DSL
  reset: () => void;
  clearProject: () => void; // Reset everything to empty state
}

/**
 * Zustand store for managing architecture data and DSL operations.
 */
export const useArchitectureStore = create<ArchitectureState>()(
  persist(
    (set, get): ArchitectureState => ({
      model: null,
      convertedMarkdown: null,
      isLoading: false,
      error: null,
      lastLoaded: null,
      dslSource: null,
      sourceType: null,
      currentExampleFile: null,
      isConverting: false,
      baselineModel: null,

      // Initial Chaos State
      chaosState: {
        enabled: false,
        failedNodeId: null,
      },

      // Initial Capacity State
      capacityState: {
        userLoad: 100, // 100% nominal load
        trafficDistribution: { "us-east": 60, "eu-west": 40 },
      },

      loadFromDSL: async (json, dsl, file) => {
        // Store model and DSL in a single atomic update to prevent extra renders
        set({
          model: json,
          isLoading: false,
          error: null,
          lastLoaded: new Date().toISOString(),
          sourceType: "dsl",
          dslSource: dsl,
          currentExampleFile: file,
        });

        // Clear history and add initial state
        useHistoryStore.getState().clear();
        useHistoryStore.getState().push(json);

        // Convert DSL to Markdown
        if (dsl) {
          set({ isConverting: true });
          const { error, data: convertedData } = await safeAsync(
            async () => {
              const convertedMarkdown = await convertDslToMarkdown(dsl);
              return { convertedMarkdown };
            },
            "Failed to convert DSL to Markdown",
            ErrorType.VALIDATION
          );

          if (error) {
            handleError(error, "architectureStore.loadFromDSL.convert");
          } else if (convertedData) {
            set({
              convertedMarkdown: convertedData.convertedMarkdown,
            });
          }
          set({ isConverting: false });
        }
      },

      loadFromModel: async (json, file) => {
        set({ isConverting: true });

        // Ensure required fields are present for JSON format compatibility
        const updatedJson = {
          ...json,
          _stage: json._stage || ("parsed" as const),
          project: json.project || {
            id: json.projectId || "default-project",
            name: json._metadata?.name || "Architecture",
          },
          projectId: json.projectId || json.project?.id || "default-project",
          globals: json.globals || { predicates: {}, dynamicPredicates: {}, styles: {} },
        };

        // CRITICAL: include must be a ViewRuleExpr object, not an array!
        // Format: { include: { wildcard: true } } not { include: [{ wildcard: true }] }
        const defaultViewConfig = {
          rules: [{ include: { wildcard: true } }],
          nodes: [],
          edges: [],
        };

        // Create a mutable copy of views to modify
        const views: Record<string, ParsedView> = updatedJson.views ? { ...updatedJson.views } : {};

        // Merge existing views with default config (preserve existing view data)
        Object.keys(views).forEach((viewId) => {
          const existingView = views[viewId];
          if (
            !existingView.rules ||
            !Array.isArray(existingView.rules) ||
            existingView.rules.length === 0
          ) {
            views[viewId] = {
              ...existingView,
              ...defaultViewConfig,
              id: existingView.id || viewId,
              title: existingView.title || viewId,
            };
          }
        });

        // Add default views if they don't exist
        if (!views["index"]) {
          views["index"] = {
            id: "index",
            title: "Index",
            ...defaultViewConfig,
          };
        }
        if (!views["L1"]) {
          views["L1"] = {
            id: "L1",
            title: "Landscape View (L1)",
            ...defaultViewConfig,
          };
        }
        if (!views["L2"]) {
          views["L2"] = {
            id: "L2",
            title: "Container View (L2)",
            ...defaultViewConfig,
          };
        }
        if (!views["L3"]) {
          views["L3"] = {
            id: "L3",
            title: "Component View (L3)",
            ...defaultViewConfig,
          };
        }

        // Assign the modified views back to updatedJson
        updatedJson.views = views;

        // Update model and history
        set({
          model: updatedJson,
          isLoading: false,
          error: null,
          lastLoaded: new Date().toISOString(),
          currentExampleFile: file ?? null,
          sourceType: "dsl",
        });
        useHistoryStore.getState().push(updatedJson);

        // Convert updated JSON to DSL
        const { error, data: dsl } = await safeAsync(
          () => convertModelToDsl(updatedJson),
          "Failed to convert JSON to DSL",
          ErrorType.VALIDATION
        );

        if (error) {
          handleError(error, "architectureStore.loadFromModel");
          // Even if conversion fails, set a placeholder DSL so the code panel shows something
          const errorMessage = error instanceof Error ? error.message : "Unknown error";
          set({
            dslSource: `// Architecture: ${updatedJson._metadata?.name || updatedJson.project?.name || "Architecture"}\n// Failed to convert model to DSL. Please check the model structure.\n// Error: ${errorMessage}`,
            isConverting: false,
          });
        } else if (dsl !== undefined && dsl) {
          set({
            dslSource: dsl,
            isConverting: false,
          });

          // Also convert to Markdown
          try {
            const markdown = await convertDslToMarkdown(dsl);
            set({ convertedMarkdown: markdown });
          } catch (markdownError) {
            // Markdown conversion failure is non-critical
            logger.warn("Failed to convert DSL to Markdown", {
              component: "architectureStore",
              action: "convert_dsl_to_markdown",
              error: markdownError instanceof Error ? markdownError.message : String(markdownError),
            });
          }
        } else {
          set({ isConverting: false });
          // Set placeholder if conversion returned empty
          set({
            dslSource: `// Architecture: ${updatedJson._metadata?.name || updatedJson.project?.name || "Architecture"}\n// Model loaded but DSL conversion returned empty result.`,
          });
        }
      },

      setDslSource: async (dsl, file) => {
        const currentDsl = get().dslSource;
        set({ dslSource: dsl, sourceType: dsl ? "dsl" : null, currentExampleFile: file ?? null });

        // If DSL changed, refresh converted JSON and Markdown
        if (dsl && dsl !== currentDsl) {
          get().refreshConvertedJson();
        } else if (!dsl) {
          // Clear converted data if DSL is removed
          set({ model: null, convertedMarkdown: null });
        }
      },

      refreshConvertedJson: async () => {
        const dsl = get().dslSource;
        if (!dsl) {
          set({ model: null, convertedMarkdown: null });
          return;
        }

        set({ isConverting: true });
        const { error, data: convertedData } = await safeAsync(
          async () => {
            // Use model export directly
            let modelJson: SrujaModelDump | null = null;
            try {
              const modelData = await convertDslToModel(dsl);
              if (modelData && typeof modelData === "object" && "elements" in modelData) {
                modelJson = modelData as SrujaModelDump;
              }
            } catch (e) {
              logger.error("Model export failed", {
                component: "architectureStore",
                action: "export_model",
                error: e instanceof Error ? e.message : String(e),
              });
              throw e;
            }

            const convertedMarkdown = await convertDslToMarkdown(dsl);

            return {
              model: modelJson,
              convertedMarkdown,
            };
          },
          "Failed to refresh converted JSON and Markdown",
          ErrorType.VALIDATION
        );

        if (error) {
          handleError(error, "architectureStore.refreshConvertedJson");
          set({ isConverting: false });
        } else if (convertedData) {
          set({
            model: convertedData.model,
            convertedMarkdown: convertedData.convertedMarkdown,
            isConverting: false,
          });

          // Add to history if successful
          if (convertedData.model) {
            useHistoryStore.getState().push(convertedData.model);
          }
        } else {
          set({ isConverting: false });
        }
      },

      updateArchitecture: async (updater) => {
        const currentModel = get().model;

        // Even if no model exists, allow the updater to create one.
        // This is safe for templates that provide a full initial model.
        let baseModel = currentModel;
        if (!baseModel) {
          // console.log("[architectureStore] No current model, providing empty base for updater");
          baseModel = {
            _stage: "parsed",
            elements: {},
            relations: [],
            views: {},
            deployments: {},
            specification: { elements: {}, tags: {}, relationships: {} },
            project: { id: "sruja-project", name: "New Project" },
            _metadata: {
              name: "Untitled",
              version: "1.0.0",
              generated: new Date().toISOString(),
              srujaVersion: "2.0.0",
            },
          };
        }

        // Update the model
        // This triggers: Builder → Model → Diagram (via useEffect in SrujaCanvas)
        const updatedModel = updater(baseModel!);
        set({ model: updatedModel, lastLoaded: new Date().toISOString() });

        // Add to history
        useHistoryStore.getState().push(updatedModel);

        // Convert updated JSON to DSL
        // This triggers: Builder → Model → DSL → DSL Panel (via useEffect in DSLPanel)
        const { error } = await safeAsync(
          async () => {
            // console.log("[architectureStore] Converting model to DSL (Builder → DSL sync)");
            const newDsl = await convertModelToDsl(updatedModel);
            // Update DSL source - this will trigger DSLPanel to sync
            // Note: we don't call setDslSource directly to avoid circular update loop
            // DSLPanel's useEffect watches storeDslSource and updates local state
            set({
              dslSource: newDsl,
              sourceType: "dsl",
              isConverting: false,
            });
          },
          "Failed to convert JSON to DSL",
          ErrorType.VALIDATION
        );

        if (error) {
          handleError(error, "architectureStore.updateArchitecture");
          set({ isConverting: false });
        }
      },

      setChaosEnabled: (enabled) =>
        set((state) => ({
          chaosState: {
            ...state.chaosState,
            enabled,
            failedNodeId: enabled ? state.chaosState.failedNodeId : null,
          },
        })),

      setFailedNode: (nodeId) =>
        set((state) => ({
          chaosState: { ...state.chaosState, failedNodeId: nodeId },
        })),

      setCapacityLoad: (load) =>
        set((state) => ({
          capacityState: { ...state.capacityState, userLoad: load },
        })),

      reset: () => {
        set({
          model: null,
          convertedMarkdown: null,
          isLoading: false,
          error: null,
          lastLoaded: null,
          dslSource: null,
          sourceType: null,
          currentExampleFile: null,
          isConverting: false,
        });
        useHistoryStore.getState().clear();
      },

      setBaseline: (model) => {
        if (model === undefined) {
          // If no argument, use current model as baseline
          set({ baselineModel: get().model });
        } else {
          set({ baselineModel: model });
        }
      },

      clearProject: () => {
        get().reset();
        localStorage.removeItem(STORAGE_KEY);
      },
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => {
        // Return stable object reference - only create new object if values actually changed
        return {
          model: state.model,
          convertedMarkdown: state.convertedMarkdown,
          lastLoaded: state.lastLoaded,
          dslSource: state.dslSource,
          sourceType: state.sourceType,
          currentExampleFile: state.currentExampleFile,
          baselineModel: state.baselineModel, // Persist baseline
        };
      },
      // Handle rehydration (when data is loaded from storage)
      onRehydrateStorage: () => (state, error) => {
        if (error) {
          logger.warn("Failed to rehydrate architecture store", {
            component: "architectureStore",
            action: "rehydrate",
            error: error instanceof Error ? error.message : String(error),
          });
        } else if (state) {
          // Clear old syntax DSL from localStorage (old syntax starts with "architecture")
          if (state.dslSource && state.dslSource.trim().startsWith("architecture")) {
            logger.warn("Detected old syntax DSL in localStorage, clearing it", {
              component: "architectureStore",
              action: "clear_old_dsl",
            });
            state.dslSource = null;
            state.sourceType = null;
            state.model = null;
          }

          if (state.model) {
            /*
            logger.info(
              `Architecture automatically loaded from localStorage (saved: ${state.lastLoaded || "unknown"})`
            );
            */
          }
        }
      },
    }
  )
);

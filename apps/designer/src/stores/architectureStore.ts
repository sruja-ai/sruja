// apps/designer/src/stores/architectureStore.ts
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { convertDslToModel, convertDslToMarkdown } from "../wasm";
import { convertModelToDsl } from "../utils/modelToDsl";
import { useHistoryStore } from "./historyStore";
import { safeAsync, handleError, ErrorType, AppError } from "../utils/errorHandling";
import { logger } from "@sruja/shared";
import type { SrujaModelDump, ParsedView } from "@sruja/shared";

const STORAGE_KEY = "sruja-architecture-data";

type DslSyncOptions = {
  syncModel?: boolean;
};

type RefreshOptions = {
  includeMarkdown?: boolean;
};

let conversionCounter = 0;

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
  setDslSource: (dsl: string | null, file?: string | null, options?: DslSyncOptions) => Promise<void>;
  refreshConvertedJson: (options?: RefreshOptions) => Promise<{ error: string | null }>; // Refresh JSON when DSL changes
  updateArchitecture: (updater: (arch: SrujaModelDump) => SrujaModelDump) => Promise<void>; // Update architecture and sync DSL
  reset: () => void;
  clearProject: () => void; // Reset everything to empty state

  // Visual editing actions
  addNode: (
    nodeType: "person" | "system" | "container" | "component" | "datastore" | "queue",
    name: string,
    parentId?: string,
    position?: { x: number; y: number }
  ) => Promise<void>;
  addRelation: (fromId: string, toId: string, label?: string) => Promise<void>;
  deleteNodes: (nodeIds: string[]) => Promise<void>;
  deleteRelations: (relationIds: string[]) => Promise<void>;
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

      setDslSource: async (dsl, file, options) => {
        const currentDsl = get().dslSource;
        const shouldSync = options?.syncModel !== false;
        set({ dslSource: dsl, sourceType: dsl ? "dsl" : null, currentExampleFile: file ?? null });

        // If DSL changed, refresh converted JSON and Markdown
        if (dsl && dsl !== currentDsl && shouldSync) {
          get().refreshConvertedJson();
        } else if (!dsl) {
          // Clear converted data if DSL is removed
          set({ model: null, convertedMarkdown: null, error: null });
        }
      },

      refreshConvertedJson: async (options) => {
        const dsl = get().dslSource;
        if (!dsl) {
          set({ model: null, convertedMarkdown: null, error: null, isConverting: false });
          return { error: null };
        }

        const conversionId = (conversionCounter += 1);
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

            const shouldUpdateMarkdown = options?.includeMarkdown !== false;
            const convertedMarkdown = shouldUpdateMarkdown
              ? await convertDslToMarkdown(dsl)
              : get().convertedMarkdown;

            return {
              model: modelJson,
              convertedMarkdown,
            };
          },
          "Failed to refresh converted JSON and Markdown",
          ErrorType.VALIDATION
        );

        if (conversionId !== conversionCounter) {
          return { error: null };
        }

        if (error) {
          handleError(error, "architectureStore.refreshConvertedJson");
          const errorMessage =
            error instanceof AppError && error.context?.originalError
              ? String(error.context.originalError)
              : error instanceof Error
                ? error.message
                : String(error);
          set({ isConverting: false, error: errorMessage });
          return { error: errorMessage };
        } else if (convertedData) {
          set({
            model: convertedData.model,
            convertedMarkdown: convertedData.convertedMarkdown ?? null,
            isConverting: false,
            error: null,
          });

          // Add to history if successful
          if (convertedData.model) {
            useHistoryStore.getState().push(convertedData.model);
          }
          return { error: null };
        } else {
          set({ isConverting: false });
          return { error: null };
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

      // Visual editing actions
      addNode: async (nodeType, name, parentId, _position) => {
        const currentModel = get().model;
        if (!currentModel) {
          // Initialize empty model if needed
          await get().loadFromModel({
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
          });
        }

        await get().updateArchitecture((model) => {
          const updatedModel = { ...model };
          if (!updatedModel.elements) {
            updatedModel.elements = {};
          }

          // Generate a unique ID (simple approach: lowercase name with spaces replaced)
          const baseId = name.toLowerCase().replace(/\s+/g, "");
          let nodeId = baseId;
          let counter = 1;
          while (updatedModel.elements[nodeId]) {
            nodeId = `${baseId}${counter}`;
            counter++;
          }

          // Create element based on type
          const element: import("@sruja/shared").ElementDump = {
            id: nodeId,
            title: name,
            kind: nodeType,
            description: "",
            metadata: {},
          };

          // Always add to flat elements map (model uses flat structure)
          updatedModel.elements[nodeId] = element;

          // Handle parent relationship for nested elements (if parentId provided)
          // Note: The model structure is flat, but we can set parent reference if needed
          // For now, we'll create top-level elements and let the DSL generator handle nesting
          if (parentId && updatedModel.elements[parentId]) {
            const parent = updatedModel.elements[parentId];
            if (!parent.children) {
              parent.children = {};
            }
            // Also add to parent's children for hierarchical structure
            parent.children[nodeId] = element;
          }

          // Save initial position in view metadata if provided
          // Note: View key will be set by the caller based on current view context
          // For now, we'll save to a default L1 view, but this should be passed as parameter
          // The position will be saved when the node is dragged or when view context is available

          return updatedModel;
        });
      },

      addRelation: async (fromId, toId, label = "") => {
        const currentModel = get().model;
        if (!currentModel) {
          logger.warn("Cannot add relation: no model loaded");
          return;
        }

        // Verify both nodes exist
        if (!currentModel.elements?.[fromId] || !currentModel.elements?.[toId]) {
          logger.warn("Cannot add relation: source or target node not found", {
            fromId,
            toId,
          });
          return;
        }

        await get().updateArchitecture((model) => {
          const updatedModel = { ...model };
          if (!updatedModel.relations) {
            updatedModel.relations = [];
          }

          // Check if relation already exists
          const existingRelation = updatedModel.relations.find(
            (r) => r.source.model === fromId && r.target.model === toId
          );
          if (existingRelation) {
            // Update existing relation label if provided
            if (label) {
              existingRelation.title = label;
            }
            return updatedModel;
          }

          // Add new relation
          const relation: import("@sruja/shared").RelationDump = {
            id: `rel-${fromId}-${toId}-${Date.now()}`,
            source: { model: fromId },
            target: { model: toId },
            title: label || "",
          };
          updatedModel.relations = [...updatedModel.relations, relation];

          return updatedModel;
        });
      },

      deleteNodes: async (nodeIds) => {
        const currentModel = get().model;
        if (!currentModel) {
          return;
        }

        await get().updateArchitecture((model) => {
          const updatedModel = { ...model };
          if (!updatedModel.elements) {
            return updatedModel;
          }

          // Collect all node IDs to delete (including children)
          const nodesToDelete = new Set<string>(nodeIds);
          const collectChildren = (parentId: string) => {
            const element = updatedModel.elements?.[parentId];
            if (element?.children) {
              Object.keys(element.children).forEach((childId) => {
                nodesToDelete.add(childId);
                collectChildren(childId);
              });
            }
          };

          nodeIds.forEach((nodeId) => collectChildren(nodeId));

          // Remove elements
          const updatedElements = { ...updatedModel.elements };
          nodesToDelete.forEach((nodeId) => {
            delete updatedElements[nodeId];
            // Also remove from parent's children if exists
            Object.values(updatedElements).forEach((element) => {
              if (element.children?.[nodeId]) {
                const updatedChildren = { ...element.children };
                delete updatedChildren[nodeId];
                element.children =
                  Object.keys(updatedChildren).length > 0 ? updatedChildren : undefined;
              }
            });
          });
          updatedModel.elements = updatedElements;

          // Remove relations involving deleted nodes
          if (updatedModel.relations) {
            updatedModel.relations = updatedModel.relations.filter(
              (r) => !nodesToDelete.has(r.source.model) && !nodesToDelete.has(r.target.model)
            );
          }

          return updatedModel;
        });
      },

      deleteRelations: async (relationIds) => {
        // Note: relationIds can be indices or we can match by from/to
        // For simplicity, we'll match by from/to pairs
        const currentModel = get().model;
        if (!currentModel?.relations) {
          return;
        }

        await get().updateArchitecture((model) => {
          const updatedModel = { ...model };
          if (!updatedModel.relations) {
            return updatedModel;
          }

          // If relationIds are indices, remove by index
          // Otherwise, treat as from:to pairs
          const idsToDelete = new Set(relationIds);
          updatedModel.relations = updatedModel.relations.filter((relation, index) => {
            const relationKey = `${relation.source.model}:${relation.target.model}`;
            return !idsToDelete.has(relationKey) && !idsToDelete.has(String(index));
          });

          return updatedModel;
        });
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

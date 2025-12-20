// apps/designer/src/stores/architectureStore.ts
import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { convertDslToLikeC4, convertDslToMarkdown } from "../wasm";
import { convertModelToDsl } from "../utils/modelToDsl";
import { useHistoryStore } from "./historyStore";
import { safeAsync, handleError, ErrorType } from "../utils/errorHandling";
import type { SrujaModelDump } from "@sruja/shared";

const STORAGE_KEY = "sruja-architecture-data";

/**
 * Architecture store state interface.
 * 
 * Manages the core architecture data, DSL source, and conversion state.
 * Persists data to localStorage for automatic restoration.
 */
interface ArchitectureState {
  likec4Model: SrujaModelDump | null; // Main model data
  convertedMarkdown: string | null; // Cached Markdown converted from DSL
  isLoading: boolean;
  error: string | null;
  lastLoaded: string | null; // Timestamp of when data was last loaded
  dslSource?: string | null;
  sourceType?: "dsl" | "json" | null;
  currentExampleFile?: string | null;
  isConverting: boolean; // Track if conversion is in progress

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
      likec4Model: null,
      convertedMarkdown: null,
      isLoading: false,
      error: null,
      lastLoaded: null,
      dslSource: null,
      sourceType: null,
      currentExampleFile: null,
      isConverting: false,

      loadFromDSL: async (json, dsl, file) => {
        // Store the initial JSON
        set({
          likec4Model: json,
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

        // Ensure required fields are present for LikeC4 compatibility
        const updatedJson = {
          ...json,
          _stage: json._stage || ("parsed" as const),
          project: json.project || { id: json.projectId || "default-project", name: json._metadata?.name || "Architecture" },
          projectId: json.projectId || json.project?.id || "default-project",
          globals: json.globals || { predicates: {}, dynamicPredicates: {}, styles: {} },
        };

        // Ensure default views are present
        if (!updatedJson.views) updatedJson.views = {};

        // CRITICAL: include must be an array of expressions, not an object!
        // Format: { include: [{ wildcard: true }] } not { include: { wildcard: true } }
        const defaultViewConfig = {
          rules: [{ include: [{ wildcard: true }] }],
          nodes: [],
          edges: []
        };

        // Merge existing views with default config (preserve existing view data)
        Object.keys(updatedJson.views || {}).forEach((viewId) => {
          const existingView = (updatedJson.views || {})[viewId] as any;
          if (!existingView.rules || !Array.isArray(existingView.rules) || existingView.rules.length === 0) {
            (updatedJson.views as any)[viewId] = {
              ...existingView,
              ...defaultViewConfig,
              id: existingView.id || viewId,
              title: existingView.title || viewId,
            };
          }
        });

        // Add default views if they don't exist
        if (!updatedJson.views["index"]) {
          (updatedJson.views as any)["index"] = { id: "index", title: "Index", ...defaultViewConfig };
        }
        if (!updatedJson.views["L1"]) {
          (updatedJson.views as any)["L1"] = { id: "L1", title: "Landscape View (L1)", ...defaultViewConfig };
        }
        if (!updatedJson.views["L2"]) {
          (updatedJson.views as any)["L2"] = { id: "L2", title: "Container View (L2)", ...defaultViewConfig };
        }
        if (!updatedJson.views["L3"]) {
          (updatedJson.views as any)["L3"] = { id: "L3", title: "Component View (L3)", ...defaultViewConfig };
        }

        // Update model and history
        set({
          likec4Model: updatedJson,
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
            isConverting: false
          });
        } else if (dsl !== undefined && dsl) {
          set({
            dslSource: dsl,
            isConverting: false
          });

          // Also convert to Markdown
          try {
            const markdown = await convertDslToMarkdown(dsl);
            set({ convertedMarkdown: markdown });
          } catch (markdownError) {
            // Markdown conversion failure is non-critical
            console.warn("Failed to convert DSL to Markdown:", markdownError);
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
          set({ likec4Model: null, convertedMarkdown: null });
        }
      },

      refreshConvertedJson: async () => {
        const dsl = get().dslSource;
        if (!dsl) {
          set({ likec4Model: null, convertedMarkdown: null });
          return;
        }

        set({ isConverting: true });
        const { error, data: convertedData } = await safeAsync(
          async () => {
            // Use LikeC4 export directly
            let likec4Json: SrujaModelDump | null = null;
            try {
              const likec4Data = await convertDslToLikeC4(dsl);
              if (likec4Data && typeof likec4Data === 'object' && 'elements' in likec4Data) {
                likec4Json = likec4Data as SrujaModelDump;
              }
            } catch (e) {
              console.error("LikeC4 export failed:", e);
              throw e;
            }

            const convertedMarkdown = await convertDslToMarkdown(dsl);

            return {
              likec4Model: likec4Json,
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
            likec4Model: convertedData.likec4Model,
            convertedMarkdown: convertedData.convertedMarkdown,
            isConverting: false,
          });

          // Add to history if successful
          if (convertedData.likec4Model) {
            useHistoryStore.getState().push(convertedData.likec4Model);
          }
        } else {
          set({ isConverting: false });
        }
      },

      updateArchitecture: async (updater) => {
        const currentModel = get().likec4Model;

        // Even if no model exists, allow the updater to create one.
        // This is safe for templates that provide a full initial model.
        let baseModel = currentModel;
        if (!baseModel) {
          console.log("[architectureStore] No current model, providing empty base for updater");
          baseModel = {
            _stage: "parsed",
            elements: {},
            relations: [],
            views: {},
            specification: { elements: {}, tags: {}, relationships: {} },
            project: { id: "sruja-project", name: "New Project" },
            _metadata: {
              name: "Untitled",
              version: "1.0.0",
              generated: new Date().toISOString(),
              srujaVersion: "2.0.0"
            }
          } as any;
        }

        // Update the model
        const updatedModel = updater(baseModel!);
        set({ likec4Model: updatedModel, lastLoaded: new Date().toISOString() });

        // Add to history
        useHistoryStore.getState().push(updatedModel);

        // Convert updated JSON to DSL
        const { error } = await safeAsync(
          async () => {
            const newDsl = await convertModelToDsl(updatedModel);
            // Update DSL source and trigger refresh mechanism
            // Note: we don't call setDslSource directly to avoid circular update loop
            set({
              dslSource: newDsl,
              sourceType: "dsl",
              isConverting: false
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

      reset: () => {
        set({
          likec4Model: null,
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
          likec4Model: state.likec4Model,
          convertedMarkdown: state.convertedMarkdown,
          lastLoaded: state.lastLoaded,
          dslSource: state.dslSource,
          sourceType: state.sourceType,
          currentExampleFile: state.currentExampleFile,
        };
      },
      // Handle rehydration (when data is loaded from storage)
      onRehydrateStorage: () => (state, error) => {
        if (error) {
          console.warn("Failed to rehydrate architecture store:", error);
        } else if (state) {
          // Clear old syntax DSL from localStorage (old syntax starts with "architecture")
          if (state.dslSource && state.dslSource.trim().startsWith('architecture')) {
            console.warn('Detected old syntax DSL in localStorage, clearing it.');
            state.dslSource = null;
            state.sourceType = null;
            state.likec4Model = null;
          }

          if (state.likec4Model) {
            console.log(
              `Architecture automatically loaded from localStorage (saved: ${state.lastLoaded || "unknown"})`
            );
          }
        }
      },
    }
  )
);

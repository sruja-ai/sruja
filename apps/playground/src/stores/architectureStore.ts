import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { ArchitectureJSON } from '../types';
import { convertDslToJson, convertDslToMarkdown } from '../wasm';
import { convertJsonToDsl } from '../utils/jsonToDsl';

const STORAGE_KEY = 'sruja-architecture-data';

interface ArchitectureState {
    data: ArchitectureJSON | null;
    convertedJson: ArchitectureJSON | null; // Cached JSON converted from DSL
    convertedMarkdown: string | null; // Cached Markdown converted from DSL
    isLoading: boolean;
    error: string | null;
    lastLoaded: string | null; // Timestamp of when data was last loaded
    dslSource?: string | null;
    sourceType?: 'dsl' | 'json' | null;
    currentExampleFile?: string | null;
    isConverting: boolean; // Track if conversion is in progress

    // Actions
    loadFromDSL: (json: ArchitectureJSON, dsl: string, file?: string | null) => void;
    setDslSource: (dsl: string | null, file?: string | null) => void;
    refreshConvertedJson: () => Promise<void>; // Refresh JSON when DSL changes
    updateArchitecture: (updater: (arch: ArchitectureJSON) => ArchitectureJSON) => Promise<void>; // Update architecture and sync DSL
    reset: () => void;
}


export const useArchitectureStore = create<ArchitectureState>()(
    persist(
        (set, get) => ({
            data: null,
            convertedJson: null,
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
                    data: json,
                    isLoading: false,
                    error: null,
                    lastLoaded: new Date().toISOString(),
                    sourceType: 'dsl',
                    dslSource: dsl,
                    currentExampleFile: file
                });

                // Convert DSL to JSON and Markdown in background and cache them
                if (dsl) {
                    set({ isConverting: true });
                    try {
                        const [convertedJson, convertedMarkdown] = await Promise.all([
                            convertDslToJson(dsl),
                            convertDslToMarkdown(dsl)
                        ]);

                        set({
                            convertedJson: convertedJson as ArchitectureJSON | null,
                            convertedMarkdown: convertedMarkdown,
                            isConverting: false
                        });
                    } catch (err) {
                        console.error('[architectureStore] Failed to convert DSL:', err);
                        set({ isConverting: false });
                    }
                }
            },

            setDslSource: async (dsl, file) => {
                const currentDsl = get().dslSource;
                set({ dslSource: dsl, sourceType: dsl ? 'dsl' : null, currentExampleFile: file ?? null });

                // If DSL changed, refresh converted JSON and Markdown in background
                if (dsl && dsl !== currentDsl) {
                    get().refreshConvertedJson();
                } else if (!dsl) {
                    // Clear converted data if DSL is removed
                    set({ convertedJson: null, convertedMarkdown: null });
                }
            },

            refreshConvertedJson: async () => {
                const dsl = get().dslSource;
                if (!dsl) {
                    set({ convertedJson: null, convertedMarkdown: null });
                    return;
                }

                set({ isConverting: true });
                try {
                    const [convertedJson, convertedMarkdown] = await Promise.all([
                        convertDslToJson(dsl),
                        convertDslToMarkdown(dsl)
                    ]);

                    set({
                        convertedJson: convertedJson as ArchitectureJSON | null,
                        convertedMarkdown: convertedMarkdown,
                        isConverting: false
                    });
                } catch (err) {
                    console.error('[architectureStore] Failed to refresh converted data:', err);
                    set({ isConverting: false });
                }
            },

            updateArchitecture: async (updater) => {
                const currentData = get().data;
                if (!currentData) {
                    console.warn('[architectureStore] Cannot update: no architecture data');
                    return;
                }

                // Update the JSON data
                const updatedData = updater(currentData);
                set({ data: updatedData });

                // Convert updated JSON to DSL
                try {
                    const newDsl = convertJsonToDsl(updatedData);
                    // Update DSL source and refresh
                    await get().setDslSource(newDsl, get().currentExampleFile);
                } catch (err) {
                    console.error('[architectureStore] Failed to convert JSON to DSL:', err);
                }
            },

            reset: () => {
                set({
                    data: null,
                    convertedJson: null,
                    convertedMarkdown: null,
                    isLoading: false,
                    error: null,
                    lastLoaded: null,
                    dslSource: null,
                    sourceType: null,
                    currentExampleFile: null,
                    isConverting: false
                });
            },
        }),
        {
            name: STORAGE_KEY,
            storage: createJSONStorage(() => localStorage),
            partialize: (state) => {
                // Return stable object reference - only create new object if values actually changed
                return {
                    data: state.data,
                    convertedJson: state.convertedJson,
                    convertedMarkdown: state.convertedMarkdown,
                    lastLoaded: state.lastLoaded,
                    dslSource: state.dslSource,
                    sourceType: state.sourceType,
                    currentExampleFile: state.currentExampleFile
                };
            },
            // Handle rehydration (when data is loaded from storage)
            onRehydrateStorage: () => (state, error) => {
                if (error) {
                    console.warn('Failed to rehydrate architecture store:', error);
                } else if (state?.data) {
                    console.log(`Architecture automatically loaded from localStorage (saved: ${state.lastLoaded || 'unknown'})`);
                }
            },
        }
    )
);

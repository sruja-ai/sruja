import { create } from 'zustand';
import type { ArchitectureJSON } from '../types';

interface ArchitectureState {
    data: ArchitectureJSON | null;
    isLoading: boolean;
    error: string | null;

    // Actions
    loadFromJSON: (json: ArchitectureJSON) => void;
    loadFromURL: (url: string) => Promise<void>;
    reset: () => void;
}

export const useArchitectureStore = create<ArchitectureState>((set) => ({
    data: null,
    isLoading: false,
    error: null,

    loadFromJSON: (json) => {
        set({ data: json, isLoading: false, error: null });
    },

    loadFromURL: async (url) => {
        set({ isLoading: true, error: null });
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Failed to fetch: ${response.statusText}`);
            }
            const json = await response.json();
            set({ data: json, isLoading: false, error: null });
        } catch (error) {
            set({
                isLoading: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            });
        }
    },

    reset: () => {
        set({ data: null, isLoading: false, error: null });
    },
}));

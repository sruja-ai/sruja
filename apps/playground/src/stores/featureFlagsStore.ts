// apps/playground/src/stores/featureFlagsStore.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

export interface FeatureFlags {
    // Mandatory features (always enabled)
    requirements: true;
    adrs: true;
    scenarios: true;
    flows: true;
    overview: true;

    // Optional features (can be toggled)
    policies: boolean;
    metadata: boolean;
    constraints: boolean;
    conventions: boolean;
    deployment: boolean;
    contracts: boolean;
    sharedArtifacts: boolean;
    libraries: boolean;
    imports: boolean;
}

const DEFAULT_FLAGS: FeatureFlags = {
    // Mandatory - always true
    requirements: true,
    adrs: true,
    scenarios: true,
    flows: true,
    overview: true,

    // Optional - default to false for less common features
    policies: true, // Common enough to default on
    metadata: true, // Common enough to default on
    constraints: false, // Less common
    conventions: false, // Less common
    deployment: false,
    contracts: false,
    sharedArtifacts: false,
    libraries: false,
    imports: false,
};

export type EditMode = 'view' | 'edit';

interface FeatureFlagsStore {
    flags: FeatureFlags;
    editMode: EditMode;
    setFlag: <K extends keyof FeatureFlags>(key: K, value: FeatureFlags[K]) => void;
    resetFlags: () => void;
    isEnabled: (key: keyof FeatureFlags) => boolean;
    setEditMode: (mode: EditMode) => void;
    isEditMode: () => boolean;
}

export const useFeatureFlagsStore = create<FeatureFlagsStore>()(
    persist(
        (set, get) => ({
            flags: DEFAULT_FLAGS,
            editMode: 'view' as EditMode,
            setFlag: (key, value) => {
                set((state) => ({
                    flags: {
                        ...state.flags,
                        [key]: value,
                    },
                }));
            },
            resetFlags: () => {
                set({ flags: DEFAULT_FLAGS });
            },
            isEnabled: (key) => {
                return get().flags[key] === true;
            },
            setEditMode: (mode) => {
                set({ editMode: mode });
            },
            isEditMode: () => {
                return get().editMode === 'edit';
            },
        }),
        {
            name: 'architecture-visualizer-feature-flags',
            storage: createJSONStorage(() => localStorage),
        }
    )
);

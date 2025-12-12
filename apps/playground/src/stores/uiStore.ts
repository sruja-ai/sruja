import { create } from 'zustand';

export type ViewTab = 'overview' | 'diagram' | 'guided' | 'requirements' | 'adrs' | 'code';
export type PendingActionType = 'create-requirement' | 'create-adr' | 'create-flow' | 'create-scenario' | null;

interface UIState {
    activeTab: ViewTab;
    setActiveTab: (tab: ViewTab) => void;

    // Pending action to execute after tab switch or component mount
    pendingAction: PendingActionType;
    setPendingAction: (action: PendingActionType) => void;
    clearPendingAction: () => void;
}

export const useUIStore = create<UIState>((set) => ({
    activeTab: 'overview', // Default will be set by App.tsx from URL
    setActiveTab: (tab) => set({ activeTab: tab }),

    pendingAction: null,
    setPendingAction: (action) => set({ pendingAction: action }),
    clearPendingAction: () => set({ pendingAction: null }),
}));

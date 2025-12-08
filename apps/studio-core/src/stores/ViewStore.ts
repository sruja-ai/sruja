import { create } from 'zustand'

export type ViewStep = 'context' | 'containers' | 'components' | 'stitch'

interface ViewState {
  activeStep: ViewStep
  focusTarget: string | null
  focusPath: string[]
  filterByStep: boolean

  setStep: (step: ViewStep) => void
  setFocusTarget: (target: string | null) => void
  setFocusPath: (path: string[]) => void
  setFilterByStep: (filter: boolean) => void
  navigateUp: () => void
}

export const useViewStore = create<ViewState>((set) => ({
  activeStep: 'context',
  focusTarget: null,
  focusPath: [],
  filterByStep: false,

  setStep: (step) => set({ activeStep: step }),
  setFocusTarget: (target) => set({ focusTarget: target }),
  setFocusPath: (path) => set({ focusPath: path }),
  setFilterByStep: (filter) => set({ filterByStep: filter }),
  navigateUp: () => set((state) => ({ focusPath: state.focusPath.slice(0, -1) })),
}))


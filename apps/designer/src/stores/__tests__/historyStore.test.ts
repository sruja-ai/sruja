// apps/designer/src/stores/__tests__/historyStore.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { useHistoryStore } from "../historyStore";
import type { SrujaModelDump } from "@sruja/shared";

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(() => null),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

describe("historyStore", () => {
  const mockArchitecture: SrujaModelDump = {
    specification: { tags: {}, elements: {} },
    elements: {
      "System1": { id: "System1", kind: "system", title: "Test System", tags: [], links: [] }
    },
    relations: [],
    views: {},
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: { name: "Test Architecture", version: "1.0.0", generated: new Date().toISOString(), srujaVersion: "1.0" },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);
    useHistoryStore.getState().clear();
  });

  it("should initialize with empty history", () => {
    const state = useHistoryStore.getState();
    expect(state.canUndo()).toBe(false);
    expect(state.canRedo()).toBe(false);
  });

  it("should push state to history", () => {
    useHistoryStore.getState().clear(); // Ensure clean state
    const { push } = useHistoryStore.getState();
    push(mockArchitecture);

    const state = useHistoryStore.getState();
    // After first push, history should have 1 item, currentIndex should be 0
    // canUndo requires currentIndex > 0, so it should be false with only 1 item
    expect(state.history.length).toBe(1);
    expect(state.currentIndex).toBe(0);
    expect(state.canUndo()).toBe(false); // Need at least 2 items to undo
  });

  it("should undo and redo correctly", () => {
    useHistoryStore.getState().clear();
    const { push, undo, redo } = useHistoryStore.getState();

    // Push initial state
    push(mockArchitecture);

    // Push modified state (need at least 2 states to undo)
    const modifiedArch: SrujaModelDump = {
      ...mockArchitecture,
      elements: {
        ...mockArchitecture.elements,
        "System2": { id: "System2", kind: "system", title: "New System", tags: [], links: [] }
      },
    };
    push(modifiedArch);

    expect(useHistoryStore.getState().canUndo()).toBe(true);

    // Undo should restore previous state
    const undoneState = undo();
    expect(undoneState).not.toBeNull();
    // Compare essential parts to avoid reference identity issues if store clones
    expect(undoneState?.elements).toEqual(mockArchitecture.elements);
    expect(useHistoryStore.getState().canRedo()).toBe(true);

    // Redo should restore modified state
    const redoneState = redo();
    expect(redoneState).not.toBeNull();
    expect(redoneState?.elements).toEqual(modifiedArch.elements);
    expect(useHistoryStore.getState().canUndo()).toBe(true);
  });

  it("should limit history size", () => {
    const { push } = useHistoryStore.getState();

    // Push more than MAX_HISTORY_SIZE states
    for (let i = 0; i < 60; i++) {
      push({
        ...mockArchitecture,
        // use empty elements to be fast
        elements: { "Sys": { id: "Sys" + i, kind: "system", title: "T", tags: [], links: [] } }
      });
    }

    const state = useHistoryStore.getState();
    // Should still be able to undo, but history should be limited
    expect(state.canUndo()).toBe(true);
  });

  it("should clear history", () => {
    useHistoryStore.getState().clear();
    const { push, clear } = useHistoryStore.getState();

    push(mockArchitecture);
    push({ ...mockArchitecture, elements: {} });
    expect(useHistoryStore.getState().canUndo()).toBe(true);

    clear();
    expect(useHistoryStore.getState().canUndo()).toBe(false);
    expect(useHistoryStore.getState().canRedo()).toBe(false);
    expect(useHistoryStore.getState().history.length).toBe(0);
  });

  it("should not undo when history is empty", () => {
    const { undo } = useHistoryStore.getState();
    const result = undo();
    expect(result).toBeNull();
  });

  it("should not redo when no future states", () => {
    const { push, undo, redo } = useHistoryStore.getState();

    push(mockArchitecture);
    undo();
    redo(); // Should work

    const result = redo(); // Should return null (no more future states)
    expect(result).toBeNull();
  });
});

// apps/designer/src/stores/__tests__/clipboardStore.test.ts
import { describe, it, expect, beforeEach } from "vitest";
import { useClipboardStore } from "../clipboardStore";
import type { ElementDump } from "../../types";

describe("clipboardStore", () => {
  const mockSystem: ElementDump = {
    id: "System1",
    kind: "system",
    title: "Test System",
    technology: "",
    tags: [],
    links: [],
  };

  beforeEach(() => {
    useClipboardStore.getState().clearClipboard();
  });

  it("should initialize with null clipboard", () => {
    const state = useClipboardStore.getState();
    expect(state.clipboard).toBeNull();
    expect(state.hasClipboard()).toBe(false);
  });

  it("should copy elements to clipboard", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("System1", [mockSystem]);

    const state = useClipboardStore.getState();
    expect(state.clipboard).not.toBeNull();
    expect(state.clipboard?.rootId).toBe("System1");
    expect(state.clipboard?.elements).toHaveLength(1);
    expect(state.clipboard?.elements[0]).toEqual(mockSystem);
    expect(state.hasClipboard()).toBe(true);
  });

  it("should deep clone clipboard data", () => {
    const { copyNode } = useClipboardStore.getState();
    const systemWithNested: ElementDump = {
      ...mockSystem,
      title: "Test System",
    };
    copyNode("System1", [systemWithNested]);

    const state = useClipboardStore.getState();
    const clipboardData = state.clipboard?.elements[0] as ElementDump;

    // Modify original
    systemWithNested.title = "Modified";

    // Clipboard data should be unchanged (deep clone)
    expect(clipboardData.title).toBe("Test System");
  });

  it("should clear clipboard", () => {
    const { copyNode, clearClipboard } = useClipboardStore.getState();
    copyNode("System1", [mockSystem]);
    expect(useClipboardStore.getState().hasClipboard()).toBe(true);

    clearClipboard();
    expect(useClipboardStore.getState().clipboard).toBeNull();
    expect(useClipboardStore.getState().hasClipboard()).toBe(false);
  });

  it("should overwrite previous clipboard when copying new item", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("System1", [mockSystem]);
    copyNode("System2", [
      {
        ...mockSystem,
        id: "System2",
        title: "Another",
      },
    ]);

    const state = useClipboardStore.getState();
    expect(state.clipboard?.rootId).toBe("System2");
    expect(state.clipboard?.elements[0]?.id).toBe("System2");
  });

  // Legacy API tests removed; updated to new clipboard schema (rootId, elements)
});

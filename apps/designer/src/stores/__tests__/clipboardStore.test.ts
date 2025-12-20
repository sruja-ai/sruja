// apps/designer/src/stores/__tests__/clipboardStore.test.ts
import { describe, it, expect, beforeEach } from "vitest";
import { useClipboardStore } from "../clipboardStore";
import type { SystemJSON, ContainerJSON, ComponentJSON, PersonJSON } from "../../types";

describe("clipboardStore", () => {
  const mockSystem: SystemJSON = {
    id: "System1",
    label: "Test System",
  };

  const mockContainer: ContainerJSON = {
    id: "Container1",
    label: "Test Container",
  };

  const mockComponent: ComponentJSON = {
    id: "Component1",
    label: "Test Component",
  };

  const mockPerson: PersonJSON = {
    id: "Person1",
    label: "Test Person",
  };

  beforeEach(() => {
    useClipboardStore.getState().clearClipboard();
  });

  it("should initialize with null clipboard", () => {
    const state = useClipboardStore.getState();
    expect(state.clipboard).toBeNull();
    expect(state.hasClipboard()).toBe(false);
  });

  it("should copy system to clipboard", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("system", mockSystem);

    const state = useClipboardStore.getState();
    expect(state.clipboard).not.toBeNull();
    expect(state.clipboard?.type).toBe("system");
    expect(state.clipboard?.data).toEqual(mockSystem);
    expect(state.hasClipboard()).toBe(true);
  });

  it("should copy container to clipboard with parent ID", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("container", mockContainer, "System1");

    const state = useClipboardStore.getState();
    expect(state.clipboard).not.toBeNull();
    expect(state.clipboard?.type).toBe("container");
    expect(state.clipboard?.data).toEqual(mockContainer);
    expect(state.clipboard?.parentId).toBe("System1");
  });

  it("should copy component to clipboard with parent IDs", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("component", mockComponent, "System1:Container1");

    const state = useClipboardStore.getState();
    expect(state.clipboard).not.toBeNull();
    expect(state.clipboard?.type).toBe("component");
    expect(state.clipboard?.data).toEqual(mockComponent);
    expect(state.clipboard?.parentId).toBe("System1:Container1");
  });

  it("should copy person to clipboard", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("person", mockPerson);

    const state = useClipboardStore.getState();
    expect(state.clipboard).not.toBeNull();
    expect(state.clipboard?.type).toBe("person");
    expect(state.clipboard?.data).toEqual(mockPerson);
  });

  it("should deep clone clipboard data", () => {
    const { copyNode } = useClipboardStore.getState();
    const systemWithNested = {
      ...mockSystem,
      containers: [mockContainer],
    };
    copyNode("system", systemWithNested);

    const state = useClipboardStore.getState();
    const clipboardData = state.clipboard?.data as SystemJSON;

    // Modify original
    systemWithNested.label = "Modified";

    // Clipboard data should be unchanged (deep clone)
    expect(clipboardData.label).toBe("Test System");
  });

  it("should clear clipboard", () => {
    const { copyNode, clearClipboard } = useClipboardStore.getState();
    copyNode("system", mockSystem);
    expect(useClipboardStore.getState().hasClipboard()).toBe(true);

    clearClipboard();
    expect(useClipboardStore.getState().clipboard).toBeNull();
    expect(useClipboardStore.getState().hasClipboard()).toBe(false);
  });

  it("should overwrite previous clipboard when copying new item", () => {
    const { copyNode } = useClipboardStore.getState();
    copyNode("system", mockSystem);
    copyNode("person", mockPerson);

    const state = useClipboardStore.getState();
    expect(state.clipboard?.type).toBe("person");
    expect(state.clipboard?.data).toEqual(mockPerson);
  });
});

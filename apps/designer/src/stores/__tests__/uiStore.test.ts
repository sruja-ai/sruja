// apps/designer/src/stores/__tests__/uiStore.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock zustand persist middleware
vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual<typeof import("zustand/middleware")>("zustand/middleware");
  return {
    ...actual,
    persist: <T>(config: T) => {
      if (typeof config === "function") {
        return config as unknown as T;
      }
      return config as unknown as T;
    },
    createJSONStorage: () => ({
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    }),
  };
});

import { useUIStore } from "../uiStore";
import type { ViewTab } from "../../types";
import type { Role } from "../../components/RoleSwitcher";

describe("uiStore", () => {
  beforeEach(() => {
    // Reset store before each test
    useUIStore.setState({
      activeTab: "overview",
      selectedRole: "architect",
      codeTab: "dsl",
      targetLine: null,
      pendingAction: null,
    });
  });

  it("should initialize with default values", () => {
    const state = useUIStore.getState();

    expect(state.activeTab).toBe("overview");
    expect(state.selectedRole).toBe("architect");
    expect(state.codeTab).toBe("dsl");
    expect(state.targetLine).toBeNull();
    expect(state.pendingAction).toBeNull();
  });

  it("should set active tab", () => {
    const tabs: ViewTab[] = ["overview", "diagram", "details", "code", "builder", "governance"];

    tabs.forEach((tab) => {
      useUIStore.getState().setActiveTab(tab);
      expect(useUIStore.getState().activeTab).toBe(tab);
    });
  });

  it("should set selected persona", () => {
    const roles: Role[] = ["product", "architect", "devops", "security", "cto", "sre"];

    roles.forEach((role) => {
      useUIStore.getState().setSelectedRole(role);
      expect(useUIStore.getState().selectedRole).toBe(role);
    });
  });

  it("should set code tab", () => {
    const codeTabs: Array<"dsl" | "json" | "markdown"> = ["dsl", "json", "markdown"];

    codeTabs.forEach((tab) => {
      useUIStore.getState().setCodeTab(tab);
      expect(useUIStore.getState().codeTab).toBe(tab);
    });
  });

  it("should set target line", () => {
    useUIStore.getState().setTargetLine(42);
    expect(useUIStore.getState().targetLine).toBe(42);

    useUIStore.getState().setTargetLine(null);
    expect(useUIStore.getState().targetLine).toBeNull();
  });

  it("should set pending action", () => {
    const actions: Array<"create-requirement" | "create-adr" | "create-flow" | "create-scenario"> =
      ["create-requirement", "create-adr", "create-flow", "create-scenario"];

    actions.forEach((action) => {
      useUIStore.getState().setPendingAction(action);
      expect(useUIStore.getState().pendingAction).toBe(action);
    });

    useUIStore.getState().setPendingAction(null);
    expect(useUIStore.getState().pendingAction).toBeNull();
  });

  it("should clear pending action", () => {
    useUIStore.getState().setPendingAction("create-requirement");
    expect(useUIStore.getState().pendingAction).toBe("create-requirement");

    useUIStore.getState().clearPendingAction();
    expect(useUIStore.getState().pendingAction).toBeNull();
  });

  it("should handle multiple state updates independently", () => {
    useUIStore.getState().setActiveTab("diagram");
    useUIStore.getState().setSelectedRole("product");
    useUIStore.getState().setCodeTab("json");
    useUIStore.getState().setTargetLine(100);
    useUIStore.getState().setPendingAction("create-adr");

    const state = useUIStore.getState();
    expect(state.activeTab).toBe("diagram");
    expect(state.selectedRole).toBe("product");
    expect(state.codeTab).toBe("json");
    expect(state.targetLine).toBe(100);
    expect(state.pendingAction).toBe("create-adr");
  });

  it("should allow chaining state updates", () => {
    useUIStore.setState({
      activeTab: "details",
      selectedRole: "devops",
      codeTab: "markdown",
    });

    const state = useUIStore.getState();
    expect(state.activeTab).toBe("details");
    expect(state.selectedRole).toBe("devops");
    expect(state.codeTab).toBe("markdown");
  });
});

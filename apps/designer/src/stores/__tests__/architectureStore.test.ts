// apps/designer/src/stores/__tests__/architectureStore.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock zustand persist middleware - MUST be before any store imports
vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual<typeof import("zustand/middleware")>("zustand/middleware");
  return {
    ...actual,
    persist: <T,>(config: T) => {
      // Return config directly (bypass persistence in tests)
      if (typeof config === "function") {
        return config as any;
      }
      return config as any;
    },
    createJSONStorage: () => ({
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    }),
  };
});

import { useArchitectureStore } from "../architectureStore";
import type { SrujaModelDump } from "@sruja/shared";

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(() => null),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, "localStorage", { value: localStorageMock });

// Mock dependencies
const mockClear = vi.fn();
const mockPush = vi.fn();
vi.mock("../historyStore", () => ({
  useHistoryStore: {
    getState: vi.fn(() => ({
      clear: mockClear,
      push: mockPush,
    })),
  },
}));

vi.mock("../../wasm", () => ({
  convertDslToJson: vi.fn().mockResolvedValue({
    specification: { tags: {}, elements: {} },
    elements: {},
    relations: [],
    views: {},
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: { name: "Test", version: "1.0", generated: new Date().toISOString(), srujaVersion: "1.0" },
  }),
  convertDslToMarkdown: vi.fn().mockResolvedValue("# Test"),
  convertDslToLikeC4: vi.fn().mockResolvedValue({
    specification: { tags: {}, elements: {} },
    elements: {}, // Return empty valid dump
    relations: [],
    views: {},
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: { name: "Test", version: "1.0", generated: new Date().toISOString(), srujaVersion: "1.0" }
  })
}));

vi.mock("../../utils/jsonToDsl", () => ({
  convertJsonToDsl: vi.fn().mockResolvedValue("system TestSystem"),
}));

describe("architectureStore", () => {
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
    // Reset store state before each test
    useArchitectureStore.getState().reset();
  });

  it("should initialize with null data", () => {
    // Reset store before test
    useArchitectureStore.getState().reset();
    const state = useArchitectureStore.getState();
    expect(state.likec4Model).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("should load architecture from DSL", async () => {
    useArchitectureStore.getState().reset();
    const { convertDslToLikeC4, convertDslToMarkdown } = await import("../../wasm");
    vi.mocked(convertDslToLikeC4).mockResolvedValue(mockArchitecture);
    vi.mocked(convertDslToMarkdown).mockResolvedValue("# Test Architecture");

    const dsl = "system TestSystem";
    const file = "test.sruja";

    await useArchitectureStore.getState().loadFromDSL(mockArchitecture, dsl, file);

    const state = useArchitectureStore.getState();
    expect(state.likec4Model).toEqual(mockArchitecture);
    expect(state.dslSource).toBe(dsl);
    expect(state.sourceType).toBe("dsl");
    expect(state.currentExampleFile).toBe(file);
    expect(state.isLoading).toBe(false);
  });

  it("should update architecture", async () => {
    useArchitectureStore.getState().reset();
    const { convertJsonToDsl } = await import("../../utils/jsonToDsl");
    vi.mocked(convertJsonToDsl).mockResolvedValue("system UpdatedSystem");

    // First load some data
    await useArchitectureStore.getState().loadFromDSL(mockArchitecture, "system TestSystem");

    const updater = (arch: SrujaModelDump): SrujaModelDump => {
      return {
        ...arch,
        elements: {
          ...arch.elements,
          "System2": { id: "System2", kind: "system", title: "New System", tags: [], links: [] }
        }
      };
    };

    await useArchitectureStore.getState().updateArchitecture(updater);

    const state = useArchitectureStore.getState();
    expect(state.likec4Model).not.toBeNull();
    expect(state.likec4Model?.elements).toBeDefined();
    expect(Object.keys(state.likec4Model?.elements || {}).length).toBeGreaterThanOrEqual(1);
  });

  it("should reset store", () => {
    useArchitectureStore.getState().reset();
    // Load some data first
    useArchitectureStore.getState().loadFromDSL(mockArchitecture, "system TestSystem");

    useArchitectureStore.getState().reset();

    const state = useArchitectureStore.getState();
    expect(state.likec4Model).toBeNull();
    expect(state.dslSource).toBeNull();
    expect(state.error).toBeNull();
  });

  it("should handle conversion errors gracefully", async () => {
    const { convertDslToLikeC4 } = await import("../../wasm");
    vi.mocked(convertDslToLikeC4).mockRejectedValue(new Error("Conversion failed"));

    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => { });

    await useArchitectureStore.getState().loadFromDSL(mockArchitecture, "invalid dsl");

    // Should not throw, but log error
    expect(consoleErrorSpy).toHaveBeenCalled();

    consoleErrorSpy.mockRestore();
  });
});

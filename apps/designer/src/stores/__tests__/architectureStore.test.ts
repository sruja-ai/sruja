// apps/designer/src/stores/__tests__/architectureStore.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock zustand persist middleware - MUST be before any store imports
vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual<typeof import("zustand/middleware")>("zustand/middleware");
  return {
    ...actual,
    persist: <T>(config: T) => {
      // Return config directly (bypass persistence in tests)
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
    _metadata: {
      name: "Test",
      version: "1.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0",
    },
  }),
  convertDslToMarkdown: vi.fn().mockResolvedValue("# Test"),
  convertDslToModel: vi.fn().mockResolvedValue({
    specification: { tags: {}, elements: {} },
    elements: {}, // Return empty valid dump
    relations: [],
    views: {},
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: {
      name: "Test",
      version: "1.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0",
    },
  }),
}));

vi.mock("../../utils/jsonToDsl", () => ({
  convertJsonToDsl: vi.fn().mockResolvedValue("system TestSystem"),
}));

vi.mock("../../utils/modelToDsl", () => ({
  convertModelToDsl: vi.fn().mockResolvedValue("system TestSystem"),
}));
describe("architectureStore", () => {
  const mockArchitecture: SrujaModelDump = {
    specification: { tags: {}, elements: {} },
    elements: {
      System1: { id: "System1", kind: "system", title: "Test System", tags: [], links: [] },
    },
    relations: [],
    views: {},
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: {
      name: "Test Architecture",
      version: "1.0.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0",
    },
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
    expect(state.model).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("should load architecture from DSL", async () => {
    useArchitectureStore.getState().reset();
    const { convertDslToModel, convertDslToMarkdown } = await import("../../wasm");
    vi.mocked(convertDslToModel).mockResolvedValue(mockArchitecture);
    vi.mocked(convertDslToMarkdown).mockResolvedValue("# Test Architecture");

    const dsl = "system TestSystem";
    const file = "test.sruja";

    await useArchitectureStore.getState().loadFromDSL(mockArchitecture, dsl, file);

    const state = useArchitectureStore.getState();
    expect(state.model).toEqual(mockArchitecture);
    expect(state.dslSource).toBe(dsl);
    expect(state.sourceType).toBe("dsl");
    expect(state.currentExampleFile).toBe(file);
    expect(state.isLoading).toBe(false);
  });

  it("should update architecture", async () => {
    useArchitectureStore.getState().reset();
    const modelToDsl = await import("../../utils/modelToDsl");
    vi.mocked(modelToDsl.convertModelToDsl).mockResolvedValue("system UpdatedSystem");

    // First load some data
    await useArchitectureStore.getState().loadFromDSL(mockArchitecture, "system TestSystem");

    const updater = (arch: SrujaModelDump): SrujaModelDump => {
      return {
        ...arch,
        elements: {
          ...arch.elements,
          System2: { id: "System2", kind: "system", title: "New System", tags: [], links: [] },
        },
      };
    };

    await useArchitectureStore.getState().updateArchitecture(updater);

    const state = useArchitectureStore.getState();
    expect(state.model).not.toBeNull();
    expect(state.model?.elements).toBeDefined();
    expect(Object.keys(state.model?.elements || {}).length).toBeGreaterThanOrEqual(1);
  });

  it("should reset store", () => {
    useArchitectureStore.getState().reset();
    // Load some data first
    useArchitectureStore.getState().loadFromDSL(mockArchitecture, "system TestSystem");

    useArchitectureStore.getState().reset();

    const state = useArchitectureStore.getState();
    expect(state.model).toBeNull();
    expect(state.dslSource).toBeNull();
    expect(state.error).toBeNull();
  });

  it("should handle conversion errors gracefully", async () => {
    const wasm = await import("../../wasm");
    vi.mocked(wasm.convertDslToModel).mockRejectedValue(new Error("Conversion failed"));
    const shared = await import("@sruja/shared");
    const errorSpy = vi.spyOn(shared.logger, "error").mockImplementation(() => {});

    await useArchitectureStore.getState().setDslSource("invalid dsl", "test.sruja");
    await new Promise<void>((resolve) => {
      const check = () => {
        const s = useArchitectureStore.getState();
        if (!s.isConverting) resolve();
        else setTimeout(check, 10);
      };
      check();
    });

    // Should not throw, but log error
    expect(errorSpy).toHaveBeenCalled();
    errorSpy.mockRestore();
  });

  describe("Bidirectional Sync", () => {
    it("Builder → DSL: updateArchitecture converts model to DSL", async () => {
      useArchitectureStore.getState().reset();
      const modelToDsl = await import("../../utils/modelToDsl");
      vi.mocked(modelToDsl.convertModelToDsl).mockResolvedValue("system SyncedSystem");

      const updater = (model: SrujaModelDump): SrujaModelDump => {
        return {
          ...model,
          elements: {
            ...model.elements,
            newsystem: {
              id: "newsystem",
              kind: "system",
              title: "SyncedSystem",
              technology: "",
              tags: [],
              links: [],
            },
          },
        };
      };

      await useArchitectureStore.getState().updateArchitecture(updater);

      const state = useArchitectureStore.getState();
      expect(state.dslSource).toBe("system SyncedSystem");
      expect(state.model?.elements["newsystem"]?.title).toBe("SyncedSystem");
      expect(modelToDsl.convertModelToDsl).toHaveBeenCalled();
    });

    it("DSL → Model: setDslSource converts DSL to model", async () => {
      useArchitectureStore.getState().reset();
      const { convertDslToModel } = await import("../../wasm");
      const mockModel: SrujaModelDump = {
        ...mockArchitecture,
        elements: {
          test: {
            id: "test",
            kind: "system",
            title: "TestSystem",
            technology: "",
            tags: [],
            links: [],
          },
        },
      };
      vi.mocked(convertDslToModel).mockResolvedValue(mockModel);

      const dsl = "system TestSystem";
      await useArchitectureStore.getState().setDslSource(dsl, "test.sruja");
      await new Promise<void>((resolve) => {
        const check = () => {
          const s = useArchitectureStore.getState();
          if (!s.isConverting) resolve();
          else setTimeout(check, 10);
        };
        check();
      });
      const state = useArchitectureStore.getState();
      expect(state.model).toEqual(mockModel);
      expect(convertDslToModel).toHaveBeenCalledWith(dsl);
      expect(state.dslSource).toBe(dsl);
    });

    it("round-trip: Model → DSL → Model preserves data", async () => {
      useArchitectureStore.getState().reset();
      const { convertModelToDsl } = await import("../../utils/modelToDsl");
      const { convertDslToModel } = await import("../../wasm");
      vi.mocked(convertModelToDsl).mockResolvedValue("system System1");

      const originalModel: SrujaModelDump = {
        elements: {
          sys1: {
            id: "sys1",
            kind: "system",
            title: "System1",
            technology: "",
            tags: [],
            links: [],
          },
        },
        relations: [],
        views: {},
        _metadata: {
          name: "Test",
          version: "1.0",
          generated: new Date().toISOString(),
          srujaVersion: "1.0",
        },
      };

      // Load model
      await useArchitectureStore.getState().loadFromModel(originalModel);

      // Get DSL
      const dslSource = useArchitectureStore.getState().dslSource;
      expect(dslSource).toContain("System1");

      // Simulate DSL round-trip
      vi.mocked(convertDslToModel).mockResolvedValue(originalModel);
      await useArchitectureStore.getState().setDslSource(dslSource!, null);
      await new Promise<void>((resolve) => {
        const check = () => {
          const s = useArchitectureStore.getState();
          if (!s.isConverting) resolve();
          else setTimeout(check, 10);
        };
        check();
      });

      // Verify model preserved
      const finalModel = useArchitectureStore.getState().model;
      expect(finalModel?.elements["sys1"]?.title).toBe("System1");
    });

    it("sets correct sourceType for DSL updates", async () => {
      useArchitectureStore.getState().reset();
      const { convertDslToModel } = await import("../../wasm");
      vi.mocked(convertDslToModel).mockResolvedValue(mockArchitecture);

      await useArchitectureStore.getState().setDslSource("system Test", "test.sruja");

      const state = useArchitectureStore.getState();
      expect(state.sourceType).toBe("dsl");
    });
  });
});

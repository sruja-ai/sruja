// apps/designer/src/hooks/__tests__/useDSLSync.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import type { SrujaModelDump } from "@sruja/shared";

// Mock zustand persist middleware BEFORE importing store
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

// Mock WASM module
vi.mock("../../wasm", () => ({
  convertDslToModel: vi.fn(),
  convertDslToMarkdown: vi.fn(),
}));

const { convertDslToModel } = await import("../../wasm");

// Now import store and hook after mocks are set up
import { useDSLSync } from "../useDSLSync";
import { useArchitectureStore } from "../../stores/architectureStore";

describe("useDSLSync - Bidirectional Sync", () => {
  beforeEach(() => {
    // Reset store before each test
    useArchitectureStore.getState().reset();
    vi.clearAllMocks();
  });

  describe("DSL → Model Sync", () => {
    it("parses DSL and updates model on valid DSL input", async () => {
      const mockModel: SrujaModelDump = {
        elements: {
          testsystem: {
            id: "testsystem",
            kind: "system",
            title: "TestSystem",
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

      vi.mocked(convertDslToModel).mockResolvedValue(mockModel);

      const { result } = renderHook(() => useDSLSync());

      // Trigger DSL change
      await result.current.handleDSLChange("system TestSystem");

      // Verify model was updated
      await waitFor(
        () => {
          const model = useArchitectureStore.getState().model;
          expect(model?.elements["testsystem"]?.title).toBe("TestSystem");
        },
        { timeout: 3000 }
      );

      expect(convertDslToModel).toHaveBeenCalledWith("system TestSystem");
    });

    it("shows error on invalid DSL syntax", async () => {
      const errorMessage = "Failed to parse DSL. Invalid syntax at line 1";
      vi.mocked(convertDslToModel).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useDSLSync());

      // Trigger invalid DSL change
      await result.current.handleDSLChange("invalid dsl syntax");

      // Verify error state
      await waitFor(
        () => {
          expect(result.current.error).toBeTruthy();
          expect(result.current.error).toContain("Failed to parse DSL");
        },
        { timeout: 3000 }
      );

      // Model should not be updated on error
      const model = useArchitectureStore.getState().model;
      expect(model).toBeNull();
    });

    it("sets saving state during DSL processing", async () => {
      let resolvePromise: () => void;
      const pendingPromise = new Promise<void>((resolve) => {
        resolvePromise = resolve;
      });

      vi.mocked(convertDslToModel).mockReturnValue(
        pendingPromise as unknown as Promise<SrujaModelDump>
      );

      const { result } = renderHook(() => useDSLSync());

      // Trigger DSL change
      const changePromise = result.current.handleDSLChange("system Test");

      // Verify saving state is true during processing
      await waitFor(
        () => {
          expect(result.current.isSaving).toBe(true);
        },
        { timeout: 3000 }
      );

      // Resolve and wait for completion
      resolvePromise!();
      await changePromise;

      // Verify saving state is false after completion
      await waitFor(
        () => {
          expect(result.current.isSaving).toBe(false);
        },
        { timeout: 3000 }
      );
    });
  });

  describe("Model → DSL Sync", () => {
    it("syncs DSL source when store DSL changes externally", async () => {
      const { result } = renderHook(() => useDSLSync());

      // Simulate external DSL update (e.g., from Builder)
      const store = useArchitectureStore.getState();
      await store.setDslSource("system ExternallyUpdated");

      // Verify hook's DSL is updated
      await waitFor(
        () => {
          expect(result.current.dslSource).toBe("system ExternallyUpdated");
          expect(result.current.error).toBeNull();
        },
        { timeout: 3000 }
      );
    });
  });

  describe("Edge Cases", () => {
    it("handles empty DSL input", async () => {
      const { result } = renderHook(() => useDSLSync());

      await result.current.handleDSLChange("");

      // Store should have empty DSL
      await waitFor(
        () => {
          const dsl = useArchitectureStore.getState().dslSource;
          expect(dsl).toBe("");
        },
        { timeout: 3000 }
      );
    });
  });
});

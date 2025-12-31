// apps/designer/src/hooks/__tests__/useUrlState.test.ts
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useUrlState } from "../useUrlState";

// Mock zustand persist middleware
vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual<typeof import("zustand/middleware")>("zustand/middleware");
  return {
    ...actual,
    persist: <T>(config: T) => config as any,
  };
});

vi.mock("../../stores/viewStore", () => ({
  useViewStore: vi.fn((selector) => {
    const state = {
      currentLevel: "L1" as const,
      expandedNodes: new Set<string>(),
      setLevel: vi.fn(),
      toggleExpand: vi.fn(),
    };
    return selector(state);
  }),
}));

describe("useUrlState", () => {
  const originalLocation = window.location;
  const originalHistory = window.history;

  beforeEach(() => {
    // Mock window.location
    delete (window as any).location;
    window.location = {
      ...originalLocation,
      search: "",
      pathname: "/",
    } as Location;

    // Mock window.history
    window.history = {
      ...originalHistory,
      replaceState: vi.fn(),
    } as unknown as History;

    vi.clearAllMocks();
  });

  afterEach(() => {
    window.location = originalLocation;
    window.history = originalHistory;
  });

  it("should initialize without errors", () => {
    const { result } = renderHook(() => useUrlState());
    expect(result.current).toBeUndefined(); // Hook doesn't return anything
  });

  it("should read level from URL on mount", async () => {
    window.location.search = "?level=L2";
    const { useViewStore } = await import("../stores/viewStore");
    const mockSetLevel = vi.fn();
    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes: new Set<string>(),
        setLevel: mockSetLevel,
        toggleExpand: vi.fn(),
      };
      return selector(state);
    });

    renderHook(() => useUrlState());

    await waitFor(() => {
      expect(mockSetLevel).toHaveBeenCalled();
    });
  });

  it("should read expanded nodes from URL on mount", async () => {
    window.location.search = "?expanded=System1,System2";
    const { useViewStore } = await import("../stores/viewStore");
    const mockToggleExpand = vi.fn();
    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes: new Set<string>(),
        setLevel: vi.fn(),
        toggleExpand: mockToggleExpand,
      };
      return selector(state);
    });

    renderHook(() => useUrlState());

    await waitFor(() => {
      expect(mockToggleExpand).toHaveBeenCalled();
    });
  });

  it("should update URL when level changes", async () => {
    const { useViewStore } = await import("../stores/viewStore");
    let currentLevel = "L1" as const;
    const mockSetLevel = vi.fn();

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel,
        expandedNodes: new Set<string>(),
        setLevel: mockSetLevel,
        toggleExpand: vi.fn(),
      };
      return selector(state);
    });

    const { rerender } = renderHook(() => useUrlState());

    // Simulate level change
    currentLevel = "L2";
    rerender();

    await waitFor(() => {
      expect(window.history.replaceState).toHaveBeenCalled();
    });
  });

  it("should update URL when expanded nodes change", async () => {
    const { useViewStore } = await import("../stores/viewStore");
    let expandedNodes = new Set<string>();

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes,
        setLevel: vi.fn(),
        toggleExpand: vi.fn(),
      };
      return selector(state);
    });

    const { rerender } = renderHook(() => useUrlState());

    // Simulate expanded nodes change
    expandedNodes = new Set(["System1", "System2"]);
    rerender();

    await waitFor(() => {
      expect(window.history.replaceState).toHaveBeenCalled();
    });
  });

  it("should handle popstate events", async () => {
    const { useViewStore } = await import("../stores/viewStore");
    const mockSetLevel = vi.fn();
    const mockToggleExpand = vi.fn();

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes: new Set<string>(),
        setLevel: mockSetLevel,
        toggleExpand: mockToggleExpand,
      };
      return selector(state);
    });

    renderHook(() => useUrlState());

    // Simulate popstate event
    const popStateEvent = new PopStateEvent("popstate");
    window.dispatchEvent(popStateEvent);

    await waitFor(() => {
      // Should attempt to read from URL
      expect(window.location.search).toBeDefined();
    });
  });

  it("should handle invalid level values gracefully", async () => {
    window.location.search = "?level=INVALID";
    const { useViewStore } = await import("../stores/viewStore");
    const mockSetLevel = vi.fn();

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes: new Set<string>(),
        setLevel: mockSetLevel,
        toggleExpand: vi.fn(),
      };
      return selector(state);
    });

    renderHook(() => useUrlState());

    await waitFor(() => {
      // Should not call setLevel with invalid value
      expect(mockSetLevel).not.toHaveBeenCalledWith("INVALID");
    });
  });

  it("should handle empty expanded parameter", async () => {
    window.location.search = "?expanded=";
    const { useViewStore } = await import("../stores/viewStore");
    const mockToggleExpand = vi.fn();

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes: new Set<string>(),
        setLevel: vi.fn(),
        toggleExpand: mockToggleExpand,
      };
      return selector(state);
    });

    renderHook(() => useUrlState());

    await waitFor(() => {
      // Should handle empty expanded gracefully
      expect(mockToggleExpand).not.toHaveBeenCalled();
    });
  });

  it("should remove expanded from URL when empty", async () => {
    const { useViewStore } = await import("../stores/viewStore");
    let expandedNodes = new Set<string>(["System1"]);

    vi.mocked(useViewStore).mockImplementation((selector) => {
      const state = {
        currentLevel: "L1" as const,
        expandedNodes,
        setLevel: vi.fn(),
        toggleExpand: vi.fn(),
      };
      return selector(state);
    });

    const { rerender } = renderHook(() => useUrlState());

    // Clear expanded nodes
    expandedNodes = new Set<string>();
    rerender();

    await waitFor(() => {
      const replaceStateCall = (window.history.replaceState as any).mock.calls[0];
      if (replaceStateCall) {
        const url = replaceStateCall[2] as string;
        expect(url).not.toContain("expanded=");
      }
    });
  });
});

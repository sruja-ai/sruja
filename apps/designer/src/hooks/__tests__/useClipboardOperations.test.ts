// apps/designer/src/hooks/__tests__/useClipboardOperations.test.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useClipboardOperations } from "../useClipboardOperations";
import type { SrujaModelDump } from "@sruja/shared";
import type { ArchitectureCanvasRef } from "../../components/Canvas/types";

// Mock dependencies
const mockCopyNode = vi.fn();
const mockUpdateArchitecture = vi.fn();
const mockSelectNode = vi.fn();
const mockShowToast = vi.fn();
const mockGenerateUniqueId = vi.fn();
const mockGetAllNodeIds = vi.fn();
const mockFindNodeInArchitecture = vi.fn();
const mockGetReactFlowInstance = vi.fn();

vi.mock("../../stores/clipboardStore", () => ({
  useClipboardStore: vi.fn((selector) => {
    return selector({
      clipboard: null,
      copyNode: mockCopyNode,
      clearClipboard: vi.fn(),
      hasClipboard: vi.fn(() => false),
    });
  }),
}));

vi.mock("../../stores/architectureStore", () => ({
  useArchitectureStore: vi.fn((selector) => {
    const mockData: SrujaModelDump = {
      specification: { elements: {}, tags: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "Test System", tags: [], links: [] },
      },
      relations: [],
      views: {},
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "0.0.1",
      },
    };
    return selector({
      model: mockData,
      updateArchitecture: mockUpdateArchitecture,
    });
  }),
}));

vi.mock("../../stores/viewStore", () => ({
  useSelectionStore: vi.fn((selector) => {
    return selector({
      selectedNodeId: "System1",
      selectNode: mockSelectNode,
    });
  }),
  useViewStore: {
    getState: vi.fn(() => ({
      focusedSystemId: null,
      focusedContainerId: null,
    })),
  },
}));

vi.mock("../../stores/toastStore", () => ({
  useToastStore: vi.fn((selector) => {
    return selector({ showToast: mockShowToast });
  }),
}));

vi.mock("../../utils/nodeUtils", () => ({
  generateUniqueId: mockGenerateUniqueId,
  getAllNodeIds: mockGetAllNodeIds,
  findNodeInArchitecture: mockFindNodeInArchitecture,
}));

describe("useClipboardOperations", () => {
  const mockCanvasRef = {
    current: {
      getReactFlowInstance: mockGetReactFlowInstance,
    } as unknown as ArchitectureCanvasRef,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockGetAllNodeIds.mockReturnValue(new Set(["System1"]));
    mockGenerateUniqueId.mockReturnValue("System1-copy");
    mockUpdateArchitecture.mockResolvedValue(undefined);
    mockGetReactFlowInstance.mockReturnValue({
      getNode: vi.fn(() => ({
        id: "System1",
        data: { id: "System1", label: "Test System", type: "system" },
      })),
    });
    mockFindNodeInArchitecture.mockReturnValue({
      type: "system",
      node: { id: "System1", label: "Test System" },
    });
  });

  it("should initialize with correct handlers", () => {
    const { result } = renderHook(() => useClipboardOperations(mockCanvasRef));

    expect(result.current.handleCopy).toBeDefined();
    expect(result.current.handlePaste).toBeDefined();
    expect(result.current.handleDuplicate).toBeDefined();
  });

  it("should copy node to clipboard", () => {
    const { result } = renderHook(() => useClipboardOperations(mockCanvasRef));

    result.current.handleCopy();

    expect(mockCopyNode).toHaveBeenCalledWith(
      "system",
      expect.objectContaining({ id: "System1" }),
      undefined
    );
    expect(mockShowToast).toHaveBeenCalledWith("Copied to clipboard", "success");
  });

  it("should paste node from clipboard", async () => {
    // Mock clipboard with data
    const { useClipboardStore } = await import("../../stores/clipboardStore");
    vi.mocked(useClipboardStore).mockImplementation((selector) => {
      return selector({
        clipboard: {
          type: "system" as const,
          data: { id: "System2", label: "Copied System" },
        },
        copyNode: mockCopyNode,
        clearClipboard: vi.fn(),
        hasClipboard: vi.fn(() => true),
      });
    });

    const { result } = renderHook(() => useClipboardOperations(mockCanvasRef));

    await result.current.handlePaste();

    expect(mockUpdateArchitecture).toHaveBeenCalled();
  });

  it("should duplicate selected node", async () => {
    // Mock clipboard to be set after copy
    const { useClipboardStore } = await import("../../stores/clipboardStore");
    let clipboardState: any = null;
    vi.mocked(useClipboardStore).mockImplementation((selector) => {
      return selector({
        clipboard: clipboardState,
        copyNode: (type: any, data: any) => {
          clipboardState = { type, data };
        },
        clearClipboard: vi.fn(),
        hasClipboard: vi.fn(() => !!clipboardState),
      });
    });

    const { result } = renderHook(() => useClipboardOperations(mockCanvasRef));

    await result.current.handleDuplicate();

    expect(mockUpdateArchitecture).toHaveBeenCalled();
    expect(mockGenerateUniqueId).toHaveBeenCalled();
  });

  it("should not copy if no node is selected", async () => {
    const { useSelectionStore } = await import("../../stores/viewStore");
    vi.mocked(useSelectionStore).mockImplementation((selector) => {
      return selector({
        selectedNodeId: null,
        selectNode: mockSelectNode,
      } as any);
    });

    const { result } = renderHook(() => useClipboardOperations(mockCanvasRef));

    result.current.handleCopy();

    expect(mockCopyNode).not.toHaveBeenCalled();
  });
});

// apps/designer/src/hooks/__tests__/useFileHandlers.test.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useFileHandlers } from "../useFileHandlers";
import type { SrujaModelDump } from "@sruja/shared";
import type { ArchitectureCanvasRef } from "../../components/Canvas/LikeC4Canvas";

// Mock dependencies
const mockLoadFromDSL = vi.fn();
const mockShowToast = vi.fn();
const mockSetActiveTab = vi.fn();
const mockConvertJsonToDsl = vi.fn();
const mockCreateNewProject = vi.fn();
const mockSaveProject = vi.fn();
const mockBuildShareUrl = vi.fn();

// Helper

vi.mock("../../stores/architectureStore", () => ({
  useArchitectureStore: vi.fn((selector) => {
    const mockData: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {},
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
    const state = {
      data: mockData,
      dslSource: null,
      loadFromDSL: mockLoadFromDSL,
    };
    return selector(state);
  }),
}));

vi.mock("../../stores/toastStore", () => ({
  useToastStore: vi.fn((selector) => {
    return selector({ showToast: mockShowToast });
  }),
}));

vi.mock("../../stores/uiStore", () => ({
  useUIStore: vi.fn((selector) => {
    return selector({ setActiveTab: mockSetActiveTab });
  }),
}));

vi.mock("../../utils/jsonToDsl", () => ({
  convertJsonToDsl: mockConvertJsonToDsl,
}));

vi.mock("../../utils/firebaseShareService", () => ({
  firebaseShareService: {
    parseUrl: vi.fn(() => ({ projectId: null, keyBase64: null })),
    getCurrentProjectId: vi.fn(() => null),
    createNewProject: mockCreateNewProject,
    saveProject: mockSaveProject,
    buildShareUrl: mockBuildShareUrl,
  },
}));

vi.mock("../../wasm", () => ({
  convertDslToJson: vi.fn(),
  convertDslToLikeC4: vi.fn(), // Add this if needed
}));

vi.mock("../../utils/errorHandling", () => ({
  handleError: vi.fn((error) => error),
  getUserFriendlyMessage: vi.fn((error) => error?.message || "An error occurred"),
  safeAsync: vi.fn(async (operation) => {
    try {
      const data = await operation();
      return { data, error: null };
    } catch (error) {
      return { data: null, error };
    }
  }),
  NetworkError: class NetworkError extends Error {},
  ErrorType: {
    NETWORK: "NETWORK",
    VALIDATION: "VALIDATION",
    PERMISSION: "PERMISSION",
    NOT_FOUND: "NOT_FOUND",
    UNKNOWN: "UNKNOWN",
  },
}));

describe("useFileHandlers", () => {
  const mockCanvasRef = {
    current: {
      exportAsPNG: vi.fn(),
      exportAsSVG: vi.fn(),
    } as unknown as ArchitectureCanvasRef,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockConvertJsonToDsl.mockResolvedValue("system TestSystem");
    mockCreateNewProject.mockResolvedValue({ projectId: "test", keyBase64: "key" });
    mockSaveProject.mockResolvedValue(undefined);
    mockBuildShareUrl.mockReturnValue("https://example.com/share");
  });

  it("should initialize with correct default values", () => {
    const { result } = renderHook(() => useFileHandlers(mockCanvasRef));

    expect(result.current.isImporting).toBe(false);
    expect(result.current.fileInputRef).toBeDefined();
  });

  it("should handle export correctly", () => {
    const { result } = renderHook(() => useFileHandlers(mockCanvasRef));

    // Mock document.createElement and related DOM APIs
    const mockAnchor = {
      href: "",
      download: "",
      click: vi.fn(),
    };
    const createElementSpy = vi
      .spyOn(document, "createElement")
      .mockReturnValue(mockAnchor as unknown as HTMLElement);
    const appendChildSpy = vi
      .spyOn(document.body, "appendChild")
      .mockImplementation(() => mockAnchor as unknown as Node);
    const removeChildSpy = vi
      .spyOn(document.body, "removeChild")
      .mockImplementation(() => mockAnchor as unknown as Node);
    const createObjectURLSpy = vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:url");

    result.current.handleExport();

    expect(mockConvertJsonToDsl).toHaveBeenCalled();
    expect(createElementSpy).toHaveBeenCalledWith("a");
    expect(mockAnchor.download).toBe("Test Architecture.sruja");
    expect(mockAnchor.click).toHaveBeenCalled();

    createElementSpy.mockRestore();
    appendChildSpy.mockRestore();
    removeChildSpy.mockRestore();
    createObjectURLSpy.mockRestore();
  });

  it("should handle import correctly", async () => {
    const { convertDslToJson } = await import("../../wasm");
    const mockConvertDslToJson = vi.mocked(convertDslToJson);
    mockConvertDslToJson.mockResolvedValue({
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
    });

    const { result } = renderHook(() => useFileHandlers(mockCanvasRef));

    // Mock file input
    const mockFileReader = {
      readAsText: vi.fn((_file: File) => {
        setTimeout(() => {
          if (mockFileReader.onload) {
            mockFileReader.onload({
              target: { result: "system TestSystem" },
            } as ProgressEvent<FileReader>);
          }
        }, 0);
      }),
      result: "system TestSystem",
      onload: null as ((e: ProgressEvent<FileReader>) => void) | null,
    };

    vi.spyOn(window, "FileReader").mockImplementation(
      () => mockFileReader as unknown as FileReader
    );

    result.current.handleImport();

    await waitFor(
      () => {
        expect(mockConvertDslToJson).toHaveBeenCalled();
      },
      { timeout: 1000 }
    );
  });

  it("should handle share correctly", async () => {
    const mockWriteText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText: mockWriteText } });

    const { result } = renderHook(() => useFileHandlers(mockCanvasRef));

    await result.current.handleShare();

    await waitFor(
      () => {
        expect(mockCreateNewProject).toHaveBeenCalled();
        expect(mockSaveProject).toHaveBeenCalled();
        expect(mockWriteText).toHaveBeenCalled();
        expect(mockShowToast).toHaveBeenCalled();
      },
      { timeout: 1000 }
    );
  });
});

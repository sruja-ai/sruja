// apps/designer/src/hooks/__tests__/useAppShortcuts.test.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useAppShortcuts } from "../useAppShortcuts";
import type { ViewTab } from "../../types";
import type { ArchitectureCanvasRef } from "../../components/Canvas";
import type { SrujaModelDump } from "@sruja/shared";

describe("useAppShortcuts", () => {
  const mockCanvasRef = {
    current: {
      fitView: vi.fn(),
      zoomToSelection: vi.fn(),
      zoomToActualSize: vi.fn(),
    } as unknown as ArchitectureCanvasRef,
  };

  const mockHandlers = {
    handleExport: vi.fn(),
    handleExportPNG: vi.fn().mockResolvedValue(undefined),
    handleImport: vi.fn(),
    handleCopy: vi.fn(),
    handlePaste: vi.fn(),
    handleDuplicate: vi.fn(),
    undo: vi.fn().mockReturnValue(null),
    redo: vi.fn().mockReturnValue(null),
    updateArchitecture: vi.fn().mockResolvedValue(undefined),
  };

  const mockUI = {
    setShowCommandPalette: vi.fn(),
    setShowShortcuts: vi.fn(),
    setShowActions: vi.fn(),
    setShowSettings: vi.fn(),
  };

  const mockModel: SrujaModelDump = {
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
  };

  const defaultProps = {
    activeTab: "overview" as ViewTab,
    model: mockModel,
    canvasRef: mockCanvasRef,
    handlers: mockHandlers,
    ui: mockUI,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return array of shortcut definitions", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    expect(result.current).toBeInstanceOf(Array);
    expect(result.current.length).toBeGreaterThan(0);
  });

  it("should include all required shortcuts", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const shortcutKeys = result.current.map((s) => s.key);
    expect(shortcutKeys).toContain("s");
    expect(shortcutKeys).toContain("o");
    expect(shortcutKeys).toContain("z");
    expect(shortcutKeys).toContain("y");
    expect(shortcutKeys).toContain("k");
    expect(shortcutKeys).toContain("c");
    expect(shortcutKeys).toContain("v");
    expect(shortcutKeys).toContain("d");
    expect(shortcutKeys).toContain("0");
    expect(shortcutKeys).toContain("=");
    expect(shortcutKeys).toContain("1");
    expect(shortcutKeys).toContain("?");
    expect(shortcutKeys).toContain("Escape");
  });

  it("should execute save/export shortcut correctly for non-diagram tabs", () => {
    const { result } = renderHook(() =>
      useAppShortcuts({ ...defaultProps, activeTab: "overview" })
    );

    const saveShortcut = result.current.find((s) => s.key === "s" && s.ctrlKey);
    expect(saveShortcut).toBeDefined();

    saveShortcut?.action();
    expect(mockHandlers.handleExport).toHaveBeenCalledTimes(1);
    expect(mockHandlers.handleExportPNG).not.toHaveBeenCalled();
  });

  it("should execute PNG export shortcut for diagram tab", () => {
    const { result } = renderHook(() => useAppShortcuts({ ...defaultProps, activeTab: "diagram" }));

    const saveShortcut = result.current.find((s) => s.key === "s" && s.ctrlKey);
    expect(saveShortcut).toBeDefined();

    saveShortcut?.action();
    expect(mockHandlers.handleExportPNG).toHaveBeenCalledTimes(1);
  });

  it("should not execute PNG export if model is null", () => {
    const { result } = renderHook(() =>
      useAppShortcuts({ ...defaultProps, activeTab: "diagram", model: null })
    );

    const saveShortcut = result.current.find((s) => s.key === "s" && s.ctrlKey);
    expect(saveShortcut).toBeDefined();

    saveShortcut?.action();
    expect(mockHandlers.handleExportPNG).not.toHaveBeenCalled();
  });

  it("should execute import shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const importShortcut = result.current.find((s) => s.key === "o" && s.ctrlKey);
    expect(importShortcut).toBeDefined();

    importShortcut?.action();
    expect(mockHandlers.handleImport).toHaveBeenCalledTimes(1);
  });

  it("should execute undo shortcut correctly", () => {
    const previousState = { ...mockModel };
    mockHandlers.undo.mockReturnValue(previousState);

    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const undoShortcut = result.current.find((s) => s.key === "z" && s.ctrlKey && !s.shiftKey);
    expect(undoShortcut).toBeDefined();

    undoShortcut?.action();
    expect(mockHandlers.undo).toHaveBeenCalledTimes(1);
    expect(mockHandlers.updateArchitecture).toHaveBeenCalled();
  });

  it("should not execute undo if no previous state", () => {
    mockHandlers.undo.mockReturnValue(null);

    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const undoShortcut = result.current.find((s) => s.key === "z" && s.ctrlKey && !s.shiftKey);
    expect(undoShortcut).toBeDefined();

    undoShortcut?.action();
    expect(mockHandlers.undo).toHaveBeenCalledTimes(1);
    expect(mockHandlers.updateArchitecture).not.toHaveBeenCalled();
  });

  it("should execute redo shortcut (Ctrl+Shift+Z) correctly", () => {
    const nextState = { ...mockModel };
    mockHandlers.redo.mockReturnValue(nextState);

    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const redoShortcut = result.current.find((s) => s.key === "z" && s.ctrlKey && s.shiftKey);
    expect(redoShortcut).toBeDefined();

    redoShortcut?.action();
    expect(mockHandlers.redo).toHaveBeenCalledTimes(1);
    expect(mockHandlers.updateArchitecture).toHaveBeenCalled();
  });

  it("should execute redo shortcut (Ctrl+Y) correctly", () => {
    const nextState = { ...mockModel };
    mockHandlers.redo.mockReturnValue(nextState);

    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const redoShortcut = result.current.find((s) => s.key === "y" && s.ctrlKey);
    expect(redoShortcut).toBeDefined();

    redoShortcut?.action();
    expect(mockHandlers.redo).toHaveBeenCalledTimes(1);
    expect(mockHandlers.updateArchitecture).toHaveBeenCalled();
  });

  it("should execute command palette shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const paletteShortcut = result.current.find((s) => s.key === "k" && s.ctrlKey);
    expect(paletteShortcut).toBeDefined();

    paletteShortcut?.action();
    expect(mockUI.setShowCommandPalette).toHaveBeenCalledWith(true);
  });

  it("should execute copy shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const copyShortcut = result.current.find((s) => s.key === "c" && s.ctrlKey);
    expect(copyShortcut).toBeDefined();

    copyShortcut?.action();
    expect(mockHandlers.handleCopy).toHaveBeenCalledTimes(1);
  });

  it("should execute paste shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const pasteShortcut = result.current.find((s) => s.key === "v" && s.ctrlKey);
    expect(pasteShortcut).toBeDefined();

    pasteShortcut?.action();
    expect(mockHandlers.handlePaste).toHaveBeenCalledTimes(1);
  });

  it("should execute duplicate shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const duplicateShortcut = result.current.find((s) => s.key === "d" && s.ctrlKey);
    expect(duplicateShortcut).toBeDefined();

    duplicateShortcut?.action();
    expect(mockHandlers.handleDuplicate).toHaveBeenCalledTimes(1);
  });

  it("should execute fit view shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const fitViewShortcut = result.current.find((s) => s.key === "0" && s.ctrlKey);
    expect(fitViewShortcut).toBeDefined();

    fitViewShortcut?.action();
    expect(mockCanvasRef.current?.fitView).toHaveBeenCalledTimes(1);
  });

  it("should not execute fit view if canvas ref is null", () => {
    const nullCanvasRef = { current: null } as React.RefObject<ArchitectureCanvasRef>;

    const { result } = renderHook(() =>
      useAppShortcuts({ ...defaultProps, canvasRef: nullCanvasRef })
    );

    const fitViewShortcut = result.current.find((s) => s.key === "0" && s.ctrlKey);
    expect(fitViewShortcut).toBeDefined();

    fitViewShortcut?.action();
    // Should not throw, just not call
  });

  it("should execute zoom to selection shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const zoomShortcut = result.current.find((s) => s.key === "=" && s.ctrlKey);
    expect(zoomShortcut).toBeDefined();

    zoomShortcut?.action();
    expect(mockCanvasRef.current?.zoomToSelection).toHaveBeenCalledTimes(1);
  });

  it("should execute actual size shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const actualSizeShortcut = result.current.find((s) => s.key === "1" && s.ctrlKey);
    expect(actualSizeShortcut).toBeDefined();

    actualSizeShortcut?.action();
    expect(mockCanvasRef.current?.zoomToActualSize).toHaveBeenCalledTimes(1);
  });

  it("should execute show shortcuts shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const showShortcuts = result.current.find((s) => s.key === "?");
    expect(showShortcuts).toBeDefined();

    showShortcuts?.action();
    expect(mockUI.setShowShortcuts).toHaveBeenCalledWith(true);
  });

  it("should execute escape shortcut correctly", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    const escapeShortcut = result.current.find((s) => s.key === "Escape");
    expect(escapeShortcut).toBeDefined();

    escapeShortcut?.action();
    expect(mockUI.setShowActions).toHaveBeenCalledWith(false);
    expect(mockUI.setShowSettings).toHaveBeenCalledWith(false);
  });

  it("should include descriptions for all shortcuts", () => {
    const { result } = renderHook(() => useAppShortcuts(defaultProps));

    result.current.forEach((shortcut) => {
      expect(shortcut.description).toBeDefined();
      expect(typeof shortcut.description).toBe("string");
      expect(shortcut.description.length).toBeGreaterThan(0);
    });
  });

  it("should memoize shortcuts based on dependencies", () => {
    const { result, rerender } = renderHook((props) => useAppShortcuts(props), {
      initialProps: defaultProps,
    });

    const firstResult = result.current;

    // Rerender with same props
    rerender(defaultProps);
    expect(result.current).toBe(firstResult);

    // Rerender with different activeTab
    rerender({ ...defaultProps, activeTab: "diagram" });
    expect(result.current).not.toBe(firstResult);
  });
});

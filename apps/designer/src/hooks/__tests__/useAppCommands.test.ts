// apps/designer/src/hooks/__tests__/useAppCommands.test.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useAppCommands } from "../useAppCommands";
import type { ViewTab } from "../../types";

describe("useAppCommands", () => {
  const mockSetActiveTab = vi.fn();
  const mockHandleExport = vi.fn();
  const mockHandleImport = vi.fn();
  const mockHandleExportPNG = vi.fn().mockResolvedValue(undefined);
  const mockHandleExportSVG = vi.fn().mockResolvedValue(undefined);

  const defaultProps = {
    activeTab: "overview" as ViewTab,
    setActiveTab: mockSetActiveTab,
    handleExport: mockHandleExport,
    handleImport: mockHandleImport,
    handleExportPNG: mockHandleExportPNG,
    handleExportSVG: mockHandleExportSVG,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return base commands for all tabs", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    expect(result.current).toBeInstanceOf(Array);
    expect(result.current.length).toBeGreaterThan(0);

    // Check for navigation commands
    const navigationCmds = result.current.filter((cmd) => cmd.category === "navigation");
    expect(navigationCmds.length).toBeGreaterThan(0);

    // Check for export/import commands
    const exportCmds = result.current.filter((cmd) => cmd.category === "export");
    expect(exportCmds.length).toBeGreaterThan(0);
  });

  it("should include all navigation tabs", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    const tabIds = result.current.filter((cmd) => cmd.id.startsWith("tab-")).map((cmd) => cmd.id);

    expect(tabIds).toContain("tab-overview");
    expect(tabIds).toContain("tab-diagram");
    expect(tabIds).toContain("tab-details");
    expect(tabIds).toContain("tab-code");
    expect(tabIds).toContain("tab-builder");
  });

  it("should include export and import commands", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    const cmdIds = result.current.map((cmd) => cmd.id);
    expect(cmdIds).toContain("export-json");
    expect(cmdIds).toContain("import-json");
  });

  it("should add PNG and SVG export commands when activeTab is diagram", () => {
    const { result } = renderHook(() => useAppCommands({ ...defaultProps, activeTab: "diagram" }));

    const cmdIds = result.current.map((cmd) => cmd.id);
    expect(cmdIds).toContain("export-png");
    expect(cmdIds).toContain("export-svg");
  });

  it("should not include PNG/SVG export commands for non-diagram tabs", () => {
    const tabs: ViewTab[] = ["overview", "details", "code", "builder", "governance"];

    tabs.forEach((tab) => {
      const { result } = renderHook(() => useAppCommands({ ...defaultProps, activeTab: tab }));

      const cmdIds = result.current.map((cmd) => cmd.id);
      expect(cmdIds).not.toContain("export-png");
      expect(cmdIds).not.toContain("export-svg");
    });
  });

  it("should execute navigation commands correctly", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    const overviewCmd = result.current.find((cmd) => cmd.id === "tab-overview");
    expect(overviewCmd).toBeDefined();

    overviewCmd?.action();
    expect(mockSetActiveTab).toHaveBeenCalledWith("overview");
  });

  it("should execute export command correctly", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    const exportCmd = result.current.find((cmd) => cmd.id === "export-json");
    expect(exportCmd).toBeDefined();

    exportCmd?.action();
    expect(mockHandleExport).toHaveBeenCalledTimes(1);
  });

  it("should execute import command correctly", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    const importCmd = result.current.find((cmd) => cmd.id === "import-json");
    expect(importCmd).toBeDefined();

    importCmd?.action();
    expect(mockHandleImport).toHaveBeenCalledTimes(1);
  });

  it("should execute PNG export command correctly", () => {
    const { result } = renderHook(() => useAppCommands({ ...defaultProps, activeTab: "diagram" }));

    const pngCmd = result.current.find((cmd) => cmd.id === "export-png");
    expect(pngCmd).toBeDefined();

    pngCmd?.action();
    expect(mockHandleExportPNG).toHaveBeenCalledTimes(1);
  });

  it("should execute SVG export command correctly", () => {
    const { result } = renderHook(() => useAppCommands({ ...defaultProps, activeTab: "diagram" }));

    const svgCmd = result.current.find((cmd) => cmd.id === "export-svg");
    expect(svgCmd).toBeDefined();

    svgCmd?.action();
    expect(mockHandleExportSVG).toHaveBeenCalledTimes(1);
  });

  it("should include keywords for searchability", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    result.current.forEach((cmd) => {
      expect(cmd.keywords).toBeDefined();
      expect(Array.isArray(cmd.keywords)).toBe(true);
      expect(cmd.keywords.length).toBeGreaterThan(0);
    });
  });

  it("should include icons for all commands", () => {
    const { result } = renderHook(() => useAppCommands(defaultProps));

    result.current.forEach((cmd) => {
      expect(cmd.icon).toBeDefined();
    });
  });

  it("should memoize commands based on dependencies", () => {
    const { result, rerender } = renderHook((props) => useAppCommands(props), {
      initialProps: defaultProps,
    });

    const firstResult = result.current;

    // Rerender with same props
    rerender(defaultProps);
    expect(result.current).toBe(firstResult); // Should be same reference

    // Rerender with different activeTab
    rerender({ ...defaultProps, activeTab: "diagram" });
    expect(result.current).not.toBe(firstResult); // Should be new reference
  });

  it("should handle all ViewTab types", () => {
    const tabs: ViewTab[] = ["overview", "diagram", "details", "code", "builder", "governance"];

    tabs.forEach((tab) => {
      const { result } = renderHook(() => useAppCommands({ ...defaultProps, activeTab: tab }));

      expect(result.current).toBeInstanceOf(Array);
      expect(result.current.length).toBeGreaterThan(0);
    });
  });
});

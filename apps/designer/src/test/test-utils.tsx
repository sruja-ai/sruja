// apps/designer/src/test/test-utils.tsx
/**
 * Test utilities for consistent test setup and helpers
 * Following FAANG-level testing best practices
 */

import { render, type RenderOptions } from "@testing-library/react";
import { type ReactElement } from "react";
import { vi } from "vitest";

/**
 * Custom render function that wraps components with necessary providers
 * Use this instead of the default render from @testing-library/react
 */
export function renderWithProviders(ui: ReactElement, options?: Omit<RenderOptions, "wrapper">) {
  return render(ui, {
    ...options,
  });
}

/**
 * Create a mock function with proper typing
 */
export function createMockFn<T extends (...args: any[]) => any>(
  implementation?: T
): ReturnType<typeof vi.fn<T>> {
  return vi.fn(implementation) as ReturnType<typeof vi.fn<T>>;
}

/**
 * Wait for async operations to complete
 * Useful for testing async hooks and effects
 */
export async function waitForAsync(ms = 0) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Create a mock SrujaModelDump for testing
 */
export function createMockModel(overrides?: Partial<any>): any {
  return {
    specification: { tags: {}, elements: {} },
    elements: {},
    relations: [],
    views: {},
    sruja: {
      requirements: [],
      adrs: [],
      flows: [],
      scenarios: [],
    },
    _metadata: {
      name: "Test Architecture",
      version: "1.0.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0",
    },
    ...overrides,
  };
}

/**
 * Mock window.location for URL testing
 */
export function mockWindowLocation(overrides: Partial<Location>) {
  const originalLocation = window.location;
  const mockLocation = {
    ...originalLocation,
    ...overrides,
  } as Location;

  // Use Object.defineProperty to replace the location object
  Object.defineProperty(window, "location", {
    value: mockLocation,
    writable: true,
    configurable: true,
  });

  return () => {
    Object.defineProperty(window, "location", {
      value: originalLocation,
      writable: true,
      configurable: true,
    });
  };
}

/**
 * Mock window.history for history testing
 */
export function mockWindowHistory() {
  const originalHistory = window.history;
  const mockReplaceState = vi.fn();
  const mockPushState = vi.fn();

  window.history = {
    ...originalHistory,
    replaceState: mockReplaceState,
    pushState: mockPushState,
  } as unknown as History;

  return {
    mockReplaceState,
    mockPushState,
    restore: () => {
      window.history = originalHistory;
    },
  };
}

/**
 * Create a mock canvas ref for testing
 */
export function createMockCanvasRef() {
  return {
    current: {
      fitView: vi.fn(),
      zoomToSelection: vi.fn(),
      zoomToActualSize: vi.fn(),
      exportAsPNG: vi.fn().mockResolvedValue(undefined),
      exportAsSVG: vi.fn().mockResolvedValue(undefined),
    },
  };
}

/**
 * Reset all mocks between tests
 */
export function resetAllMocks() {
  vi.clearAllMocks();
}

/**
 * Create a delay for testing async behavior
 */
export function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Re-export everything from @testing-library/react for convenience
export * from "@testing-library/react";

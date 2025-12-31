// apps/designer/src/hooks/__tests__/useTabCounts.test.ts
import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useTabCounts } from "../useTabCounts";
import type { SrujaModelDump } from "@sruja/shared";

describe("useTabCounts", () => {
  it("should return zero counts for null data", () => {
    const { result } = renderHook(() => useTabCounts(null));

    expect(result.current).toEqual({
      requirements: 0,
      adrs: 0,
    });
  });

  it("should return zero counts for data without sruja property", () => {
    const data: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {},
      relations: [],
      views: {},
      sruja: undefined as any,
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const { result } = renderHook(() => useTabCounts(data));

    expect(result.current).toEqual({
      requirements: 0,
      adrs: 0,
    });
  });

  it("should return correct counts for requirements and adrs", () => {
    const data: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {},
      relations: [],
      views: {},
      sruja: {
        requirements: [
          { id: "req1", title: "Requirement 1" },
          { id: "req2", title: "Requirement 2" },
        ],
        adrs: [
          { id: "adr1", title: "ADR 1" },
          { id: "adr2", title: "ADR 2" },
          { id: "adr3", title: "ADR 3" },
        ],
        flows: [],
        scenarios: [],
      },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const { result } = renderHook(() => useTabCounts(data));

    expect(result.current).toEqual({
      requirements: 2,
      adrs: 3,
    });
  });

  it("should return zero for empty arrays", () => {
    const data: SrujaModelDump = {
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
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const { result } = renderHook(() => useTabCounts(data));

    expect(result.current).toEqual({
      requirements: 0,
      adrs: 0,
    });
  });

  it("should handle undefined requirements and adrs", () => {
    const data: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {},
      relations: [],
      views: {},
      sruja: {
        requirements: undefined as any,
        adrs: undefined as any,
        flows: [],
        scenarios: [],
      },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const { result } = renderHook(() => useTabCounts(data));

    expect(result.current).toEqual({
      requirements: 0,
      adrs: 0,
    });
  });

  it("should memoize results based on data reference", () => {
    const data: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {},
      relations: [],
      views: {},
      sruja: {
        requirements: [{ id: "req1", title: "Requirement 1" }],
        adrs: [{ id: "adr1", title: "ADR 1" }],
        flows: [],
        scenarios: [],
      },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const { result, rerender } = renderHook((props) => useTabCounts(props), {
      initialProps: data,
    });

    const firstResult = result.current;

    // Rerender with same data reference
    rerender(data);
    expect(result.current).toBe(firstResult); // Should be same reference

    // Rerender with new data reference but same content
    const newData = { ...data };
    rerender(newData);
    expect(result.current).not.toBe(firstResult); // Should be new reference
    expect(result.current.requirements).toBe(1); // But same values
    expect(result.current.adrs).toBe(1);
  });
});

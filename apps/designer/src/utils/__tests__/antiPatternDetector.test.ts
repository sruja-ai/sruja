// apps/designer/src/utils/__tests__/antiPatternDetector.test.ts
import { describe, it, expect } from "vitest";
import { detectAntiPatterns } from "../antiPatternDetector";
import type { SrujaModelDump } from "@sruja/shared";

describe("antiPatternDetector", () => {
  it("should return empty array for null model", () => {
    expect(detectAntiPatterns(null)).toEqual([]);
  });

  it("should return empty array for model with no relations", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
      },
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

    expect(detectAntiPatterns(model)).toEqual([]);
  });

  it("should detect cycles in relations", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
        System3: { id: "System3", kind: "system", title: "System 3", tags: [], links: [] },
      },
      relations: [
        { source: "System1", target: "System2" } as { source: string; target: string },
        { source: "System2", target: "System3" } as { source: string; target: string },
        { source: "System3", target: "System1" } as { source: string; target: string }, // Creates cycle
      ],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    expect(patterns.length).toBeGreaterThan(0);
    expect(patterns.some((p) => p.type === "cycle")).toBe(true);
  });

  it("should detect god objects with high fan-out", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        GodSystem: { id: "GodSystem", kind: "system", title: "God System", tags: [], links: [] },
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
        System3: { id: "System3", kind: "system", title: "System 3", tags: [], links: [] },
        System4: { id: "System4", kind: "system", title: "System 4", tags: [], links: [] },
        System5: { id: "System5", kind: "system", title: "System 5", tags: [], links: [] },
        System6: { id: "System6", kind: "system", title: "System 6", tags: [], links: [] },
      },
      relations: [
        { source: "GodSystem", target: "System1" } as { source: string; target: string },
        { source: "GodSystem", target: "System2" } as { source: string; target: string },
        { source: "GodSystem", target: "System3" } as { source: string; target: string },
        { source: "GodSystem", target: "System4" } as { source: string; target: string },
        { source: "GodSystem", target: "System5" } as { source: string; target: string },
        { source: "GodSystem", target: "System6" } as { source: string; target: string }, // 6 connections > threshold
      ],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    expect(patterns.length).toBeGreaterThan(0);
    expect(patterns.some((p) => p.type === "god-object" || p.type === "god-component")).toBe(true);
  });

  it("should detect god objects with high fan-in", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        GodSystem: { id: "GodSystem", kind: "system", title: "God System", tags: [], links: [] },
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
        System3: { id: "System3", kind: "system", title: "System 3", tags: [], links: [] },
        System4: { id: "System4", kind: "system", title: "System 4", tags: [], links: [] },
        System5: { id: "System5", kind: "system", title: "System 5", tags: [], links: [] },
        System6: { id: "System6", kind: "system", title: "System 6", tags: [], links: [] },
      },
      relations: [
        { source: "System1", target: "GodSystem" } as { source: string; target: string },
        { source: "System2", target: "GodSystem" } as { source: string; target: string },
        { source: "System3", target: "GodSystem" } as { source: string; target: string },
        { source: "System4", target: "GodSystem" } as { source: string; target: string },
        { source: "System5", target: "GodSystem" } as { source: string; target: string },
        { source: "System6", target: "GodSystem" } as { source: string; target: string }, // 6 connections > threshold
      ],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    expect(patterns.length).toBeGreaterThan(0);
    expect(patterns.some((p) => p.type === "god-object" || p.type === "god-component")).toBe(true);
  });

  it("should mark critical severity for very high connections", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        GodSystem: { id: "GodSystem", kind: "system", title: "God System", tags: [], links: [] },
        ...Array.from({ length: 11 }, (_, i) => ({
          [`System${i + 1}`]: {
            id: `System${i + 1}`,
            kind: "system",
            title: `System ${i + 1}`,
            tags: [],
            links: [],
          },
        })).reduce((acc, item) => ({ ...acc, ...item }), {}),
      },
      relations: Array.from({ length: 11 }, (_, i) => ({
        source: "GodSystem",
        target: `System${i + 1}`,
      })) as any,
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    const godObjectPattern = patterns.find(
      (p) => p.type === "god-object" || p.type === "god-component"
    );
    expect(godObjectPattern?.severity).toBe("critical");
  });

  it("should include affected elements in anti-pattern", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
        System3: { id: "System3", kind: "system", title: "System 3", tags: [], links: [] },
      },
      relations: [
        { source: "System1", target: "System2" } as any,
        { source: "System2", target: "System3" } as any,
        { source: "System3", target: "System1" } as any,
      ],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    expect(patterns.length).toBeGreaterThan(0);
    patterns.forEach((pattern) => {
      expect(pattern.affectedElements).toBeDefined();
      expect(Array.isArray(pattern.affectedElements)).toBe(true);
      expect(pattern.affectedElements.length).toBeGreaterThan(0);
    });
  });

  it("should include suggestions for anti-patterns", () => {
    const model: SrujaModelDump = {
      specification: { tags: {}, elements: {} },
      elements: {
        System1: { id: "System1", kind: "system", title: "System 1", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "System 2", tags: [], links: [] },
        System3: { id: "System3", kind: "system", title: "System 3", tags: [], links: [] },
      },
      relations: [
        { source: "System1", target: "System2" } as any,
        { source: "System2", target: "System3" } as any,
        { source: "System3", target: "System1" } as any,
      ],
      views: {},
      sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
      _metadata: {
        name: "Test",
        version: "1.0",
        generated: new Date().toISOString(),
        srujaVersion: "1.0",
      },
    };

    const patterns = detectAntiPatterns(model);
    expect(patterns.length).toBeGreaterThan(0);
    patterns.forEach((pattern) => {
      expect(pattern.description).toBeDefined();
      expect(typeof pattern.description).toBe("string");
    });
  });
});

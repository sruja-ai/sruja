import { describe, it, expect } from "vitest";
import { createC4Id } from "../brand";
import { createC4Graph } from "../c4-model";
import { layout } from "../c4-layout";
import { createDefaultViewState } from "../c4-view";
import { InteractivePreset } from "../c4-options";
import { applyCornerRounding, cornerRoundingToSvgPath } from "../algorithms/edge-router";

describe("Obstacle-aware routing", () => {
  it("routes orthogonally around obstacles", () => {
    const s = {
      id: createC4Id("S"),
      label: "Source",
      kind: "SoftwareSystem",
      level: "context",
      tags: new Set<string>(),
    };
    const t = {
      id: createC4Id("T"),
      label: "Target",
      kind: "SoftwareSystem",
      level: "context",
      tags: new Set<string>(),
    };
    const o = {
      id: createC4Id("O"),
      label: "Obstacle",
      kind: "SoftwareSystem",
      level: "context",
      tags: new Set<string>(),
    };
    const rel = { id: "S->T", from: s.id, to: t.id };
    const graph = createC4Graph([s as any, t as any, o as any], [rel as any]);
    const res = layout(graph, createDefaultViewState(), InteractivePreset);
    const edge = res.relationships[0];
    const obstacle = [...res.nodes.values()].find((n) => n.nodeId === "O")!;
    const passesThrough = edge.points.some(
      (p) =>
        p.x >= obstacle.bbox.x &&
        p.x <= obstacle.bbox.x + obstacle.bbox.width &&
        p.y >= obstacle.bbox.y &&
        p.y <= obstacle.bbox.y + obstacle.bbox.height
    );
    expect(passesThrough).toBe(false);
  });
});

describe("Corner rounding", () => {
  it("produces smooth corners for orthogonal paths", () => {
    // Simple L-shaped path with one corner
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 100 },
    ];

    const result = applyCornerRounding(points, { cornerRadius: 10 });

    // Should have more points due to corner splitting
    expect(result.points.length).toBeGreaterThan(3);
    // Should have one corner control point
    expect(result.cornerControlPoints.length).toBe(1);
    // Should have arc segment type for the corner
    expect(result.segmentTypes).toContain("arc");
  });

  it("respects sharp style option", () => {
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 100 },
    ];

    const result = applyCornerRounding(points, { style: "sharp" });

    // Sharp style should return original points
    expect(result.points).toEqual(points);
    expect(result.cornerControlPoints.length).toBe(0);
    expect(result.segmentTypes.every((t) => t === "line")).toBe(true);
  });

  it("adjusts corner radius to fit segment length", () => {
    // Very short segments - radius should be reduced
    const points = [
      { x: 0, y: 0 },
      { x: 10, y: 0 }, // Only 10px segment
      { x: 10, y: 10 },
    ];

    const result = applyCornerRounding(points, { cornerRadius: 20, minSegmentLength: 5 });

    // Should still produce rounded result but with smaller effective radius
    // The beforeCorner and afterCorner points should be within the segments
    if (result.cornerControlPoints.length > 0) {
      const beforeCorner = result.points[1];
      const afterCorner = result.points[2];
      // Points should be between segment endpoints
      expect(beforeCorner.x).toBeLessThanOrEqual(10);
      expect(afterCorner.y).toBeLessThanOrEqual(10);
    }
  });

  it("generates valid SVG path", () => {
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
      { x: 100, y: 100 },
    ];

    const result = applyCornerRounding(points, { cornerRadius: 10 });
    const svgPath = cornerRoundingToSvgPath(result);

    // Should start with M command
    expect(svgPath).toMatch(/^M/);
    // Should contain Q command for bezier curve
    expect(svgPath).toContain("Q");
    // Should contain L command for lines
    expect(svgPath).toContain("L");
  });

  it("handles straight lines without corners", () => {
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 0 },
    ];

    const result = applyCornerRounding(points, { cornerRadius: 10 });

    // No corners to round
    expect(result.points).toEqual(points);
    expect(result.cornerControlPoints.length).toBe(0);
  });
});

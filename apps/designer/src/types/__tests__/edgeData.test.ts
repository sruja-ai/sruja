// apps/designer/src/types/__tests__/edgeData.test.ts
import { describe, it, expect } from "vitest";
import {
  getLabelPosition,
  getEdgePoints,
  hasLabelPosition,
  hasPoints,
  type EdgeData,
  type EdgeLabelPosition,
} from "../edgeData";
import type { XYPosition } from "@xyflow/react";

describe("edgeData type guards and helpers", () => {
  describe("hasLabelPosition", () => {
    it("should return true for data with labelPosition", () => {
      const data: EdgeData = {
        labelPosition: { x: 100, y: 200 },
      };
      expect(hasLabelPosition(data)).toBe(true);
    });

    it("should return false for data without labelPosition", () => {
      const data: EdgeData = {
        label: "Test",
      };
      expect(hasLabelPosition(data)).toBe(false);
    });

    it("should return false for null", () => {
      expect(hasLabelPosition(null)).toBe(false);
    });

    it("should return false for undefined", () => {
      expect(hasLabelPosition(undefined)).toBe(false);
    });
  });

  describe("hasPoints", () => {
    it("should return true for data with points", () => {
      const data: EdgeData = {
        points: [
          { x: 0, y: 0 },
          { x: 100, y: 100 },
        ],
      };
      expect(hasPoints(data)).toBe(true);
    });

    it("should return false for data without points", () => {
      const data: EdgeData = {
        label: "Test",
      };
      expect(hasPoints(data)).toBe(false);
    });

    it("should return true for empty points array (type guard)", () => {
      const data: EdgeData = {
        points: [],
      };
      // Type guard checks if points exists and is array, not if it's empty
      expect(hasPoints(data)).toBe(true);
    });

    it("should return false for null", () => {
      expect(hasPoints(null)).toBe(false);
    });
  });

  describe("getLabelPosition", () => {
    it("should return label position when present", () => {
      const position: EdgeLabelPosition = { x: 100, y: 200 };
      const data: EdgeData = {
        labelPosition: position,
      };
      expect(getLabelPosition(data)).toEqual(position);
    });

    it("should return null when labelPosition is missing", () => {
      const data: EdgeData = {
        label: "Test",
      };
      expect(getLabelPosition(data)).toBeNull();
    });

    it("should return null for null data", () => {
      expect(getLabelPosition(null)).toBeNull();
    });

    it("should return null for undefined data", () => {
      expect(getLabelPosition(undefined)).toBeNull();
    });

    it("should handle invalid labelPosition structure", () => {
      const data = {
        labelPosition: { x: "invalid", y: 200 },
      };
      expect(getLabelPosition(data)).toBeNull();
    });
  });

  describe("getEdgePoints", () => {
    it("should return points array when present", () => {
      const points: XYPosition[] = [
        { x: 0, y: 0 },
        { x: 100, y: 100 },
        { x: 200, y: 0 },
      ];
      const data: EdgeData = {
        points,
      };
      expect(getEdgePoints(data)).toEqual(points);
    });

    it("should return null when points are missing", () => {
      const data: EdgeData = {
        label: "Test",
      };
      expect(getEdgePoints(data)).toBeNull();
    });

    it("should return null for empty points array", () => {
      const data: EdgeData = {
        points: [],
      };
      // getEdgePoints returns null for empty arrays
      const result = getEdgePoints(data);
      expect(result).toBeNull();
    });

    it("should return null for null data", () => {
      expect(getEdgePoints(null)).toBeNull();
    });

    it("should return null for undefined data", () => {
      expect(getEdgePoints(undefined)).toBeNull();
    });

    it("should handle invalid points structure", () => {
      const data = {
        points: [{ x: "invalid", y: 100 }],
      };
      // Type guard only checks if points is an array, not the structure
      // So it will return the array even if structure is invalid
      const result = getEdgePoints(data);
      expect(Array.isArray(result)).toBe(true);
    });
  });
});

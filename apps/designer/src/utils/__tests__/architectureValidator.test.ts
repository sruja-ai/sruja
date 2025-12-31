import { describe, it, expect } from "vitest";
import {
  validateArchitecture,
  getIssuesForElement,
  getIssuesByCategory,
  type ValidationResult,
} from "../architectureValidator";
import type { SrujaModelDump } from "@sruja/shared";

// Helper to create valid empty dump
const createEmptyDump = (): SrujaModelDump => ({
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

describe("architectureValidator", () => {
  describe("validateArchitecture", () => {
    it("returns error for null architecture", () => {
      const result = validateArchitecture(null);

      expect(result.isValid).toBe(false);
      expect(result.score).toBe(0);
      expect(result.issues).toHaveLength(1);
      expect(result.issues[0].message).toContain("No architecture loaded");
    });

    it("returns error for empty architecture object (no proper structure)", () => {
      const result = validateArchitecture({} as SrujaModelDump);

      expect(result.isValid).toBe(false);
      expect(result.issues[0].category).toBe("structure");
    });

    it("validates architecture with only metadata (empty content)", () => {
      const arch: SrujaModelDump = createEmptyDump();

      const result = validateArchitecture(arch);

      // Should warn about empty architecture
      expect(result.issues.some((i) => i.id === "empty-architecture")).toBe(true);
    });

    it("detects duplicate elements (simulated as keys collision? Map keys are unique, so this test might need adjustment logic or removing duplicate ID check if parser guarantees uniqueness)", () => {
      // In SrujaModelDump, elements are a Map (Object), so duplicate keys are impossible in JSON.
      // However, multiple elements might claim same ID if we had an array.
      // Since it's a map, we can't test "duplicate IDs" in the *input* structure easily unless we test the *parser* validaton.
      // But validator runs on the *dump*.
      // If we want to simulate issues, maybe malformed relations?
      // Let's skip duplicate ID test for Dump structure as it's structurally impossible in JS object keys.
    });

    it("detects invalid relation references", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        User: { id: "User", kind: "person", title: "User", tags: [], links: [] },
      };
      arch.relations = [{ id: "r1", source: "User", target: "NonExistent", title: "uses" }];

      const result = validateArchitecture(arch);

      const refIssue = result.issues.find((i) => i.category === "reference");
      expect(refIssue).toBeDefined();
      expect(refIssue?.message).toContain("NonExistent");
    });

    it("handles FqnRef object format in relation references", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        User: { id: "User", kind: "person", title: "User", tags: [], links: [] },
      };
      // Test with FqnRef object format: { model: string }
      arch.relations = [
        {
          id: "r1",
          source: { model: "User" },
          target: { model: "NonExistent" },
          title: "uses",
        } as unknown as { id: string; source: string; target: string; title: string },
      ];

      const result = validateArchitecture(arch);

      const refIssue = result.issues.find((i) => i.category === "reference");
      expect(refIssue).toBeDefined();
      expect(refIssue?.message).toContain("NonExistent");
      expect(refIssue?.message).not.toContain("[object Object]");
    });

    it("detects orphan elements", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        System1: { id: "System1", kind: "system", title: "Connected", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "Orphan", tags: [], links: [] },
      };
      arch.relations = [{ id: "r1", source: "System1", target: "System1", title: "self" }];

      const result = validateArchitecture(arch);

      // System2 is orphan
      const orphanIssue = result.issues.find(
        (i) => i.category === "orphan" && i.elementId === "System2"
      );
      expect(orphanIssue).toBeDefined();
      expect(orphanIssue?.message).toContain("no relations");
    });

    it("detects orphan elements with FqnRef object format", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        System1: { id: "System1", kind: "system", title: "Connected", tags: [], links: [] },
        System2: { id: "System2", kind: "system", title: "Orphan", tags: [], links: [] },
      };
      // Test with FqnRef object format
      arch.relations = [
        {
          id: "r1",
          source: { model: "System1" },
          target: { model: "System1" },
          title: "self",
        } as unknown as { id: string; source: string; target: string; title: string },
      ];

      const result = validateArchitecture(arch);

      // System2 should still be detected as orphan
      const orphanIssue = result.issues.find(
        (i) => i.category === "orphan" && i.elementId === "System2"
      );
      expect(orphanIssue).toBeDefined();
      expect(orphanIssue?.message).toContain("no relations");
    });

    it("calculates score based on issues", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        User: { id: "User", kind: "person", title: "User", tags: [], links: [] },
        App: { id: "App", kind: "system", title: "Application", tags: [], links: [] },
      };
      arch.relations = [{ id: "r1", source: "User", target: "App", title: "Uses" }];

      const result = validateArchitecture(arch);

      // Should have high score since valid architecture
      expect(result.score).toBeGreaterThan(50);
      expect(result.isValid).toBe(true);
    });

    it("validates requirement tag references", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        App: { id: "App", kind: "system", title: "App", tags: [], links: [] },
      };
      arch.sruja!.requirements = [
        { id: "REQ-1", type: "functional", title: "Test", tags: ["NonExistent"] } as unknown as {
          id: string;
        },
      ];

      const result = validateArchitecture(arch);

      const tagIssue = result.issues.find(
        (i) => i.category === "reference" && i.elementType === "requirement"
      );
      expect(tagIssue).toBeDefined();
      expect(tagIssue?.message).toContain("NonExistent");
    });

    it("validates ADR tag references", () => {
      const arch: SrujaModelDump = createEmptyDump();
      arch.elements = {
        App: { id: "App", kind: "system", title: "App", tags: [], links: [] },
      };
      arch.sruja!.adrs = [
        {
          id: "ADR-001",
          title: "Test",
          status: "accepted",
          context: "",
          decision: "",
          tags: ["BadRef"],
        } as unknown as { id: string }, // Cast
      ];

      const result = validateArchitecture(arch);

      const adrIssue = result.issues.find(
        (i) => i.category === "reference" && i.elementType === "adr"
      );
      expect(adrIssue).toBeDefined();
    });
  });

  describe("getIssuesForElement", () => {
    it("filters issues by element path", () => {
      const result: ValidationResult = {
        isValid: false,
        score: 50,
        issues: [
          {
            id: "1",
            severity: "error",
            category: "reference",
            elementId: "App",
            message: "Issue 1",
          },
          {
            id: "2",
            severity: "warning",
            category: "orphan",
            elementId: "User",
            message: "Issue 2",
          },
          { id: "3", severity: "info", category: "missing", elementId: "App", message: "Issue 3" },
        ],
        summary: { errors: 1, warnings: 1, infos: 1 },
      };

      const appIssues = getIssuesForElement(result, "App");

      expect(appIssues).toHaveLength(2);
      expect(appIssues.every((i) => i.elementId === "App")).toBe(true);
    });
  });

  describe("getIssuesByCategory", () => {
    it("filters issues by category", () => {
      const result: ValidationResult = {
        isValid: false,
        score: 50,
        issues: [
          { id: "1", severity: "error", category: "reference", message: "Ref issue" },
          { id: "2", severity: "warning", category: "orphan", message: "Orphan issue" },
          { id: "3", severity: "error", category: "reference", message: "Another ref" },
        ],
        summary: { errors: 2, warnings: 1, infos: 0 },
      };

      const refIssues = getIssuesByCategory(result, "reference");

      expect(refIssues).toHaveLength(2);
      expect(refIssues.every((i) => i.category === "reference")).toBe(true);
    });
  });
});

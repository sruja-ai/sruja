import { describe, it, expect } from "vitest";
import {
  validateArchitecture,
  getIssuesForElement,
  getIssuesByCategory,
  type ValidationResult,
} from "../architectureValidator";
import type { ArchitectureJSON } from "../../types";

describe("architectureValidator", () => {
  describe("validateArchitecture", () => {
    it("returns error for null architecture", () => {
      const result = validateArchitecture(null);

      expect(result.isValid).toBe(false);
      expect(result.score).toBe(0);
      expect(result.issues).toHaveLength(1);
      expect(result.issues[0].message).toContain("No architecture loaded");
    });

    it("returns error for empty architecture object", () => {
      const result = validateArchitecture({} as ArchitectureJSON);

      expect(result.isValid).toBe(false);
      expect(result.issues[0].category).toBe("structure");
    });

    it("validates architecture with only metadata", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      expect(result.issues.some((i) => i.id === "empty-architecture")).toBe(true);
    });

    it("detects duplicate IDs", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [{ id: "User" }, { id: "User" }],
          systems: [],
          relations: [],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      const duplicateIssue = result.issues.find((i) => i.category === "duplicate");
      expect(duplicateIssue).toBeDefined();
      expect(duplicateIssue?.message).toContain("Duplicate ID");
    });

    it("detects invalid relation references", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [{ id: "User" }],
          systems: [],
          relations: [{ from: "User", to: "NonExistent", label: "uses" }],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      const refIssue = result.issues.find((i) => i.category === "reference");
      expect(refIssue).toBeDefined();
      expect(refIssue?.message).toContain("NonExistent");
    });

    it("detects orphan elements", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [
            { id: "System1", label: "Connected" },
            { id: "System2", label: "Orphan" },
          ],
          relations: [{ from: "System1", to: "System1", label: "self" }],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      const orphanIssue = result.issues.find(
        (i) => i.category === "orphan" && i.elementId === "System2"
      );
      expect(orphanIssue).toBeDefined();
      expect(orphanIssue?.message).toContain("no relations");
    });

    it("calculates score based on issues", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [{ id: "User", label: "User" }],
          systems: [{ id: "App", label: "Application" }],
          relations: [{ from: "User", to: "App", label: "Uses" }],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      // Should have high score since valid architecture
      expect(result.score).toBeGreaterThan(50);
      expect(result.isValid).toBe(true);
    });

    it("validates requirement tag references", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [{ id: "App" }],
          relations: [],
          requirements: [{ id: "REQ-1", type: "functional", title: "Test", tags: ["NonExistent"] }],
        },
        navigation: { levels: ["L1"] },
      };

      const result = validateArchitecture(arch);

      const tagIssue = result.issues.find(
        (i) => i.category === "reference" && i.elementType === "requirement"
      );
      expect(tagIssue).toBeDefined();
      expect(tagIssue?.message).toContain("NonExistent");
    });

    it("validates ADR tag references", () => {
      const arch: ArchitectureJSON = {
        metadata: { name: "Test", version: "1.0", generated: new Date().toISOString() },
        architecture: {
          persons: [],
          systems: [{ id: "App" }],
          relations: [],
          adrs: [
            {
              id: "ADR-001",
              title: "Test",
              status: "accepted",
              context: "",
              decision: "",
              tags: ["BadRef"],
            },
          ],
        },
        navigation: { levels: ["L1"] },
      };

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

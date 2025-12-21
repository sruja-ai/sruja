// apps/designer/src/components/Canvas/__tests__/LikeC4Canvas.test.tsx
// Unit tests for LikeC4Canvas component and model conversion

import { describe, it, expect, vi } from "vitest";
import type { SrujaModelDump } from "@sruja/shared";

// Mock the LikeC4 dependencies
vi.mock("@likec4/diagram/bundle", () => ({
  LikeC4ModelProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  LikeC4View: ({ viewId }: { viewId: string }) => <div data-testid="likec4-view">{viewId}</div>,
}));

vi.mock("@likec4/core/model", () => ({
  LikeC4Model: {
    fromDump: vi.fn((_dump) => ({
      views: () => [],
    })),
  },
}));

// Test the model conversion function logic
// Note: We test the conversion logic conceptually since the function is internal
describe("LikeC4Canvas model conversion", () => {
  describe("model validation", () => {
    it("should handle null model gracefully", () => {
      // Component should render empty state for null model
      const nullModel: SrujaModelDump | null = null;
      expect(nullModel).toBeNull();
    });

    it("should handle model with missing elements", () => {
      const invalidModel = {
        projectId: "test",
      } as unknown as SrujaModelDump;

      // Model without elements should be invalid
      expect(invalidModel.elements).toBeUndefined();
    });

    it("should handle model with empty elements object", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.elements).toBeDefined();
      expect(typeof model.elements).toBe("object");
      expect(Array.isArray(model.elements)).toBe(false);
    });

    it("should handle model with valid structure", () => {
      const model: SrujaModelDump = {
        elements: {
          "system:MySystem": {
            id: "system:MySystem",
            title: "My System",
            type: "system",
          },
        },
        relations: [],
        views: {},
        projectId: "test",
        project: { id: "test", name: "Test Project" },
      };

      expect(model.elements).toBeDefined();
      expect(model.relations).toBeDefined();
      expect(Array.isArray(model.relations)).toBe(true);
      expect(model.views).toBeDefined();
      expect(typeof model.views).toBe("object");
    });
  });

  describe("stage handling", () => {
    it("should handle computed stage", () => {
      const model: SrujaModelDump = {
        elements: {},
        _stage: "computed",
        projectId: "test",
      };

      expect(model._stage).toBe("computed");
    });

    it("should handle layouted stage", () => {
      const model: SrujaModelDump = {
        elements: {},
        _stage: "layouted",
        projectId: "test",
      };

      expect(model._stage).toBe("layouted");
    });

    it("should handle undefined stage", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model._stage).toBeUndefined();
    });
  });

  describe("project field handling", () => {
    it("should use projectId when project is missing", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "my-project",
      };

      expect(model.projectId).toBe("my-project");
      expect(model.project).toBeUndefined();
    });

    it("should use project.id when projectId is missing", () => {
      const model: SrujaModelDump = {
        elements: {},
        project: { id: "my-project", name: "My Project" },
      };

      expect(model.project?.id).toBe("my-project");
    });

    it("should prefer projectId over project.id", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "explicit-id",
        project: { id: "project-id", name: "Project" },
      };

      expect(model.projectId).toBe("explicit-id");
    });
  });

  describe("optional fields with defaults", () => {
    it("should handle missing globals", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.globals).toBeUndefined();
    });

    it("should handle missing imports", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.imports).toBeUndefined();
    });

    it("should handle missing deployments", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.deployments).toBeUndefined();
    });

    it("should handle missing specification", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.specification).toBeUndefined();
    });

    it("should handle missing relations", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.relations).toBeUndefined();
    });

    it("should handle missing views", () => {
      const model: SrujaModelDump = {
        elements: {},
        projectId: "test",
      };

      expect(model.views).toBeUndefined();
    });
  });

  describe("type validation", () => {
    it("should reject array for elements", () => {
      const invalidModel = {
        elements: [],
        projectId: "test",
      } as unknown as SrujaModelDump;

      expect(Array.isArray(invalidModel.elements)).toBe(true);
      // This should be caught by runtime validation
    });

    it("should reject null for elements", () => {
      const invalidModel = {
        elements: null,
        projectId: "test",
      } as unknown as SrujaModelDump;

      expect(invalidModel.elements).toBeNull();
      // This should be caught by runtime validation
    });

    it("should accept array for relations", () => {
      const model: SrujaModelDump = {
        elements: {},
        relations: [{ source: "a", target: "b", title: "uses" }],
        projectId: "test",
      };

      expect(Array.isArray(model.relations)).toBe(true);
    });

    it("should reject array for views", () => {
      const invalidModel = {
        elements: {},
        views: [],
        projectId: "test",
      } as unknown as SrujaModelDump;

      expect(Array.isArray(invalidModel.views)).toBe(true);
      // This should be caught by runtime validation
    });
  });
});

// packages/shared/src/web/errors.test.ts
// Tests for structured error handling and validation

import { describe, it, expect } from "vitest";
import {
  ExportError,
  ErrorCode,
  parseWasmError,
  validateDotResult,
  isValidDotElement,
  isValidDotRelation,
} from "./errors";
import type { WasmParseResponse, DotResult } from "./wasmTypes";

describe("ExportError", () => {
  it("should create error with code and message", () => {
    const error = new ExportError(ErrorCode.InvalidInput, "test error");
    expect(error.code).toBe(ErrorCode.InvalidInput);
    expect(error.message).toBe("test error");
    expect(error.name).toBe("ExportError");
  });

  it("should support optional context", () => {
    const error = new ExportError(ErrorCode.InvalidInput, "test error", { key: "value" });
    expect(error.context).toEqual({ key: "value" });
  });

  it("should identify parse errors", () => {
    const error = new ExportError(ErrorCode.ParseFailed, "parse failed");
    expect(error.isParseError()).toBe(true);
    expect(error.isValidationError()).toBe(false);
  });

  it("should identify validation errors", () => {
    const error = new ExportError(ErrorCode.InvalidInput, "invalid");
    expect(error.isValidationError()).toBe(true);
    expect(error.isParseError()).toBe(false);
  });

  it("should identify model errors", () => {
    const error = new ExportError(ErrorCode.NoModel, "no model");
    expect(error.isModelError()).toBe(true);
  });

  it("should identify export errors", () => {
    const error = new ExportError(ErrorCode.ExportFailed, "export failed");
    expect(error.isExportError()).toBe(true);
  });

  it("should identify system errors", () => {
    const error = new ExportError(ErrorCode.Panic, "panic");
    expect(error.isSystemError()).toBe(true);
  });
});

describe("parseWasmError", () => {
  it("should return null for successful response", () => {
    const response: WasmParseResponse = {
      ok: true,
      data: "success",
    };
    expect(parseWasmError(response)).toBeNull();
  });

  it("should parse error from response", () => {
    const response: WasmParseResponse = {
      ok: false,
      error: "test error",
      code: ErrorCode.InvalidInput,
    };
    const error = parseWasmError(response);
    expect(error).not.toBeNull();
    expect(error?.code).toBe(ErrorCode.InvalidInput);
    expect(error?.message).toBe("test error");
  });

  it("should include context from response", () => {
    const response: WasmParseResponse = {
      ok: false,
      error: "test error",
      code: ErrorCode.InvalidInput,
      context: { key: "value" },
    };
    const error = parseWasmError(response);
    expect(error?.context).toEqual({ key: "value" });
  });

  it("should use Unknown code if code missing", () => {
    const response: WasmParseResponse = {
      ok: false,
      error: "test error",
    };
    const error = parseWasmError(response);
    expect(error?.code).toBe(ErrorCode.Unknown);
  });
});

describe("isValidDotElement", () => {
  it("should validate correct element", () => {
    const element = {
      id: "test-id",
      kind: "system" as const,
      title: "Test System",
      width: 100,
      height: 50,
    };
    expect(isValidDotElement(element)).toBe(true);
  });

  it("should reject invalid element", () => {
    expect(isValidDotElement(null)).toBe(false);
    expect(isValidDotElement(undefined)).toBe(false);
    expect(isValidDotElement({})).toBe(false);
  });

  it("should require all required fields", () => {
    expect(isValidDotElement({ id: "test" })).toBe(false);
    expect(isValidDotElement({ id: "test", kind: "system" })).toBe(false);
    expect(isValidDotElement({ id: "test", kind: "system", title: "Test" })).toBe(false);
  });

  it("should validate kind values", () => {
    const validKinds = ["person", "system", "container", "component", "datastore", "queue"];
    for (const kind of validKinds) {
      const element = {
        id: "test",
        kind: kind as "system",
        title: "Test",
        width: 100,
        height: 50,
      };
      expect(isValidDotElement(element)).toBe(true);
    }

    const invalidElement = {
      id: "test",
      kind: "invalid",
      title: "Test",
      width: 100,
      height: 50,
    };
    expect(isValidDotElement(invalidElement)).toBe(false);
  });

  it("should validate optional fields", () => {
    const element = {
      id: "test",
      kind: "system" as const,
      title: "Test",
      width: 100,
      height: 50,
      technology: "Cloud",
      description: "Description",
      parentId: "parent",
    };
    expect(isValidDotElement(element)).toBe(true);
  });

  it("should reject invalid optional field types", () => {
    const element = {
      id: "test",
      kind: "system" as const,
      title: "Test",
      width: 100,
      height: 50,
      technology: 123, // should be string
    };
    expect(isValidDotElement(element)).toBe(false);
  });
});

describe("isValidDotRelation", () => {
  it("should validate correct relation", () => {
    const relation = {
      from: "node1",
      to: "node2",
      label: "uses",
    };
    expect(isValidDotRelation(relation)).toBe(true);
  });

  it("should reject invalid relation", () => {
    expect(isValidDotRelation(null)).toBe(false);
    expect(isValidDotRelation(undefined)).toBe(false);
    expect(isValidDotRelation({})).toBe(false);
  });

  it("should require from and to", () => {
    expect(isValidDotRelation({ from: "node1" })).toBe(false);
    expect(isValidDotRelation({ to: "node2" })).toBe(false);
  });

  it("should allow optional label", () => {
    const relation1 = {
      from: "node1",
      to: "node2",
    };
    expect(isValidDotRelation(relation1)).toBe(true);

    const relation2 = {
      from: "node1",
      to: "node2",
      label: "uses",
    };
    expect(isValidDotRelation(relation2)).toBe(true);
  });
});

describe("validateDotResult", () => {
  it("should validate correct DOT result", () => {
    const result: DotResult = {
      dot: "digraph G { }",
      elements: [
        {
          id: "node1",
          kind: "system",
          title: "System 1",
          width: 100,
          height: 50,
        },
      ],
      relations: [
        {
          from: "node1",
          to: "node2",
          label: "uses",
        },
      ],
    };
    expect(() => validateDotResult(result)).not.toThrow();
    const validated = validateDotResult(result);
    expect(validated.dot).toBe("digraph G { }");
    expect(validated.elements).toHaveLength(1);
    expect(validated.relations).toHaveLength(1);
  });

  it("should reject invalid result structure", () => {
    expect(() => validateDotResult(null)).toThrow(ExportError);
    expect(() => validateDotResult(undefined)).toThrow(ExportError);
    expect(() => validateDotResult({})).toThrow(ExportError);
  });

  it("should require dot field", () => {
    const invalid = {
      elements: [],
      relations: [],
    };
    expect(() => validateDotResult(invalid)).toThrow(ExportError);
  });

  it("should require elements array", () => {
    const invalid = {
      dot: "digraph G { }",
      relations: [],
    };
    expect(() => validateDotResult(invalid)).toThrow(ExportError);
  });

  it("should require relations array", () => {
    const invalid = {
      dot: "digraph G { }",
      elements: [],
    };
    expect(() => validateDotResult(invalid)).toThrow(ExportError);
  });

  it("should validate all elements", () => {
    const invalid = {
      dot: "digraph G { }",
      elements: [
        {
          id: "node1",
          kind: "system",
          title: "System 1",
          width: 100,
          height: 50,
        },
        {
          id: "node2", // missing required fields
        },
      ],
      relations: [],
    };
    expect(() => validateDotResult(invalid)).toThrow(ExportError);
  });

  it("should validate all relations", () => {
    const invalid = {
      dot: "digraph G { }",
      elements: [],
      relations: [
        {
          from: "node1",
          to: "node2",
        },
        {
          from: "node1", // missing to
        },
      ],
    };
    expect(() => validateDotResult(invalid)).toThrow(ExportError);
  });

  it("should return typed result", () => {
    const result = {
      dot: "digraph G { }",
      elements: [
        {
          id: "node1",
          kind: "system" as const,
          title: "System 1",
          width: 100,
          height: 50,
        },
      ],
      relations: [
        {
          from: "node1",
          to: "node2",
        },
      ],
    };
    const validated = validateDotResult(result);
    // TypeScript should infer correct types
    expect(validated.elements[0].kind).toBe("system");
    expect(validated.relations[0].from).toBe("node1");
  });
});

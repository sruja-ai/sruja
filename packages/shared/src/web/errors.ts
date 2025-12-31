// packages/shared/src/web/errors.ts
// Structured error handling for WASM exports

import type { WasmParseResponse } from "./wasmTypes";

/**
 * Error codes matching Go ErrorCode constants.
 *
 * @public
 */
export enum ErrorCode {
  // Parse errors (1000-1999)
  ParseFailed = "PARSE_1001",
  ParseSyntax = "PARSE_1002",
  ParseSemantic = "PARSE_1003",

  // Validation errors (2000-2999)
  InvalidArgs = "VALID_2001",
  InvalidInput = "VALID_2002",
  InvalidView = "VALID_2003",
  InvalidFilename = "VALID_2004",
  InputTooLarge = "VALID_2005",

  // Model errors (3000-3999)
  NoModel = "MODEL_3001",
  EmptyModel = "MODEL_3002",

  // Export errors (4000-4999)
  ExportFailed = "EXPORT_4001",
  ExportEmpty = "EXPORT_4002",
  ExportTimeout = "EXPORT_4003",

  // System errors (5000-5999)
  Panic = "SYSTEM_5001",
  Unknown = "SYSTEM_5002",
}

/**
 * Structured export error with code, message, and context.
 *
 * @public
 */
export class ExportError extends Error {
  public readonly code: string;
  public readonly context?: Record<string, unknown>;

  constructor(code: string, message: string, context?: Record<string, unknown>) {
    super(message);
    this.name = "ExportError";
    this.code = code;
    this.context = context;
  }

  /**
   * Check if this is a parse error.
   */
  isParseError(): boolean {
    return this.code.startsWith("PARSE_");
  }

  /**
   * Check if this is a validation error.
   */
  isValidationError(): boolean {
    return this.code.startsWith("VALID_");
  }

  /**
   * Check if this is a model error.
   */
  isModelError(): boolean {
    return this.code.startsWith("MODEL_");
  }

  /**
   * Check if this is an export error.
   */
  isExportError(): boolean {
    return this.code.startsWith("EXPORT_");
  }

  /**
   * Check if this is a system error.
   */
  isSystemError(): boolean {
    return this.code.startsWith("SYSTEM_");
  }
}

/**
 * Parse a WASM response into an ExportError if it contains an error.
 *
 * @public
 * @param response - WASM response to parse
 * @returns ExportError if response contains error, null otherwise
 */
export function parseWasmError(response: WasmParseResponse): ExportError | null {
  if (response.ok) {
    return null;
  }

  const code = response.code || ErrorCode.Unknown;
  const message = response.error || "Unknown error";
  const context = response.context;

  return new ExportError(code, message, context);
}

/**
 * Validate a DOT element structure.
 *
 * @internal
 */
export function isValidDotElement(elem: unknown): elem is import("./wasmTypes").DotElement {
  if (!elem || typeof elem !== "object") {
    return false;
  }

  const e = elem as Record<string, unknown>;

  // Required fields
  if (typeof e.id !== "string" || e.id === "") {
    return false;
  }

  if (typeof e.kind !== "string") {
    return false;
  }

  const validKinds = ["person", "system", "container", "component", "datastore", "queue"];
  if (!validKinds.includes(e.kind)) {
    return false;
  }

  if (typeof e.title !== "string") {
    return false;
  }

  if (typeof e.width !== "number" || e.width < 0) {
    return false;
  }

  if (typeof e.height !== "number" || e.height < 0) {
    return false;
  }

  // Optional fields
  if (e.technology !== undefined && typeof e.technology !== "string") {
    return false;
  }

  if (e.description !== undefined && typeof e.description !== "string") {
    return false;
  }

  if (e.parentId !== undefined && typeof e.parentId !== "string") {
    return false;
  }

  return true;
}

/**
 * Validate a DOT relation structure.
 *
 * @internal
 */
export function isValidDotRelation(rel: unknown): rel is import("./wasmTypes").DotRelation {
  if (!rel || typeof rel !== "object") {
    return false;
  }

  const r = rel as Record<string, unknown>;

  if (typeof r.from !== "string" || r.from === "") {
    return false;
  }

  if (typeof r.to !== "string" || r.to === "") {
    return false;
  }

  if (r.label !== undefined && typeof r.label !== "string") {
    return false;
  }

  return true;
}

/**
 * Validate a DOT result structure.
 *
 * @public
 * @param data - Data to validate
 * @returns Validated DotResult or throws ExportError
 */
export function validateDotResult(data: unknown): import("./wasmTypes").DotResult {
  if (!data || typeof data !== "object") {
    throw new ExportError(ErrorCode.ExportFailed, "Invalid DOT result: expected object", {
      type: typeof data,
    });
  }

  const result = data as Record<string, unknown>;

  // Validate dot field
  if (typeof result.dot !== "string") {
    throw new ExportError(
      ErrorCode.ExportFailed,
      'Invalid DOT result: missing or invalid "dot" field',
      { hasDot: "dot" in result, dotType: typeof result.dot }
    );
  }

  // Validate elements array
  if (!Array.isArray(result.elements)) {
    throw new ExportError(
      ErrorCode.ExportFailed,
      'Invalid DOT result: missing or invalid "elements" array',
      { hasElements: "elements" in result, elementsType: typeof result.elements }
    );
  }

  // Validate each element
  const elements: import("./wasmTypes").DotElement[] = [];
  for (let i = 0; i < result.elements.length; i++) {
    const elem = result.elements[i];
    if (!isValidDotElement(elem)) {
      throw new ExportError(
        ErrorCode.ExportFailed,
        `Invalid element structure in DOT result at index ${i}`,
        { index: i, element: elem }
      );
    }
    elements.push(elem);
  }

  // Validate relations array
  if (!Array.isArray(result.relations)) {
    throw new ExportError(
      ErrorCode.ExportFailed,
      'Invalid DOT result: missing or invalid "relations" array',
      { hasRelations: "relations" in result, relationsType: typeof result.relations }
    );
  }

  // Validate each relation
  const relations: import("./wasmTypes").DotRelation[] = [];
  for (let i = 0; i < result.relations.length; i++) {
    const rel = result.relations[i];
    if (!isValidDotRelation(rel)) {
      throw new ExportError(
        ErrorCode.ExportFailed,
        `Invalid relation structure in DOT result at index ${i}`,
        { index: i, relation: rel }
      );
    }
    relations.push(rel);
  }

  return {
    dot: result.dot,
    elements,
    relations,
  };
}

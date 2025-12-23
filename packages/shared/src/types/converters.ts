// packages/shared/src/types/converters.ts
// Conversion helpers between LikeC4 types and Sruja Dump types

import type {
  Element,
  Relationship,
  ParsedView,
  ParsedLikeC4ModelData,
  MarkdownOrString,
} from "@likec4/core/types";
import type { SrujaModelDump, ElementDump } from "./architecture";
import { extractText } from "../utils/richtext";
import { toString } from "../utils/branded";

/**
 * Type aliases for converter functions.
 * These represent the serialized format used in SrujaModelDump.
 * 
 * @public
 */
// ElementDump is imported from ./architecture

export type RelationDump = NonNullable<SrujaModelDump["relations"]>[number];
export type ViewDump = SrujaModelDump["views"] extends Record<string, infer V>
  ? V
  : ParsedView<any>;

/**
 * Convert LikeC4 Element to SrujaModelDump format (for Go backend JSON).
 * 
 * @public
 * @param element - The LikeC4 Element to convert
 * @returns Element in SrujaModelDump format
 * 
 * @remarks
 * This conversion handles:
 * - RichText descriptions (converts to string)
 * - Link format normalization
 * - Metadata type conversion
 * - Style defaults
 * 
 * @example
 * const element: Element = { id: 'system:api', kind: 'system', title: 'API' };
 * const dump = elementToDump(element);
 */
export function elementToDump(element: Element<any>): ElementDump {
  const description = extractText(element.description);

  return {
    id: element.id,
    kind: element.kind,
    title: element.title,
    description: description as MarkdownOrString | null,
    technology: element.technology,
    tags: element.tags,
    links: element.links?.map((l) =>
      typeof l === "string" ? { url: l } : l
    ),
    metadata: (element.metadata as Record<string, string> | undefined) ?? undefined,
    style: element.style ?? {},
  } as ElementDump;
}

/**
 * Convert SrujaModelDump Element to LikeC4 Element (from Go backend JSON).
 * 
 * @public
 * @param dump - The ElementDump to convert
 * @returns Partial Element compatible with LikeC4 types
 * 
 * @remarks
 * This is a simplified conversion - some LikeC4 features may be lost
 * during serialization. The returned element is a Partial type to handle
 * potential missing fields.
 * 
 * @example
 * const dump: ElementDump = { id: 'system:api', kind: 'system', title: 'API' };
 * const element = dumpToElement(dump);
 */
export function dumpToElement(dump: ElementDump): Partial<Element<any>> {
  const d = dump as Element<any>;
  return {
    id: d.id,
    kind: d.kind,
    title: d.title,
    description: d.description as MarkdownOrString | null | undefined,
    technology: d.technology,
    tags: d.tags,
    links: d.links,
    metadata: d.metadata,
    style: d.style ?? {},
  };
}

/**
 * Convert LikeC4 Relationship to SrujaModelDump format (for Go backend JSON).
 * 
 * @public
 * @param rel - The LikeC4 Relationship to convert
 * @returns Relationship in SrujaModelDump format
 * 
 * @remarks
 * Converts branded FQN types to plain strings for JSON serialization.
 * Uses source/target (LikeC4 standard) instead of from/to.
 * 
 * @example
 * const rel: Relationship = { id: 'rel-1', source: { model: 'system:a' }, target: { model: 'system:b' }, title: 'uses' };
 * const dump = relationshipToDump(rel);
 */
export function relationshipToDump(rel: Relationship<any>): RelationDump {
  const description = extractText(rel.description);

  return {
    id: rel.id,
    source: { model: toString(rel.source.model) },
    target: { model: toString(rel.target.model) },
    title: rel.title ?? undefined,
    description: description as MarkdownOrString | null,
    technology: rel.technology,
    kind: rel.kind,
    tags: rel.tags,
    color: typeof rel.color === "string" ? rel.color : undefined,
    line: rel.line,
    head: rel.head,
    tail: rel.tail,
  } as RelationDump;
}

/**
 * Convert LikeC4 ParsedView to SrujaModelDump format (for Go backend JSON).
 * 
 * @public
 * @param view - The LikeC4 ParsedView to convert
 * @returns View in SrujaModelDump format
 * 
 * @remarks
 * Converts branded ViewId types to plain strings for JSON serialization.
 * Handles RichText descriptions and optional viewOf field.
 * 
 * @example
 * const view: ParsedView = { id: 'view-1', title: 'Overview' };
 * const dump = viewToDump(view);
 */
export function viewToDump(view: ParsedView<any>): ViewDump {
  const description = extractText(view.description);

  return {
    ...view,
    id: toString(view.id),
    title: view.title ?? null,
    description: description as MarkdownOrString | null,
    viewOf: "viewOf" in view && view.viewOf ? toString(view.viewOf) : undefined,
    tags: view.tags,
  } as ViewDump;
}

/**
 * Type guard: Check if data is SrujaModelDump (from Go backend).
 * 
 * @public
 * @param data - The value to check
 * @returns true if data is a valid SrujaModelDump
 * 
 * @example
 * if (isSrujaModelDump(data)) { const elements = data.elements; }
 */
export function isSrujaModelDump(data: unknown): data is SrujaModelDump {
  return (
    typeof data === "object" &&
    data !== null &&
    "specification" in data &&
    "elements" in data &&
    "_metadata" in data
  );
}

/**
 * Type guard: Check if data is ParsedLikeC4ModelData (from Builder).
 * 
 * @public
 * @param data - The value to check
 * @returns true if data is a valid ParsedLikeC4ModelData
 * 
 * @example
 * if (isLikeC4ModelData(data)) { const stage = data._stage; }
 */
export function isLikeC4ModelData(
  data: unknown
): data is ParsedLikeC4ModelData {
  if (typeof data !== "object" || data === null) {
    return false;
  }
  const obj = data as Record<string, unknown>;
  return (
    "specification" in obj &&
    "elements" in obj &&
    "_stage" in obj &&
    obj._stage === "parsed"
  );
}

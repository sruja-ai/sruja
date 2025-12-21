// packages/shared/src/builder/likec4-adapter.ts
// Adapter to use LikeC4's Builder and convert to SrujaModelDump format

import { Builder as LikeC4Builder } from "@likec4/core/builder";
import type {
  ParsedLikeC4ModelData,
  Element,
  Relationship,
  ParsedView,
  Specification,
  MarkdownOrString,
} from "@likec4/core/types";
import type { Builder } from "@likec4/core/builder";
import type { SrujaModelDump } from "../types/architecture";
import { extractText } from "../utils/richtext";
import { toString } from "../utils/branded";

/**
 * Re-export LikeC4's BuilderSpecification for convenience.
 * 
 * @public
 */
export type { BuilderSpecification } from "@likec4/core/builder";

/**
 * Convert LikeC4's ParsedLikeC4ModelData to SrujaModelDump format.
 * 
 * @public
 * @param likec4Model - The LikeC4 parsed model data
 * @returns SrujaModelDump format ready for JSON serialization
 * 
 * @remarks
 * Even though SrujaModelDump uses LikeC4 types, conversion is needed because:
 * 1. Branded types (Fqn, RelationId) must be converted to plain strings for JSON
 * 2. RichText (description) must be converted from {txt: string} to string
 * 3. Relations: Record → Array format for Go backend compatibility
 * 4. Views: Handle branded ViewId types
 * 
 * @example
 * const likec4Model = builder.build();
 * const srujaDump = convertLikeC4ToSruja(likec4Model);
 */
export function convertLikeC4ToSruja(
  likec4Model: ParsedLikeC4ModelData
): SrujaModelDump {
  const elements: Record<string, Element<any>> = {};
  const relationsArray: Array<NonNullable<SrujaModelDump["relations"]>[number]> = [];
  const views: Record<string, NonNullable<NonNullable<SrujaModelDump["views"]>[string]>> = {};

  // Convert elements - handle RichText descriptions and branded types
  for (const [id, element] of Object.entries(likec4Model.elements)) {
    const el = element as Element<any>;
    const description = extractText(el.description);
    elements[toString(id)] = {
      ...el,
      id: toString(el.id),
      description: description as MarkdownOrString | null | undefined,
    } as Element<any>;
  }

  // Convert relations: Record → Array, with FqnRef format for source/target
  for (const [id, relation] of Object.entries(likec4Model.relations || {})) {
    const rel = relation as Relationship<any>;
    // Omit source/target and add new FqnRef format
    const { source, target, ...relWithoutSourceTarget } = rel;
    relationsArray.push({
      ...relWithoutSourceTarget,
      id: toString(id),
      source: { model: toString(rel.source.model) },
      target: { model: toString(rel.target.model) },
      title: rel.title ?? undefined,
      description: extractText(rel.description),
      color: typeof rel.color === "string" ? rel.color : undefined,
    } as NonNullable<SrujaModelDump["relations"]>[number]);
  }

  // Convert views - handle RichText and branded types
  for (const [id, view] of Object.entries(likec4Model.views || {})) {
    const v = view as ParsedView<any>;
    const description = extractText(v.description);
    views[toString(id)] = {
      ...v,
      id: toString(v.id),
      title: v.title ?? undefined,
      description: description as MarkdownOrString | null,
      viewOf: "viewOf" in v && v.viewOf ? toString(v.viewOf) : undefined,
    } as NonNullable<SrujaModelDump["views"]>[string];
  }

  // Specification can be used directly
  const specification = likec4Model.specification as Specification<any>;

  return {
    specification,
    elements,
    relations: relationsArray.length > 0 ? relationsArray : undefined,
    views: Object.keys(views).length > 0 ? views : undefined,
    deployments: {
      elements: {},
      relations: {},
    },
    _metadata: {
      name: likec4Model.projectId || "from-builder",
      version: "1.0.0",
      generated: new Date().toISOString(),
      srujaVersion: "1.0.0",
    },
  };
}

/**
 * Wrapper around LikeC4 Builder that returns SrujaModelDump
 * 
 * This provides a clean API that uses LikeC4's proven Builder implementation
 * and automatically converts the output to SrujaModelDump format.
 * 
 * The builder is immutable - each method returns a new instance.
 */
export class SrujaBuilder {
  private constructor(
    private likec4Builder: Builder<any> // LikeC4 Builder - using 'any' for generic type parameter only
  ) { }

  /**
   * Create a builder using LikeC4's compositional style (recommended).
   * 
   * @public
   * @param spec - Builder specification
   * @returns Builder instance with model, views, and deployment helpers
   * 
   * @remarks
   * Uses LikeC4's BuilderSpecification type directly - no conversion needed for input.
   * This is the recommended way to create a builder as it provides better type safety.
   * 
   * @example
   * const { builder, model } = SrujaBuilder.forSpecification({ elements: { system: {} } });
   * const result = builder.with(model(({ system }, _) => _(system('api', 'API')))).build();
   */
  static forSpecification(
    spec: Parameters<typeof LikeC4Builder.forSpecification>[0]
  ): {
    builder: SrujaBuilder;
    model: ReturnType<typeof LikeC4Builder.forSpecification>["model"];
    views: ReturnType<typeof LikeC4Builder.forSpecification>["views"];
    deployment: ReturnType<typeof LikeC4Builder.forSpecification>["deployment"];
  } {
    const likec4 = LikeC4Builder.forSpecification(spec);
    return {
      builder: new SrujaBuilder(likec4.builder),
      model: likec4.model,
      views: likec4.views,
      deployment: likec4.deployment,
    };
  }

  /**
   * Create a builder using LikeC4's chainable style.
   * 
   * @public
   * @param spec - Builder specification
   * @returns New SrujaBuilder instance
   * 
   * @example
   * const result = SrujaBuilder.specification({ elements: ['system'] })
   *   .model(({ system }, _) => _(system('api', 'API')))
   *   .build();
   */
  static specification(
    spec: Parameters<typeof LikeC4Builder.specification>[0]
  ): SrujaBuilder {
    const likec4 = LikeC4Builder.specification(spec);
    return new SrujaBuilder(likec4);
  }

  /**
   * Apply operations using LikeC4's `.with()` pattern.
   * 
   * @public
   * @param ops - Builder operation functions
   * @returns New SrujaBuilder instance
   * 
   * @remarks
   * The builder is immutable, so this returns a new instance.
   * LikeC4's runtime accepts rest parameter, so we use Function.apply
   * to properly handle the array of operations.
   */
  with(
    ...ops: ((builder: Builder<any>) => Builder<any>)[]
  ): SrujaBuilder {
    // Runtime implementation accepts rest parameter
    // Use apply to properly call with array of arguments
    const newBuilder = (this.likec4Builder.with as (
      ...args: ((builder: Builder<any>) => Builder<any>)[]
    ) => Builder<any>).apply(this.likec4Builder, ops);
    return new SrujaBuilder(newBuilder);
  }

  /**
   * Add model elements.
   * 
   * @public
   * @param callback - Model builder callback
   * @returns New SrujaBuilder instance
   */
  model(
    callback: Parameters<Builder<any>["model"]>[0]
  ): SrujaBuilder {
    const newBuilder = this.likec4Builder.model(callback);
    return new SrujaBuilder(newBuilder);
  }

  /**
   * Add deployment model.
   * 
   * @public
   * @param callback - Deployment builder callback
   * @returns New SrujaBuilder instance
   */
  deployment(
    callback: Parameters<Builder<any>["deployment"]>[0]
  ): SrujaBuilder {
    const newBuilder = this.likec4Builder.deployment(callback);
    return new SrujaBuilder(newBuilder);
  }

  /**
   * Add views.
   * 
   * @public
   * @param callback - Views builder callback
   * @returns New SrujaBuilder instance
   */
  views(
    callback: Parameters<Builder<any>["views"]>[0]
  ): SrujaBuilder {
    const newBuilder = this.likec4Builder.views(callback);
    return new SrujaBuilder(newBuilder);
  }

  /**
   * Build and convert to SrujaModelDump format.
   * 
   * @public
   * @returns SrujaModelDump ready for JSON serialization
   */
  build(): SrujaModelDump {
    const likec4Model = this.likec4Builder.build();
    return convertLikeC4ToSruja(likec4Model);
  }

  /**
   * Get the LikeC4 model directly (for advanced use cases).
   * 
   * @public
   * @returns ParsedLikeC4ModelData
   * 
   * @remarks
   * Useful if you need LikeC4-specific features or computed views
   * that aren't available in SrujaModelDump format.
   */
  buildLikeC4(): ParsedLikeC4ModelData {
    return this.likec4Builder.build();
  }

  /**
   * Get computed LikeC4 model (includes computed views).
   * 
   * @public
   * @returns ComputedLikeC4ModelData
   */
  toLikeC4Model(): ReturnType<Builder<any>["toLikeC4Model"]> {
    return this.likec4Builder.toLikeC4Model();
  }

  /**
   * Clone the builder (creates a new instance).
   * 
   * @public
   * @returns New SrujaBuilder instance with copied state
   */
  clone(): SrujaBuilder {
    return new SrujaBuilder(this.likec4Builder.clone());
  }
}

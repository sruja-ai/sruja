// packages/shared/src/builder/index.ts
// Sruja Builder API exports

/**
 * @packageDocumentation
 * 
 * # Builder Module
 * 
 * Programmatic API for building architecture models using LikeC4's proven Builder pattern.
 * 
 * ## Features
 * 
 * - Immutable builder pattern
 * - Automatic conversion to SrujaModelDump format
 * - Type-safe API with full TypeScript support
 * - Compositional and chainable styles
 * 
 * ## Usage
 * 
 * ```typescript
 * import { SrujaBuilder, type BuilderSpecification } from "@sruja/shared/builder";
 * 
 * const spec: BuilderSpecification = {
 *   elements: ['system', 'container', 'component'],
 * };
 * 
 * const { builder, model } = SrujaBuilder.forSpecification(spec);
 * 
 * const result = builder
 *   .model(({ system, component }, _) =>
 *     _(system('cloud', 'Cloud System'))
 *   )
 *   .build();
 * ```
 * 
 * @module builder
 */

// ============================================================================
// Builder API
// ============================================================================
/**
 * Main builder class and conversion utilities.
 */
export { SrujaBuilder, convertLikeC4ToSruja } from "./likec4-adapter";

// ============================================================================
// Type Exports
// ============================================================================
/**
 * Re-export LikeC4 types for direct use (no conversion needed for input).
 * 
 * @remarks
 * These types can be used directly in your code without conversion.
 */
export type { BuilderSpecification } from "@likec4/core/builder";
export type { Builder as LikeC4Builder } from "@likec4/core/builder";
export type { ParsedLikeC4ModelData, Specification } from "@likec4/core/types";

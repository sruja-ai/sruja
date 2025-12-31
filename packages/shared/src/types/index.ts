// packages/shared/src/types/index.ts
// Shared types for Sruja frontend

/**
 * @packageDocumentation
 *
 * # Types Module
 *
 * Core TypeScript type definitions for Sruja architecture models.
 *
 * ## Module Structure
 *
 * - **architecture.ts**: Core type definitions for models, governance, and legacy formats
 * - **converters.ts**: Type conversion utilities
 *
 * ## Usage
 *
 * ```typescript
 * import type { SrujaModelDump, Requirement, ADR } from '@sruja/shared/types';
 * import { elementToDump, isSrujaModelDump } from '@sruja/shared/types';
 * ```
 *
 * @module types
 */

/**
 * Sruja configuration interface.
 *
 * @public
 */
export interface SrujaConfig {
  /** Sruja version string */
  readonly version: string;
}

// ============================================================================
// Architecture Types
// ============================================================================
/**
 * Core architecture type definitions.
 *
 * @remarks
 * Core types are used for TypeScript code.
 * "Dump" types are only for Go backend JSON compatibility.
 */
export * from "./architecture";

// ============================================================================
// Type Converters
// ============================================================================
/**
 * Type conversion utilities.
 *
 * @remarks
 * These converters handle:
 * - RichText to string conversion
 * - Branded type to plain string conversion
 * - Record to Array format conversion (for relations)
 */

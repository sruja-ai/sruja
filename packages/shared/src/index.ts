// @sruja/shared
// Shared utilities and types for Sruja frontend applications

/**
 * @packageDocumentation
 *
 * # Sruja Shared Package
 *
 * This package provides shared utilities, types, and adapters for Sruja frontend applications.
 *
 * ## Module Organization
 *
 * - **types**: Core TypeScript type definitions for architecture models
 * - **utils**: Utility functions (logging, CSS variables, markdown processing)
 * - **builder**: Programmatic API for building architecture models
 * - **web**: Web WASM adapters (includes markdown and mermaid export)
 * - **analytics**: Analytics and error tracking utilities
 * - **storage**: Browser storage utilities
 * - **documentation**: Documentation loading utilities
 * - **examples**: Example code management
 *
 * ## Usage
 *
 * ```typescript
 * import { SrujaBuilder, type SrujaModelDump } from '@sruja/shared';
 * import { logger } from '@sruja/shared';
 * ```
 *
 * @module
 */

// ============================================================================
// Core Types
// ============================================================================
/**
 * Core type definitions for architecture models, converters, and validation.
 *
 * @module types
 */
export * from "./types";

// ============================================================================
// Utilities
// ============================================================================
/**
 * Utility functions for logging, markdown processing, and CSS variables.
 *
 * @module utils
 */
export * from "./utils";

// ============================================================================
// Exporters (via WASM)
// ============================================================================
/**
 * Export functionality is provided via WASM adapters in the 'web' module.
 *
 * @remarks
 * Use WASM-based exports for markdown and mermaid:
 * ```typescript
 * import {
 *   convertDslToMarkdown,
 *   convertDslToMermaid
 * } from '@sruja/shared/web/wasmAdapter';
 *
 * const markdown = await convertDslToMarkdown(dsl);
 * const mermaid = await convertDslToMermaid(dsl);
 * ```
 *
 * These use the Go backend implementation for consistency and performance.
 */

// ============================================================================
// Web WASM Adapters
// ============================================================================
/**
 * Web-specific WASM adapters for browser environments.
 *
 * @module web
 * @remarks
 * Node.js WASM adapter should be imported directly from './node/wasmAdapter'
 * to avoid bundling issues in browser environments.
 */
export * from "./web/wasmAdapter";
export * from "./web/wasmAdapterViewer";

// ============================================================================
// Analytics
// ============================================================================
/**
 * Analytics and error tracking utilities.
 *
 * @module analytics
 */
export * from "./analytics";

// ============================================================================
// Storage
// ============================================================================
/**
 * Browser storage utilities (IndexedDB wrapper).
 *
 * @module storage
 */
export * from "./storage/indexedStore";

// ============================================================================
// Documentation
// ============================================================================
/**
 * Documentation loading and management utilities.
 *
 * @module documentation
 */
export * from "./documentation";

// ============================================================================
// Examples
// ============================================================================
/**
 * Example code management and loading utilities.
 *
 * @module examples
 */
export * from "./examples";

// ============================================================================
// Platform-Specific Exports (Not exported by default)
// ============================================================================
// Node.js WASM adapter (for VS Code extension)
// Should be imported directly from './node/wasmAdapter' to avoid bundling issues
// export type { NodeWasmApi } from './node/wasmAdapter';
// export { initWasmNode } from './node/wasmAdapter';

// LSP exports are browser-only
// Should be imported directly from './lsp' in client components to prevent SSR imports
// export * from './lsp';

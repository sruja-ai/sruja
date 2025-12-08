// @sruja/shared
// Shared utilities and types for Sruja frontend applications

export * from './types';
export * from './utils';
export * from './web/wasmAdapter';
// LSP exports are browser-only - import directly from './lsp' in client components
// export * from './lsp'; // Commented out to prevent SSR imports
export * from './analytics';
export * from './storage/indexedStore';
export * from './documentation';
export * from './examples';

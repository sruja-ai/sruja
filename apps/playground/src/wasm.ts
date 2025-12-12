// apps/playground/src/wasm.ts
// Re-export WASM utilities from shared package
export {
  initWasmAuto as initWasm,
  getWasmApi,
  convertDslToJson,
  convertDslToMarkdown,
  convertDslToMermaid,
  type WasmApi,
} from '@sruja/shared';

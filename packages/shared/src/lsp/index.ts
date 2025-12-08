// Export WASM-based LSP shim
export { 
  createWasmLspApi, 
  initializeMonacoWasmLsp,
  type WasmLspApi 
} from './wasmLspShim'

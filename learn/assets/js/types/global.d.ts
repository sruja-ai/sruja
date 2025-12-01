// Global type definitions
import type { CompileResult } from './index';

declare global {
  interface Window {
    // Namespaced Sruja API for better security and organization
    sruja?: {
      wasmReady: boolean;
      wasmInitializing: boolean;
      compile?: (input: string, filename: string) => CompileResult;
    };
    
    // Legacy globals (deprecated, will be removed)
    /** @deprecated Use window.sruja.wasmReady instead */
    srujaWasmReady?: boolean;
    /** @deprecated Use window.sruja.wasmInitializing instead */
    srujaWasmInitializing?: boolean;
    /** @deprecated Use window.sruja.compile instead */
    compileSruja?: (input: string, filename: string) => CompileResult;
    
    Go?: new () => {
      importObject: WebAssembly.Imports;
      run: (instance: WebAssembly.Instance) => void;
    };
  }
}

export {};

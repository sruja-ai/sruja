// Global type definitions
import type { CompileResult } from './index';

declare global {
  interface Window {
    srujaWasmReady: boolean;
    srujaWasmInitializing: boolean;
    compileSruja?: (input: string, filename: string) => CompileResult;
    Go?: new () => {
      importObject: WebAssembly.Imports;
      run: (instance: WebAssembly.Instance) => void;
    };
  }
}

export {};

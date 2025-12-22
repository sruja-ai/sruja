"use strict";
// packages/shared/src/node/wasmLoader.ts
// WASM module loading utilities for Node.js
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  (function () {
    var ownKeys = function (o) {
      ownKeys =
        Object.getOwnPropertyNames ||
        function (o) {
          var ar = [];
          for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
          return ar;
        };
      return ownKeys(o);
    };
    return function (mod) {
      if (mod && mod.__esModule) return mod;
      var result = {};
      if (mod != null)
        for (var k = ownKeys(mod), i = 0; i < k.length; i++)
          if (k[i] !== "default") __createBinding(result, mod, k[i]);
      __setModuleDefault(result, mod);
      return result;
    };
  })();
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadGoRuntime = loadGoRuntime;
exports.loadWasmModule = loadWasmModule;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const zlib = __importStar(require("zlib"));
const url_1 = require("url");
/**
 * Load Go WASM runtime for Node.js.
 *
 * @internal
 * @param options - Loading options
 * @returns Go constructor class
 */
async function loadGoRuntime(options) {
  console.debug("[WASM] Loading Go runtime...");
  // Try to find wasm_exec.js in common locations
  const candidates = [];
  // First, try explicit wasmExecPath if provided
  if (options?.wasmExecPath) {
    candidates.push(options.wasmExecPath);
  }
  // Then try extension path (most reliable for VS Code extension)
  if (options?.extensionPath) {
    candidates.push(path.join(options.extensionPath, "wasm/wasm_exec.js"));
  }
  // Then try relative paths from bundled location
  candidates.push(
    path.join(__dirname, "../../../../wasm/wasm_exec.js"),
    path.join(__dirname, "../../../wasm/wasm_exec.js"),
    path.join(process.cwd(), "wasm/wasm_exec.js"),
    path.join(process.cwd(), "node_modules/@sruja/shared/wasm/wasm_exec.js")
  );
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      console.debug(`[WASM] Trying to load wasm_exec.js from: ${candidate}`);
      try {
        // wasm_exec.js is CommonJS, use dynamic import for Node.js
        // Clear cache to ensure fresh load
        const resolvedPath = path.resolve(candidate);
        // Use dynamic import instead of require()
        const wasmExec = await Promise.resolve(
          `${(0, url_1.pathToFileURL)(resolvedPath).href}`
        ).then((s) => __importStar(require(s)));
        if (global.Go) {
          console.debug("[WASM] Go runtime loaded successfully");
          return global.Go;
        } else if (wasmExec && wasmExec.Go) {
          console.debug("[WASM] Go runtime loaded successfully (from module)");
          return wasmExec.Go;
        } else {
          console.warn("[WASM] wasm_exec.js loaded but Go class not found on global");
        }
      } catch (error) {
        console.warn(`[WASM] Failed to load ${candidate} with import:`, error);
        // Continue to next candidate
      }
    }
  }
  throw new Error("wasm_exec.js not found. Please ensure WASM files are available.");
}
/**
 * Load WASM module for Node.js (supports compressed files).
 *
 * @internal
 * @param wasmPath - Path to WASM file
 * @param options - Loading options
 * @returns Promise that resolves when WASM is loaded and functions are ready
 */
async function loadWasmModule(wasmPath, options) {
  console.debug(`[WASM] Loading WASM module from: ${wasmPath}`);
  const Go = await loadGoRuntime(options);
  const go = new Go();
  console.debug("[WASM] Go instance created");
  let wasmBuffer;
  console.debug("[WASM] Reading WASM file...");
  const wasmData = fs.readFileSync(wasmPath);
  console.debug(`[WASM] WASM file read: ${wasmData.length} bytes`);
  // Check if file is gzip compressed (magic bytes: 1f 8b)
  if (wasmData.length >= 2 && wasmData[0] === 0x1f && wasmData[1] === 0x8b) {
    console.debug("[WASM] Decompressing gzip...");
    // Decompress gzip
    wasmBuffer = zlib.gunzipSync(wasmData);
    console.debug(`[WASM] Decompressed: ${wasmBuffer.length} bytes`);
  } else {
    // Use as-is (uncompressed)
    wasmBuffer = wasmData;
    console.debug("[WASM] Using uncompressed WASM");
  }
  // Use WebAssembly API available in Node.js
  // WebAssembly is available in Node.js 12+ and ES2020+
  // Type assertion needed because TypeScript doesn't always recognize WebAssembly in Node.js context
  const WebAssemblyAPI = globalThis.WebAssembly;
  console.debug("[WASM] Compiling WASM module...");
  const wasmModule = await WebAssemblyAPI.compile(wasmBuffer);
  console.debug("[WASM] WASM module compiled");
  console.debug("[WASM] Instantiating WASM module...");
  const instance = await WebAssemblyAPI.instantiate(wasmModule, go.importObject);
  console.debug("[WASM] WASM module instantiated");
  // Run the Go program - note: Go WASM main() typically blocks on a channel
  // to keep the program alive, so go.run() will never return.
  // However, functions are registered BEFORE the blocking, so we can check
  // for them after starting go.run() without waiting for it to complete.
  console.debug("[WASM] Starting Go program...");
  return new Promise((resolve, reject) => {
    let functionsReady = false;
    const startTime = Date.now();
    // Poll for functions to be available (they're registered before main() blocks)
    const checkInterval = setInterval(() => {
      const hasParseFn = !!global.sruja_parse_dsl;
      const hasDiagnosticsFn = !!global.sruja_get_diagnostics;
      if (hasParseFn && hasDiagnosticsFn) {
        const duration = Date.now() - startTime;
        console.debug(`[WASM] Functions registered after ${duration}ms`);
        functionsReady = true;
        clearInterval(checkInterval);
        clearTimeout(timeout);
        resolve();
      }
    }, 50); // Check every 50ms
    // Timeout if functions don't appear
    const timeout = setTimeout(() => {
      if (!functionsReady) {
        clearInterval(checkInterval);
        const hasAnyFn = !!global.sruja_parse_dsl;
        if (hasAnyFn) {
          console.debug("[WASM] Some functions available - continuing despite timeout");
          resolve();
        } else {
          console.error("[WASM] No functions registered after 10 seconds");
          reject(new Error("WASM functions not registered after 10 seconds"));
        }
      }
    }, 10000);
    // Start go.run() - it will block, but functions should be registered quickly
    try {
      // Run in setImmediate to avoid blocking the current tick
      setImmediate(() => {
        try {
          go.run(instance);
          // This will never return because main() blocks on a channel
          // But that's OK - we're checking for functions above
        } catch (error) {
          console.error("[WASM] go.run() threw error:", error);
          clearInterval(checkInterval);
          clearTimeout(timeout);
          reject(error);
        }
      });
    } catch (error) {
      console.error("[WASM] Failed to start go.run():", error);
      clearInterval(checkInterval);
      clearTimeout(timeout);
      reject(error);
    }
  });
}
//# sourceMappingURL=wasmLoader.js.map

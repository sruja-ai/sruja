// WASM initialization utilities
import type { CompileResult } from '../types';

/**
 * Initialize the namespaced Sruja object on window
 */
function initSrujaNamespace(): void {
  if (!window.sruja) {
    window.sruja = {
      wasmReady: false,
      wasmInitializing: false,
    };
  }
}

/**
 * Get or create the Sruja namespace
 */
function getSrujaNamespace(): NonNullable<Window['sruja']> {
  initSrujaNamespace();
  return window.sruja!;
}

export function setLogoSpin(on: boolean): void {
  const logo = document.querySelector('.nav-logo');
  if (logo) {
    if (on) logo.classList.add('spin');
    else logo.classList.remove('spin');
  }
}

function loadScript(src: string, onload?: () => void): void {
  const s = document.createElement('script');
  s.src = src;
  s.async = true;
  if (onload) s.onload = onload;
  document.head.appendChild(s);
}

export function initSrujaWasm(): void {
  const sruja = getSrujaNamespace();
  
  // Check if already initializing or ready
  if (sruja.wasmInitializing || sruja.wasmReady) return;
  
  // Support legacy globals for backward compatibility
  if (window.srujaWasmInitializing || window.srujaWasmReady) {
    // Sync state from legacy globals
    sruja.wasmReady = window.srujaWasmReady || false;
    sruja.wasmInitializing = window.srujaWasmInitializing || false;
    if (sruja.wasmReady || sruja.wasmInitializing) return;
  }
  
  sruja.wasmInitializing = true;
  // Legacy support
  window.srujaWasmInitializing = true;
  setLogoSpin(true);
  
  loadScript('/wasm_exec.js', () => {
    if (typeof window.Go === 'undefined') {
      console.error('Go WASM runtime not available');
      setLogoSpin(false);
      sruja.wasmInitializing = false;
      window.srujaWasmInitializing = false;
      return;
    }
    try {
      const go = new window.Go!();
      WebAssembly.instantiateStreaming(fetch('/sruja.wasm'), go.importObject)
        .then(result => {
          go.run(result.instance);
          sruja.wasmReady = true;
          sruja.wasmInitializing = false;
          sruja.compile = window.compileSruja || undefined;
          // Legacy support
          window.srujaWasmReady = true;
          window.srujaWasmInitializing = false;
          setLogoSpin(false);
        })
        .catch(err => {
          console.error('Failed to init Sruja WASM', err);
          setLogoSpin(false);
          sruja.wasmInitializing = false;
          window.srujaWasmInitializing = false;
        });
    } catch (e) {
      console.error('WASM init error', e);
      setLogoSpin(false);
      sruja.wasmInitializing = false;
      window.srujaWasmInitializing = false;
    }
  });
}

export function compileSrujaCode(code: string, filename: string, format: string = 'svg'): CompileResult | null {
  const sruja = getSrujaNamespace();
  
  // Check namespaced API first
  if (!sruja.wasmReady || !sruja.compile) {
    // Fallback to legacy API for backward compatibility
    if (!window.srujaWasmReady || typeof window.compileSruja === 'undefined') {
      return { error: 'WASM not ready' };
    }
    // Legacy API doesn't support format, default to svg
    return window.compileSruja(code, filename);
  }
  
  // Pass format as third argument
  return sruja.compile(code, filename, format);
}


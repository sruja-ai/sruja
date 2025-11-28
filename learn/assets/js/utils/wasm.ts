// WASM initialization utilities
import type { CompileResult } from '../types';

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
  if (window.srujaWasmInitializing || window.srujaWasmReady) return;
  window.srujaWasmInitializing = true;
  setLogoSpin(true);
  loadScript('/wasm_exec.js', () => {
    if (typeof window.Go === 'undefined') {
      console.error('Go WASM runtime not available');
      setLogoSpin(false);
      window.srujaWasmInitializing = false;
      return;
    }
    try {
      const go = new window.Go!();
      WebAssembly.instantiateStreaming(fetch('/sruja.wasm'), go.importObject)
        .then(result => {
          go.run(result.instance);
          window.srujaWasmReady = true;
          window.srujaWasmInitializing = false;
          setLogoSpin(false);
        })
        .catch(err => {
          console.error('Failed to init Sruja WASM', err);
          setLogoSpin(false);
          window.srujaWasmInitializing = false;
        });
    } catch (e) {
      console.error('WASM init error', e);
      setLogoSpin(false);
      window.srujaWasmInitializing = false;
    }
  });
}

export function compileSrujaCode(code: string, filename: string): CompileResult | null {
  if (!window.srujaWasmReady || typeof window.compileSruja === 'undefined') {
    return { error: 'WASM not ready' };
  }
  return window.compileSruja(code, filename);
}


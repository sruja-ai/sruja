import { initWasm as initSharedWasm, type WasmApi } from '@sruja/shared'

export async function initWasm(): Promise<WasmApi> {
  // In Astro, BASE_URL might be undefined, so we need to handle it properly
  // For studio page, base should be '/' to access /wasm/sruja.wasm
  let base = '/'
  if (typeof window !== 'undefined') {
    // Try to get BASE_URL from import.meta.env (Astro) or use current path
    const envBase = (import.meta as any).env?.BASE_URL
    if (envBase) {
      base = envBase
    } else {
      // Fallback: use current location pathname, but ensure it ends with /
      const pathname = window.location.pathname
      // If we're at /studio, base should be /
      // If we're at /studio/, base should be /studio/
      if (pathname.startsWith('/studio')) {
        // For studio page, use root as base since WASM is at /wasm/
        base = '/'
      } else {
        base = pathname.endsWith('/') ? pathname : pathname + '/'
      }
    }
  }
  return initSharedWasm({ base })
}

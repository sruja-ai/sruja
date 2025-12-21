// apps/vscode-extension/src/wasmLspProvider/cache.ts
// Caching logic for WASM LSP provider

import * as vscode from "vscode";
import type { Diagnostic } from "@sruja/shared/node/wasmAdapter";
import type { CachedDocument } from "./utils";
import { documentCache, isCachingEnabled, getCacheTTL, log } from "./utils";

// Get cached diagnostics or compute new ones
export async function getCachedDiagnostics(
  document: vscode.TextDocument,
  text: string,
  wasmApi: { getDiagnostics: (text: string) => Promise<Diagnostic[]> } | null
): Promise<Diagnostic[]> {
  // Check if caching is enabled
  if (!isCachingEnabled()) {
    if (!wasmApi) return [];
    return await wasmApi.getDiagnostics(text);
  }

  const cacheKey = document.uri.toString();
  const cached = documentCache.get(cacheKey);
  const cacheTTL = getCacheTTL();

  // Check if cache is valid
  if (cached && cached.text === text && Date.now() - cached.timestamp < cacheTTL) {
    log(`[Cache] Using cached diagnostics for ${document.fileName}`);
    return cached.diagnostics;
  }

  // Compute new diagnostics
  if (!wasmApi) return [];
  const diagnostics = await wasmApi.getDiagnostics(text);

  // Update cache
  documentCache.set(cacheKey, {
    text,
    diagnostics,
    symbols: cached?.symbols || [],
    timestamp: Date.now(),
  } as CachedDocument);

  return diagnostics;
}


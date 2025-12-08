// WASM initialization and DSL conversion
// Uses @sruja/shared WASM adapter for DSL → JSON conversion
// App core only understands JSON - this is a frontend layer

import { initWasm as initSharedWasm, type WasmApi } from '@sruja/shared';

let wasmApi: WasmApi | null = null;
let initPromise: Promise<WasmApi> | null = null;

export async function initWasm(): Promise<WasmApi> {
    if (wasmApi) return wasmApi;

    if (initPromise) return initPromise;

    initPromise = (async () => {
        // For standalone app, WASM is served from /wasm/
        let base = '/';
        if (typeof window !== 'undefined') {
            const envBase = (import.meta as any).env?.BASE_URL;
            if (envBase) {
                base = envBase;
            }
        }

        wasmApi = await initSharedWasm({ base });
        return wasmApi;
    })();

    return initPromise;
}

export async function getWasmApi(): Promise<WasmApi | null> {
    if (wasmApi) return wasmApi;

    try {
        return await initWasm();
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        return null;
    }
}

/**
 * Convert DSL string to Architecture JSON string
 * Returns parsed JSON object if successful, null on error
 */
export async function convertDslToJson(dsl: string): Promise<object | null> {
    const api = await getWasmApi();
    if (!api) {
        console.error('WASM not available');
        return null;
    }

    try {
        const jsonString = await api.parseDslToJson(dsl);
        return JSON.parse(jsonString);
    } catch (error) {
        console.error('DSL parse error:', error);
        return null;
    }
}

export { type WasmApi };

// apps/playground/src/wasm.ts
// Re-export WASM utilities from shared package
export {
  initWasmAuto as initWasm,
  getWasmApi,
  convertDslToJson,
  convertDslToMarkdown,
  convertDslToMermaid,
  type WasmApi,
} from "@sruja/shared";

// Helper function to convert DSL to LikeC4 JSON format
export async function convertDslToLikeC4(dsl: string, filename?: string): Promise<object | null> {
  const { getWasmApi } = await import("@sruja/shared");
  const api = await getWasmApi();
  if (!api || !api.dslToLikeC4) {
    return null;
  }
  try {
    const jsonStr = await api.dslToLikeC4(dsl, filename);
    return JSON.parse(jsonStr);
  } catch (error) {
    console.error("Failed to convert DSL to LikeC4 format:", error);
    throw error; // Re-throw to preserve error details
  }
}

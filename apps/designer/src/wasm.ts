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

// Helper function to convert DSL to Sruja model JSON format
export async function convertDslToModel(dsl: string, filename?: string): Promise<object | null> {
  const { getWasmApi } = await import("@sruja/shared");
  const api = await getWasmApi();
  if (!api || !api.dslToModel) {
    return null;
  }
  try {
    const jsonStr = await api.dslToModel(dsl, filename);
    return JSON.parse(jsonStr);
  } catch (error) {
    const { logger } = await import("@sruja/shared");
    logger.error("Failed to convert DSL to model format", {
      component: "wasm",
      action: "convert_dsl_to_model",
      error: error instanceof Error ? error.message : String(error),
    });
    throw error; // Re-throw to preserve error details
  }
}

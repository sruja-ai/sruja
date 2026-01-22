import { logger } from "../utils/logger";
import { ConfigurationError, NetworkError } from "../utils/errors";
import { ExportError } from "./errors";
import type { DotResult, DotElement, DotRelation, ScoreResult } from "./wasmTypes";
import type { WasmApi } from "./wasmAdapter";

type RustWasmModule = {
  default: (
    moduleOrPath?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
  ) => Promise<unknown>;
  init_panic_hook?: () => void;
  sruja_dsl_to_model: (dsl: string, filename?: string) => string;
  sruja_dsl_to_mermaid: (dsl: string, configJson?: string) => string;
  sruja_dsl_to_dot: (
    dsl: string,
    viewLevel?: number,
    targetId?: string,
    nodeSizesJson?: string,
    viewId?: string,
    filename?: string
  ) => string;
  sruja_dsl_to_markdown: (dsl: string) => string;
  sruja_model_to_dsl: (modelJson: string) => string;
  sruja_get_diagnostics: (dsl: string, filename?: string) => string;
  sruja_calculate_architecture_score: (dsl: string) => string;
};

function normalizeDotKind(kind: string): DotElement["kind"] {
  const k = (kind || "").toLowerCase();
  if (k === "database") return "datastore";
  if (k === "datastore") return "datastore";
  if (k === "person") return "person";
  if (k === "system") return "system";
  if (k === "container") return "container";
  if (k === "component") return "component";
  if (k === "queue") return "queue";
  return "system";
}

function defaultSizeForKind(kind: DotElement["kind"]): { width: number; height: number } {
  switch (kind) {
    case "person":
      return { width: 200, height: 180 };
    case "system":
      return { width: 220, height: 140 };
    case "container":
      return { width: 200, height: 120 };
    case "component":
      return { width: 180, height: 100 };
    case "datastore":
    case "queue":
      return { width: 200, height: 100 };
    default:
      return { width: 200, height: 120 };
  }
}

function buildDotResultFromModel(modelJson: string, dot: string): DotResult {
  const parsed = JSON.parse(modelJson) as {
    elements?: Record<
      string,
      {
        id: string;
        kind: string;
        title: string;
        description?: string | null;
        technology?: string | null;
        parent?: string | null;
      }
    >;
    relations?: Array<{
      source: string;
      target: string;
      title?: string | null;
      description?: string | null;
    }>;
  };

  const elements: DotElement[] = Object.values(parsed.elements || {}).map((e) => {
    const kind = normalizeDotKind(e.kind);
    const { width, height } = defaultSizeForKind(kind);
    return {
      id: e.id,
      kind,
      title: e.title,
      technology: e.technology ?? undefined,
      description: e.description ?? undefined,
      parentId: e.parent ?? undefined,
      width,
      height,
    };
  });

  const relations: DotRelation[] = (parsed.relations || []).map((r) => ({
    from: r.source,
    to: r.target,
    label: r.title ?? r.description ?? undefined,
  }));

  return { dot, elements, relations };
}

async function dynamicImportRustWasm(jsUrl: string): Promise<RustWasmModule> {
  try {
    // Files in public/ cannot be imported as modules in Vite/Astro
    // We need to fetch the module code and create a blob URL for import
    if (typeof window !== "undefined") {
      // Browser environment: fetch and create blob URL
      const response = await fetch(jsUrl);
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM module: ${response.statusText}`);
      }
      const code = await response.text();

      // Create a blob URL for the module code
      const blob = new Blob([code], { type: "application/javascript" });
      const blobUrl = URL.createObjectURL(blob);

      try {
        // Import from the blob URL (this works because it's not in public/)
        const module = await import(blobUrl);
        // Clean up the blob URL after import
        URL.revokeObjectURL(blobUrl);
        return module as RustWasmModule;
      } catch (importError) {
        URL.revokeObjectURL(blobUrl);
        throw importError;
      }
    } else {
      // SSR/Node environment: use direct import (won't work in production but that's OK)
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore - dynamic URL import for public asset in SSR
      return (await import(/* @vite-ignore */ jsUrl)) as RustWasmModule;
    }
  } catch (e) {
    throw new NetworkError("Failed to import Rust WASM JS loader", {
      url: jsUrl,
      cause: e instanceof Error ? e : new Error(String(e)),
    });
  }
}

export async function initRustWasm(options?: { base?: string }): Promise<WasmApi> {
  const base = (options?.base || "/").replace(/\/?$/, "/");

  // Serve wasm-pack output from public assets:
  // - /wasm/rust/sruja_wasm.js
  // - /wasm/rust/sruja_wasm_bg.wasm
  const isDev = typeof process !== "undefined" && process.env?.NODE_ENV === "development";
  const cacheBuster = isDev ? Date.now() : `v${Date.now()}`;
  const jsUrl = `${base}wasm/rust/sruja_wasm.js?v=${cacheBuster}`;
  const wasmUrl = `${base}wasm/rust/sruja_wasm_bg.wasm?v=${cacheBuster}`;

  const mod = await dynamicImportRustWasm(jsUrl);
  try {
    await mod.default(wasmUrl);
    // Initialize panic hook for better error messages
    if (typeof mod.init_panic_hook === "function") {
      mod.init_panic_hook();
    }
  } catch (e) {
    throw new ConfigurationError("Failed to initialize Rust WASM module", {
      configKey: "rust_wasm_init",
      context: { jsUrl, wasmUrl, error: e instanceof Error ? e.message : String(e) },
    });
  }

  // Helper to wrap Rust WASM calls (which throw on error) to match Go WASM behavior
  const wrapRustCall = <T>(fn: () => T): T => {
    try {
      return fn();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      logger.error("Rust WASM call failed", {
        component: "wasm",
        action: "rust_call",
        error: msg,
      });
      throw new ExportError("RUST_5001", `Rust WASM error: ${msg}`);
    }
  };

  return {
    parseDslToJson: async (dsl: string, filename?: string) => {
      return wrapRustCall(() => mod.sruja_dsl_to_model(dsl, filename));
    },
    modelToDsl: async (modelJson: string) => {
      return wrapRustCall(() => mod.sruja_model_to_dsl(modelJson));
    },
    dslToMermaid: async (dsl: string) => {
      return wrapRustCall(() => mod.sruja_dsl_to_mermaid(dsl));
    },
    dslToMarkdown: async (dsl: string) => {
      return wrapRustCall(() => mod.sruja_dsl_to_markdown(dsl));
    },
    dslToModel: async (dsl: string, filename?: string) => {
      return wrapRustCall(() => mod.sruja_dsl_to_model(dsl, filename));
    },
    dslToDot: async (
      dsl: string,
      viewLevel?: number,
      focusNodeId?: string,
      nodeSizes?: Record<string, { width: number; height: number }>,
      viewId?: string,
      filename?: string
    ) => {
      const modelJson = wrapRustCall(() => mod.sruja_dsl_to_model(dsl, filename));
      // Pass node sizes as JSON string, viewId, and filename
      const nodeSizesJson = nodeSizes ? JSON.stringify(nodeSizes) : undefined;
      const dot = wrapRustCall(() =>
        mod.sruja_dsl_to_dot(dsl, viewLevel, focusNodeId, nodeSizesJson, viewId, filename)
      );
      return buildDotResultFromModel(modelJson, dot);
    },
    calculateArchitectureScore: async (dsl: string) => {
      const raw = wrapRustCall(() => mod.sruja_calculate_architecture_score(dsl));
      // Rust returns JSON with keys `score`, etc. Normalize to existing ScoreResult shape.
      const parsed = JSON.parse(raw) as {
        score: number;
        grade?: string;
        categories?: Record<string, number>;
      };
      const result: ScoreResult = {
        Score: parsed.score,
        Grade: parsed.grade,
        Categories: {
          Structural: parsed.categories?.structural,
          Documentation: parsed.categories?.documentation,
          Complexity: parsed.categories?.complexity,
          Standardization: parsed.categories?.standardization,
          Traceability: parsed.categories?.traceability,
        },
      };
      return result;
    },
  };
}

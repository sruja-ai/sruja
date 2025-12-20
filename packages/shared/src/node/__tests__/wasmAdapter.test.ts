/**
 * @vitest-environment node
 */
import { describe, it, expect, beforeAll } from "vitest";
import * as path from "path";
import * as fs from "fs";
import { initWasmNode, NodeWasmApi } from "../wasmAdapter";

describe("WasmAdapter Integration", () => {
    let wasmApi: NodeWasmApi;
    const projectRoot = path.resolve(__dirname, "../../../../../");
    const wasmPath = path.join(projectRoot, "apps/website/public/wasm/sruja.wasm");
    const wasmExecPath = path.join(projectRoot, "apps/website/public/wasm/wasm_exec.js");

    beforeAll(async () => {
        // Ensure WASM files exist
        if (!fs.existsSync(wasmPath)) {
            throw new Error(`WASM file not found at ${wasmPath}. Run 'make wasm' first.`);
        }
        if (!fs.existsSync(wasmExecPath)) {
            throw new Error(`wasm_exec.js not found at ${wasmExecPath}.`);
        }

        wasmApi = await initWasmNode({
            wasmPath: wasmPath,
            wasmExecPath: wasmExecPath,
        });
    }, 30000); // 30s timeout for WASM initialization

    it("should parse DSL to JSON", async () => {
        const dsl = `
      model {
        system App "App" {
          description "A test app"
        }
      }
    `;
        const jsonStr = await wasmApi.parseDslToJson(dsl);
        const json = JSON.parse(jsonStr);

        expect(json).toBeDefined();
        expect(json.elements).toBeDefined();
        expect(json.elements["App"]).toBeDefined();
        expect(json.elements["App"].kind).toBe("system");
        expect(json.elements["App"].title).toBe("App");
    });

    it("should generate Mermaid diagram", async () => {
        const dsl = `
      model {
        system App "App" {}
        system DB "Database" {}
        App -> DB "Uses"
      }
    `;
        const mermaid = await wasmApi.dslToMermaid(dsl);
        expect(mermaid).toContain("graph");
        expect(mermaid).toContain("App");
        expect(mermaid).toContain("DB");
    });

    it("should generate Markdown report", async () => {
        const dsl = `
      model {
        system App "App" {
          description "A test app"
        }
      }
    `;
        const markdown = await wasmApi.dslToMarkdown(dsl);
        expect(markdown).toContain("# Architecture");
        expect(markdown).toContain("App");
    });

    it("should catch syntax errors in diagnostics", async () => {
        const malformedDsl = `
      model {
        !!!invalid_keyword App "App" {}
      }
    `;
        const diagnostics = await wasmApi.getDiagnostics(malformedDsl);
        expect(diagnostics.length).toBeGreaterThan(0);
        expect(diagnostics[0].severity).toBe("Error");
    });

    it("should provide hover information", async () => {
        const dsl = `
model {
  system App "App" {
    description "A test app"
  }
}
    `;
        // Hover over "App" (Line 3, Column 10-12)
        const hover = await wasmApi.hover(dsl, 3, 10);
        expect(hover).toBeDefined();
        expect(hover?.contents).toBeDefined();
        expect(hover?.contents).toContain("App");
        expect(hover?.contents.toLowerCase()).toContain("system");
    });
});

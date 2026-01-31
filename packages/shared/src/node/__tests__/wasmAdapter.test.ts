/**
 * @vitest-environment node
 */
import { describe, it, expect, beforeEach, vi } from "vitest";
import { initWasmNode, convertDslToMarkdown, convertDslToMermaid } from "../wasmAdapter";
import { ConfigurationError } from "../../utils/errors";

describe("WasmAdapter (Node.js Rust WASM)", () => {
  beforeEach(() => {
    // Clear any cached WASM state between tests
    vi.clearAllMocks();
  });

  describe("initWasmNode", () => {
    it("throws ConfigurationError when extensionPath is missing", async () => {
      await expect(initWasmNode()).rejects.toThrow(ConfigurationError);
      await expect(initWasmNode()).rejects.toThrow("extensionPath is required");
    });

    it("throws ConfigurationError when extensionPath is provided but WASM files are missing", async () => {
      const invalidPath = "/nonexistent/path";
      await expect(initWasmNode({ extensionPath: invalidPath })).rejects.toThrow(
        ConfigurationError
      );
    });

    // Note: Full e2e test with actual WASM files would require:
    // 1. Building WASM with wasm-pack --target nodejs
    // 2. Placing files in test fixture directory
    // This is tested in the VSCode extension integration tests
  });

  describe("convertDslToMarkdown", () => {
    it("returns null when WASM is not initialized", async () => {
      const result = await convertDslToMarkdown("architecture Test {}");
      expect(result).toBeNull();
    });
  });

  describe("convertDslToMermaid", () => {
    it("returns null when WASM is not initialized", async () => {
      const result = await convertDslToMermaid("architecture Test {}");
      expect(result).toBeNull();
    });
  });
});

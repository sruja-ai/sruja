/**
 * @vitest-environment node
 */
import { describe, it, expect } from "vitest";
import { initWasmNode } from "../wasmAdapter";

describe("WasmAdapter (Rust-only, Go removed)", () => {
  it("initWasmNode throws with clear message", async () => {
    await expect(initWasmNode()).rejects.toThrow(
      "Go WASM support has been removed. Use Rust WASM in browser only"
    );
  });
});

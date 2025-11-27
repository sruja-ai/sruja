// pkg/kernel/wasm/wasm_loader.js
// JavaScript loader for Sruja Kernel WASM module

/**
 * SrujaKernelWASM wraps the WebAssembly kernel module.
 */
export class SrujaKernelWASM {
  constructor() {
    this.go = null;
    this.kernel = null;
    this.initialized = false;
  }

  /**
   * Initialize the WASM kernel module.
   * @param {string} wasmPath - Path to kernel.wasm file
   * @param {Object} goRuntime - Go WASM runtime object (from wasm_exec.js)
   * @returns {Promise<void>}
   */
  async init(wasmPath, goRuntime) {
    if (this.initialized) {
      console.warn("Kernel already initialized");
      return;
    }

    this.go = goRuntime;

    try {
      // Instantiate WASM module
      const wasmModule = await WebAssembly.instantiateStreaming(
        fetch(wasmPath),
        this.go.importObject
      );

      // Run Go runtime
      this.go.run(wasmModule.instance);

      // Wait for kernel to be ready
      await this.waitForKernel();

      // Call init
      const result = this.kernel.init();
      if (!result.success) {
        throw new Error("Kernel initialization failed");
      }

      this.initialized = true;
      console.log("[SrujaKernel] Initialized successfully");
    } catch (error) {
      console.error("[SrujaKernel] Initialization failed:", error);
      throw error;
    }
  }

  /**
   * Wait for global kernel object to be available.
   */
  waitForKernel() {
    return new Promise((resolve, reject) => {
      let attempts = 0;
      const maxAttempts = 50;
      const interval = 100;

      const check = () => {
        if (globalThis.srujaKernel) {
          this.kernel = globalThis.srujaKernel;
          resolve();
        } else if (attempts++ < maxAttempts) {
          setTimeout(check, interval);
        } else {
          reject(new Error("Kernel object not found after initialization"));
        }
      };

      check();
    });
  }

  /**
   * Execute a notebook cell.
   * @param {string} code - Cell source code
   * @param {string} cellId - Unique cell identifier
   * @param {string} cellType - Cell type (dsl, query, diagram, validation)
   * @returns {Promise<Object>} Execution result
   */
  async execute(code, cellId, cellType = "dsl") {
    this.ensureInitialized();
    return this.kernel.execute(code, cellId, cellType);
  }

  /**
   * Execute a SrujaQL query.
   * @param {string} query - Query string
   * @returns {Promise<Object>} Query result
   */
  async query(query) {
    this.ensureInitialized();
    return this.kernel.query(query);
  }

  /**
   * Generate a diagram.
   * @param {string} target - Diagram target (e.g., "system Billing")
   * @param {string} format - Output format (mermaid, d2, svg)
   * @returns {Promise<Object>} Diagram result
   */
  async diagram(target, format = "mermaid") {
    this.ensureInitialized();
    return this.kernel.diagram(target, format);
  }

  /**
   * Validate architecture code.
   * @param {string} code - Code to validate
   * @returns {Promise<Object>} Validation result
   */
  async validate(code) {
    this.ensureInitialized();
    return this.kernel.validate(code);
  }

  /**
   * Export architecture IR as JSON.
   * @returns {Promise<string>} IR JSON string
   */
  async exportIR() {
    this.ensureInitialized();
    const result = this.kernel.exportIR();
    if (!result.success) {
      throw new Error(result.error || "Failed to export IR");
    }
    return result.ir;
  }

  /**
   * Import architecture IR from JSON.
   * @param {string} irJSON - IR JSON string
   * @returns {Promise<void>}
   */
  async importIR(irJSON) {
    this.ensureInitialized();
    const result = this.kernel.importIR(irJSON);
    if (!result.success) {
      throw new Error(result.error || "Failed to import IR");
    }
  }

  /**
   * Reset kernel state.
   * @returns {Promise<void>}
   */
  async reset() {
    this.ensureInitialized();
    return this.kernel.reset();
  }

  /**
   * Get diagnostics for a cell.
   * @param {string} cellId - Cell identifier
   * @returns {Promise<Array>} Diagnostics array
   */
  async getDiagnostics(cellId) {
    this.ensureInitialized();
    const result = this.kernel.getDiagnostics(cellId);
    if (!result.success) {
      throw new Error(result.error || "Failed to get diagnostics");
    }
    return JSON.parse(result.diagnostics);
  }

  /**
   * Get autocomplete suggestions.
   * @param {string} code - Source code
   * @param {number} cursorPos - Cursor position
   * @returns {Promise<Array>} Completion matches
   */
  async autocomplete(code, cursorPos) {
    this.ensureInitialized();
    const result = this.kernel.autocomplete(code, cursorPos);
    return result.matches || [];
  }

  /**
   * Get hover/inspection information.
   * @param {string} code - Source code
   * @param {number} cursorPos - Cursor position
   * @returns {Promise<Object>} Inspection result
   */
  async inspect(code, cursorPos) {
    this.ensureInitialized();
    return this.kernel.inspect(code, cursorPos);
  }

  /**
   * Create a snapshot.
   * @param {string} name - Snapshot name
   * @param {string} description - Optional description
   * @returns {Promise<Object>} Snapshot info
   */
  async createSnapshot(name, description = "") {
    this.ensureInitialized();
    const result = this.kernel.snapshot(name, description);
    if (!result.success) {
      throw new Error(result.error || "Failed to create snapshot");
    }
    return result.snapshot;
  }

  /**
   * Load a snapshot.
   * @param {string} name - Snapshot name
   * @returns {Promise<void>}
   */
  async loadSnapshot(name) {
    this.ensureInitialized();
    const result = this.kernel.loadSnapshot(name);
    if (!result.success) {
      throw new Error(result.error || "Failed to load snapshot");
    }
  }

  /**
   * List all snapshots.
   * @returns {Promise<Array>} Snapshots array
   */
  async listSnapshots() {
    this.ensureInitialized();
    const result = this.kernel.listSnapshots();
    if (!result.success) {
      throw new Error(result.error || "Failed to list snapshots");
    }
    return result.snapshots || [];
  }

  /**
   * Create a variant.
   * @param {string} name - Variant name
   * @param {string} baseSnapshot - Base snapshot name
   * @param {string} description - Optional description
   * @returns {Promise<Object>} Variant info
   */
  async createVariant(name, baseSnapshot = "", description = "") {
    this.ensureInitialized();
    const result = this.kernel.createVariant(name, baseSnapshot, description);
    if (!result.success) {
      throw new Error(result.error || "Failed to create variant");
    }
    return result.variant;
  }

  /**
   * Apply a variant.
   * @param {string} name - Variant name
   * @returns {Promise<void>}
   */
  async applyVariant(name) {
    this.ensureInitialized();
    const result = this.kernel.applyVariant(name);
    if (!result.success) {
      throw new Error(result.error || "Failed to apply variant");
    }
  }

  /**
   * List all variants.
   * @returns {Promise<Array>} Variants array
   */
  async listVariants() {
    this.ensureInitialized();
    const result = this.kernel.listVariants();
    if (!result.success) {
      throw new Error(result.error || "Failed to list variants");
    }
    return result.variants || [];
  }

  /**
   * Ensure kernel is initialized.
   */
  ensureInitialized() {
    if (!this.initialized || !this.kernel) {
      throw new Error("Kernel not initialized. Call init() first.");
    }
  }
}

/**
 * Create and initialize a new Sruja Kernel WASM instance.
 * @param {string} wasmPath - Path to kernel.wasm
 * @param {Object} goRuntime - Go WASM runtime (from wasm_exec.js)
 * @returns {Promise<SrujaKernelWASM>}
 */
export async function createKernel(wasmPath, goRuntime) {
  const kernel = new SrujaKernelWASM();
  await kernel.init(wasmPath, goRuntime);
  return kernel;
}


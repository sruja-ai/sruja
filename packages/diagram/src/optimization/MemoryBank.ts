/**
 * MemoryBank - Storage for successful layouts (few-shot examples)
 *
 * Stores C4 layouts that scored 1.0 on the auditor for use as
 * reference examples in future generation prompts.
 */
import * as fs from "fs";
import * as path from "path";
import type { ArchitectureJSON } from "../types";

/**
 * A successfully validated layout
 */
export interface SuccessfulLayout {
  /** Original prompt or description that generated this layout */
  prompt: string;
  /** The validated JSON structure */
  json: ArchitectureJSON;
  /** Score achieved (should be 1.0 for storage) */
  score: number;
  /** When this was added */
  timestamp: string;
  /** Optional category for retrieval */
  category?: string;
}

/**
 * Options for MemoryBank
 */
export interface MemoryBankOptions {
  /** Path to the storage file */
  storagePath?: string;
  /** Maximum layouts to store */
  maxLayouts?: number;
}

const DEFAULT_OPTIONS: Required<MemoryBankOptions> = {
  storagePath: "./high_score_layouts.json",
  maxLayouts: 100,
};

/**
 * MemoryBank - Persistent storage for successful layouts
 *
 * Usage:
 * ```typescript
 * const bank = new MemoryBank();
 * await bank.addLayout({ prompt, json, score: 1.0, timestamp: new Date().toISOString() });
 * const examples = await bank.getExamples(3);
 * ```
 */
export class MemoryBank {
  private options: Required<MemoryBankOptions>;
  private layouts: SuccessfulLayout[] = [];
  private loaded = false;

  constructor(options: MemoryBankOptions = {}) {
    this.options = { ...DEFAULT_OPTIONS, ...options };
  }

  /**
   * Add a successful layout to the bank
   */
  async addLayout(layout: SuccessfulLayout): Promise<void> {
    await this.ensureLoaded();

    // Only store layouts with perfect or near-perfect scores
    if (layout.score < 0.95) {
      console.warn(`[MemoryBank] Skipping layout with score ${layout.score} (min: 0.95)`);
      return;
    }

    // Check for duplicates based on prompt similarity
    const isDuplicate = this.layouts.some(
      (l) => l.prompt.toLowerCase().trim() === layout.prompt.toLowerCase().trim()
    );

    if (isDuplicate) {
      console.log(`[MemoryBank] Updating existing layout for prompt`);
      const idx = this.layouts.findIndex(
        (l) => l.prompt.toLowerCase().trim() === layout.prompt.toLowerCase().trim()
      );
      this.layouts[idx] = layout;
    } else {
      this.layouts.push(layout);
    }

    // Enforce max layouts (FIFO)
    if (this.layouts.length > this.options.maxLayouts) {
      this.layouts = this.layouts.slice(-this.options.maxLayouts);
    }

    await this.save();
    console.log(`[MemoryBank] Saved layout. Total: ${this.layouts.length}`);
  }

  /**
   * Get example layouts for few-shot prompting
   */
  async getExamples(limit: number = 3): Promise<SuccessfulLayout[]> {
    await this.ensureLoaded();

    // Return most recent high-scoring layouts
    return this.layouts
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, limit);
  }

  /**
   * Get examples by category
   */
  async getExamplesByCategory(category: string, limit: number = 3): Promise<SuccessfulLayout[]> {
    await this.ensureLoaded();

    return this.layouts
      .filter((l) => l.category === category)
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, limit);
  }

  /**
   * Get total count of stored layouts
   */
  async getCount(): Promise<number> {
    await this.ensureLoaded();
    return this.layouts.length;
  }

  /**
   * Clear all stored layouts
   */
  async clear(): Promise<void> {
    this.layouts = [];
    await this.save();
  }

  /**
   * Generate a system prompt section with few-shot examples
   */
  async generateFewShotPrompt(limit: number = 2): Promise<string> {
    const examples = await this.getExamples(limit);

    if (examples.length === 0) {
      return "";
    }

    const exampleStrings = examples.map(
      (ex, i) =>
        `Example ${i + 1} (Score: ${ex.score}):\n` +
        `Prompt: "${ex.prompt}"\n` +
        `JSON:\n${JSON.stringify(ex.json, null, 2)}`
    );

    return (
      `Here are examples of PERFECT layouts that scored 100% on the visual auditor:\n\n` +
      exampleStrings.join("\n\n---\n\n")
    );
  }

  private async ensureLoaded(): Promise<void> {
    if (this.loaded) return;

    try {
      if (fs.existsSync(this.options.storagePath)) {
        const content = fs.readFileSync(this.options.storagePath, "utf-8");
        const data = JSON.parse(content);
        this.layouts = data.layouts || [];
      }
    } catch (error) {
      console.warn(`[MemoryBank] Could not load from storage:`, error);
      this.layouts = [];
    }

    this.loaded = true;
  }

  private async save(): Promise<void> {
    const dir = path.dirname(this.options.storagePath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    const data = {
      version: 1,
      lastUpdated: new Date().toISOString(),
      layouts: this.layouts,
    };

    fs.writeFileSync(this.options.storagePath, JSON.stringify(data, null, 2));
  }
}

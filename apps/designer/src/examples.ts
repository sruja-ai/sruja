// Example files configuration
// Re-exports from @sruja/shared for convenience
// All examples are loaded from the shared examples service

import { getAvailableExamples, loadExampleFile, type Example } from "@sruja/shared";

export type { Example as ExampleFile };
export { getAvailableExamples, loadExampleFile };

/**
 * Get all examples (from shared service)
 * All examples from the shared service are DSL files
 */
export async function getAllExamples(): Promise<Array<Example & { isDsl: boolean }>> {
  const examples = await getAvailableExamples();
  return examples.map((ex) => ({
    ...ex,
    isDsl: !ex.file.endsWith(".json"),
  }));
}

/**
 * Fetch example DSL source (uses shared service)
 */
export async function fetchExampleDsl(filename: string): Promise<string> {
  return loadExampleFile(filename);
}

/**
 * Legacy: Fetch example JSON (pre-exported)
 * Note: Most examples are now DSL files, use fetchExampleDsl + convertDslToJson instead
 */
export async function fetchExampleJson(filename: string): Promise<object> {
  const text = await loadExampleFile(filename);
  return JSON.parse(text);
}

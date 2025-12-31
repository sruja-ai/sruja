// packages/shared/src/registry/propertyKeys.ts
// Registry of known property keys for architecture elements

/**
 * Known property keys for architecture elements.
 *
 * @public
 * @remarks
 * These are standard property keys that can be used in architecture definitions.
 * Organized by category: capacity, observability, compliance, and cost.
 */
export const propertyKeys: readonly string[] = [
  "capacity.instanceType",
  "capacity.instanceProvider",
  "capacity.readReplicas",
  "capacity.cache",
  "obs.metrics.application",
  "obs.alerting.critical",
  "obs.logging.application",
  "obs.tracing.tool",
  "obs.tracing.sampleRate",
  "compliance.pci.level",
  "compliance.pci.status",
  "compliance.soc2.type",
  "compliance.gdpr.status",
  "cost.monthly.total",
  "cost.monthly.compute",
  "cost.perTransaction.average",
] as const;

/**
 * Check if a position in text is inside a properties or metadata block.
 *
 * @public
 * @param text - Source text to check
 * @param positionLine - Line number (1-based) of the position
 * @param block - Type of block to check ('properties' or 'metadata')
 * @returns true if position is inside the specified block
 *
 * @remarks
 * Searches backwards from the position line to find the opening brace.
 * Checks if the block type appears before the brace. Handles nested blocks
 * by tracking brace depth.
 *
 * @example
 * ```typescript
 * const text = 'properties {\n  key: value\n}';
 * const inside = isInsideBlock(text, 2, 'properties');
 * // Returns: true
 * ```
 */
export function isInsideBlock(
  text: string,
  positionLine: number,
  block: "properties" | "metadata"
): boolean {
  // Input validation
  if (typeof text !== "string" || text.length === 0) {
    return false;
  }
  if (typeof positionLine !== "number" || positionLine < 1 || !Number.isInteger(positionLine)) {
    return false;
  }
  if (block !== "properties" && block !== "metadata") {
    return false;
  }

  const lines = text.split(/\r?\n/);
  const targetLineIndex = positionLine - 1; // Convert to 0-based index

  // Validate position is within text bounds
  if (targetLineIndex < 0 || targetLineIndex >= lines.length) {
    return false;
  }

  // Search backwards from position to find the opening brace
  // Track brace depth to handle nested structures
  let braceDepth = 0;
  let foundOpeningBrace = false;
  let lastClosingBraceLine = -1;

  // We scan backwards from the target line
  for (let i = targetLineIndex; i >= 0; i--) {
    const line = lines[i];
    const trimmedLine = line.trim();

    // Skip empty lines or comments
    if (!trimmedLine || trimmedLine.startsWith("//") || trimmedLine.startsWith("#")) {
      continue;
    }

    // Count closing braces (going backwards, these increase depth)
    const closingBraces = (line.match(/\}/g) || []).length;
    if (closingBraces > 0 && lastClosingBraceLine === -1) {
      lastClosingBraceLine = i;
    }
    braceDepth += closingBraces;

    // Count opening braces (going backwards, these decrease depth)
    const openingBraces = (line.match(/\{/g) || []).length;
    braceDepth -= openingBraces;

    // Check if this line contains the block declaration
    const blockNameRegex = new RegExp(`\\b${block}\\b`, "i");
    const hasBlockName = blockNameRegex.test(trimmedLine);
    const hasOpeningBrace = line.includes("{");

    // console.log(`[DEBUG] Line ${i + 1}: "${trimmedLine}" Depth:${braceDepth} Block:${block} Match:${hasBlockName} Open:${hasOpeningBrace}`);

    if (hasBlockName) {
      if (hasOpeningBrace) {
        // Standard case: "block {"
        // -1 means inside (e.g. { ... target ... )
        // 0 means on the same line if single line block (e.g. { target } )
        if (braceDepth < 0 || (braceDepth === 0 && i === targetLineIndex)) {
          return true;
        }
      } else {
        // Multiline case
        if (braceDepth < 0) return true;
      }
    } else if (hasOpeningBrace) {
      // If we encounter an opening brace for a different KNOWN block type,
      // we consider that a barrier and stop searching.
      // This allows 'properties { a { ... } }' (a is unknown, implies nested struct)
      // but rejects 'properties { metadata { ... } }' (metadata is known sibling)
      const BARRIER_BLOCKS = [
        "properties",
        "metadata",
        "checks",
        "views",
        "model",
        "specification",
        "element",
        "system",
        "container",
        "component",
        "deployment",
      ];

      const isBarrier = BARRIER_BLOCKS.some(
        (barrier) => barrier !== block && new RegExp(`\\b${barrier}\\b`, "i").test(trimmedLine)
      );

      // If we hit a barrier that we are "inside" (braceDepth < 0), stop.
      // But we only stop if this line actually CONTRIBUTED to the depth change that makes us inside?
      // braceDepth was decremented by openingBraces count.
      // If braceDepth < 0, we are inside *some* block defined on this line (or previous).
      // Since we scan line by line, and this line has `{`:
      // If this line defines 'metadata {', and we are inside it (braceDepth < 0),
      // then metadata "claims" us.
      if (isBarrier && braceDepth < 0) {
        return false;
      }
    }

    // If we found an opening brace at depth 0 (before finding block declaration),
    // we're not inside the target block
    if (braceDepth < 0 && !foundOpeningBrace) {
      foundOpeningBrace = true;
    }
  }

  return false;
}

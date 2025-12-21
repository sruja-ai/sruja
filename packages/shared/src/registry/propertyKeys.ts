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
  'capacity.instanceType',
  'capacity.instanceProvider',
  'capacity.readReplicas',
  'capacity.cache',
  'obs.metrics.application',
  'obs.alerting.critical',
  'obs.logging.application',
  'obs.tracing.tool',
  'obs.tracing.sampleRate',
  'compliance.pci.level',
  'compliance.pci.status',
  'compliance.soc2.type',
  'compliance.gdpr.status',
  'cost.monthly.total',
  'cost.monthly.compute',
  'cost.perTransaction.average',
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
  block: 'properties' | 'metadata'
): boolean {
  // Input validation
  if (typeof text !== 'string' || text.length === 0) {
    return false;
  }
  if (typeof positionLine !== 'number' || positionLine < 1 || !Number.isInteger(positionLine)) {
    return false;
  }
  if (block !== 'properties' && block !== 'metadata') {
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
  const blockPattern = new RegExp(`\\b${block}\\s*\\{`, 'i');

  for (let i = targetLineIndex; i >= 0; i--) {
    const line = lines[i] || '';
    const trimmedLine = line.trim();

    // Count closing braces (going backwards, these increase depth)
    const closingBraces = (line.match(/\}/g) || []).length;
    braceDepth += closingBraces;

    // Count opening braces (going backwards, these decrease depth)
    const openingBraces = (line.match(/\{/g) || []).length;
    braceDepth -= openingBraces;

    // Check if this line contains the block declaration
    if (blockPattern.test(trimmedLine)) {
      // If we've found the block declaration and we're at depth 0 or 1,
      // we're inside this block
      if (braceDepth <= 1) {
        return true;
      }
      // If depth > 1, we're in a nested block, not the target block
      return false;
    }

    // If we found an opening brace at depth 0 (before finding block declaration),
    // we're not inside the target block
    if (braceDepth < 0 && !foundOpeningBrace) {
      foundOpeningBrace = true;
      if (braceDepth < 0) {
        return false;
      }
    }
  }

  return false;
}

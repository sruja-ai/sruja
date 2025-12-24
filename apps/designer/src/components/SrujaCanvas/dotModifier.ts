/**
 * DOT String Modifier
 *
 * This module modifies Graphviz DOT strings to apply layout refinements
 * without regenerating from DSL.
 *
 * Architecture:
 * - Parses DOT string structure
 * - Modifies graph attributes (spacing, rankdir, etc.)
 * - Preserves node and edge definitions
 */

import type { LayoutOptions } from "./types";

/**
 * Modify DOT string with new layout options
 */
export function modifyDotString(dotString: string, options: LayoutOptions): string {
  // Parse and modify graph attributes
  let modified = dotString;

  // Update rankdir
  modified = replaceGraphAttribute(modified, "rankdir", `"${options.rankdir}"`);

  // Update nodesep (convert pixels to inches for Graphviz)
  const nodesepInches = options.nodesep / 72; // 72 points per inch
  modified = replaceGraphAttribute(modified, "nodesep", nodesepInches.toFixed(2));

  // Update ranksep (convert pixels to inches for Graphviz)
  const ranksepInches = options.ranksep / 72;
  modified = replaceGraphAttribute(modified, "ranksep", ranksepInches.toFixed(2));

  return modified;
}

/**
 * Replace or add a graph attribute
 */
function replaceGraphAttribute(dotString: string, attrName: string, attrValue: string): string {
  // Pattern to match graph attributes: graph [ ... ]
  const graphAttrPattern = /graph\s*\[([^\]]*)\]/s;
  const match = dotString.match(graphAttrPattern);

  if (!match) {
    // No graph attributes block, add one
    const digraphMatch = dotString.match(/digraph\s+(\w+)\s*\{/);
    if (digraphMatch) {
      const insertPos = digraphMatch.index! + digraphMatch[0].length;
      return (
        dotString.slice(0, insertPos) +
        `\n  graph [\n    ${attrName}=${attrValue}\n  ];\n` +
        dotString.slice(insertPos)
      );
    }
    return dotString;
  }

  const attrBlock = match[1];
  const attrPattern = new RegExp(`\\b${attrName}\\s*=\\s*[^,\\n\\]]+`, "g");

  if (attrPattern.test(attrBlock)) {
    // Replace existing attribute
    const newAttrBlock = attrBlock.replace(attrPattern, `${attrName}=${attrValue}`);
    return dotString.replace(graphAttrPattern, `graph [${newAttrBlock}]`);
  } else {
    // Add new attribute
    const newAttrBlock =
      attrBlock.trim() + (attrBlock.trim() ? ",\n" : "") + `    ${attrName}=${attrValue}`;
    return dotString.replace(graphAttrPattern, `graph [${newAttrBlock}]`);
  }
}

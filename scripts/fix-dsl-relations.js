#!/usr/bin/env node
/**
 * Fix common DSL syntax errors in Sruja files
 * Specifically fixes relations missing the "->" operator
 *
 * Usage: node scripts/fix-dsl-relations.js <file.sruja>
 */

const fs = require("fs");
const path = require("path");

if (process.argv.length < 3) {
  console.error("Usage: node scripts/fix-dsl-relations.js <file.sruja>");
  process.exit(1);
}

const filePath = process.argv[2];

if (!fs.existsSync(filePath)) {
  console.error(`Error: File ${filePath} not found`);
  process.exit(1);
}

// Read file
let content = fs.readFileSync(filePath, "utf8");
const lines = content.split("\n");
const fixedLines = [];
let changed = false;

// Pattern to detect: Two identifiers on same line without "->" between them
// This handles cases like: "Element1 Element2" or "Element1 REQ001"
const relationPattern = /^(\s*)([A-Za-z][A-Za-z0-9_.]*)\s+([A-Z][A-Za-z0-9_]*)\s*(".*")?$/;

for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  const lineNum = i + 1;

  // Skip comments and empty lines
  if (line.trim().startsWith("//") || line.trim() === "") {
    fixedLines.push(line);
    continue;
  }

  // Skip lines that already have "->"
  if (line.includes("->")) {
    fixedLines.push(line);
    continue;
  }

  // Skip requirement/ADR definitions (they use "=" not "->")
  if (line.match(/^\s*[A-Z][A-Za-z0-9_]*\s*=\s*(requirement|adr|policy|scenario|flow|slo)/)) {
    fixedLines.push(line);
    continue;
  }

  // Check if this looks like a relation missing "->"
  // Pattern: identifier identifier (possibly with quotes)
  const match = line.match(/^(\s*)([A-Za-z][A-Za-z0-9_.]*)\s+([A-Za-z][A-Za-z0-9_.]*)\s*(.*)$/);

  if (match && !line.includes("=") && !line.includes("{") && !line.includes("}")) {
    const [, indent, from, to, rest] = match;

    // Check if 'to' looks like it could be part of a relation
    // (not a keyword like "requirement", "adr", etc.)
    const keywords = [
      "requirement",
      "adr",
      "policy",
      "scenario",
      "flow",
      "slo",
      "kind",
      "view",
      "metadata",
      "overview",
    ];
    if (!keywords.includes(to.toLowerCase())) {
      // This might be a relation missing "->"
      // But be conservative - only fix if it's clearly a relation pattern
      // (has quotes or looks like element.element)
      if (rest.trim().startsWith('"') || from.includes(".") || to.includes(".")) {
        const fixed = `${indent}${from} -> ${to}${rest}`;
        console.log(`Line ${lineNum}: Fixed missing "->" operator`);
        console.log(`  Before: ${line.trim()}`);
        console.log(`  After:  ${fixed.trim()}`);
        fixedLines.push(fixed);
        changed = true;
        continue;
      }
    }
  }

  fixedLines.push(line);
}

if (changed) {
  // Create backup
  const backupPath = `${filePath}.bak`;
  fs.writeFileSync(backupPath, content);
  console.log(`\nBackup created: ${backupPath}`);

  // Write fixed content
  fs.writeFileSync(filePath, fixedLines.join("\n"));
  console.log(`Fixed file: ${filePath}`);
} else {
  console.log("No issues found to fix.");
}

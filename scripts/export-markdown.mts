#!/usr/bin/env zx
// scripts/export-markdown.mts
// TypeScript markdown exporter wrapper for CLI
// Converts JSON (from Go parser) to Markdown using TypeScript exporter

import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');

// Read JSON from stdin or file argument
let jsonInput = '';
if (process.argv[2]) {
  jsonInput = await fs.readFile(process.argv[2], 'utf-8');
} else {
  // Read from stdin
  const chunks = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  jsonInput = Buffer.concat(chunks).toString('utf-8');
}

if (!jsonInput.trim()) {
  console.error('Error: No JSON input provided');
  process.exit(1);
}

// Parse JSON to validate
let archJson;
try {
  archJson = JSON.parse(jsonInput);
} catch (err) {
  console.error('Error: Invalid JSON input:', err.message);
  process.exit(1);
}

// Import and use TypeScript markdown exporter
const { exportToMarkdown } = await import(
  join(projectRoot, 'packages/shared/src/export/markdown.ts')
);

const markdown = exportToMarkdown(archJson);
console.log(markdown);

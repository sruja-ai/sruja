#!/usr/bin/env zx
// scripts/check-missing-descriptions.mts
// Finds markdown files in website content without description or summary in frontmatter

import { readFileSync } from 'fs';
import { join } from 'path';
import { glob } from 'glob';

// Get script directory using process.cwd() since zx scripts run from repo root
const contentDir = join(process.cwd(), 'apps/website/src/content');

console.log('=== Pages Missing Descriptions ===\n');

let total = 0;
let missing = 0;

const files = await glob('**/*.md', { cwd: contentDir, absolute: true });

for (const file of files.sort()) {
  total++;
  
  const content = readFileSync(file, 'utf-8');
  const hasDesc = /^description:/m.test(content);
  const hasSummary = /^summary:/m.test(content);
  
  if (!hasDesc && !hasSummary) {
    missing++;
    const titleMatch = content.match(/^title:\s*"?(.+?)"?$/m);
    const title = titleMatch ? titleMatch[1] : '(no title)';
    const relativePath = file.replace(contentDir + '/', '');
    console.log(`[${missing}] ${relativePath}`);
    console.log(`    Title: ${title}\n`);
  }
}

console.log('=== Summary ===');
console.log(`Checked: ${total} files`);
console.log(`Missing descriptions: ${missing} files\n`);
console.log('Tip: Add \'description:\' or \'summary:\' to frontmatter of missing files');


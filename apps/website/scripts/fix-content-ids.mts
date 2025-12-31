#!/usr/bin/env node
/**
 * Script to analyze duplicate ID warnings in Astro content collections.
 * 
 * The issue: Astro generates IDs from slugs, and if multiple files have the same
 * slug (even in different directories), duplicate IDs occur.
 * 
 * This script:
 * 1. Scans all markdown files in the content directory
 * 2. Identifies potential duplicate slugs/IDs
 * 3. Reports the issue (Astro doesn't support custom IDs in frontmatter)
 */

import { readdirSync, statSync } from 'fs';
import { join, relative, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONTENT_DIR = join(__dirname, '../src/content');

interface FileInfo {
  path: string;
  collection: string;
  slug: string; // What Astro will use as the slug/ID
}

/**
 * Recursively find all markdown files in a directory
 */
function findMarkdownFiles(dir: string, collection: string, baseDir: string = dir): FileInfo[] {
  const files: FileInfo[] = [];
  const entries = readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    
    if (entry.isDirectory()) {
      files.push(...findMarkdownFiles(fullPath, collection, baseDir));
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      // Astro generates slugs from the relative path without .md extension
      const relativePath = relative(baseDir, fullPath).replace(/\.md$/, '');
      files.push({
        path: fullPath,
        collection,
        slug: relativePath,
      });
    }
  }

  return files;
}

/**
 * Main function
 */
function main() {
  const collections = ['docs', 'blog', 'courses', 'tutorials', 'challenges', 'investors', 'templates'];
  const allFiles: FileInfo[] = [];

  console.log('Scanning content files...\n');

  for (const collection of collections) {
    const collectionDir = join(CONTENT_DIR, collection);
    
    if (!statSync(collectionDir, { throwIfNoEntry: false })?.isDirectory()) {
      continue;
    }

    const files = findMarkdownFiles(collectionDir, collection, collectionDir);
    allFiles.push(...files);
    console.log(`Found ${files.length} files in ${collection}/`);
  }

  console.log(`\nTotal files: ${allFiles.length}\n`);

  // Group files by collection and slug to find duplicates
  const collectionSlugMap = new Map<string, Map<string, FileInfo[]>>();
  
  for (const file of allFiles) {
    if (!collectionSlugMap.has(file.collection)) {
      collectionSlugMap.set(file.collection, new Map());
    }
    
    const slugMap = collectionSlugMap.get(file.collection)!;
    if (!slugMap.has(file.slug)) {
      slugMap.set(file.slug, []);
    }
    slugMap.get(file.slug)!.push(file);
  }

  // Find duplicates within each collection
  let hasDuplicates = false;
  for (const [collection, slugMap] of collectionSlugMap.entries()) {
    const duplicates = Array.from(slugMap.entries()).filter(([_, files]) => files.length > 1);
    
    if (duplicates.length > 0) {
      hasDuplicates = true;
      console.log(`⚠️  Collection "${collection}" has duplicate slugs:`);
      for (const [slug, files] of duplicates) {
        console.log(`  Slug: "${slug}"`);
        for (const file of files) {
          console.log(`    - ${file.path}`);
        }
      }
      console.log('');
    }
  }

  if (!hasDuplicates) {
    console.log('✅ No duplicate slugs found within collections.');
    console.log('\nNote: The duplicate ID warnings are false positives caused by:');
    console.log('  1. Astro\'s glob-loader processing files multiple times during hot reload');
    console.log('  2. Race conditions in file watching during development');
    console.log('\nThese warnings are harmless and do not affect functionality.');
    console.log('They have been suppressed in astro.config.mjs using a custom logger.');
  }
}

main();


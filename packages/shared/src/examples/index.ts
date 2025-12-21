// @sruja/shared/src/examples/index.ts
// Shared examples loader for Studio and Viewer

import { isBrowser, getBaseUrl } from '../utils/env';

/**
 * Example definition from manifest.
 * 
 * @public
 */
export interface Example {
  /** Example file name */
  readonly file: string;
  /** Display name */
  readonly name: string;
  /** Sort order */
  readonly order: number;
  /** Category/group */
  readonly category: string;
  /** Description text */
  readonly description: string;
  /** Skip in playground */
  readonly skipPlayground?: boolean;
  /** Skip orphan check validation */
  readonly skipOrphanCheck?: boolean;
  /** Expected failure message (for testing) */
  readonly expectedFailure?: string;
}

/**
 * Examples manifest structure.
 * 
 * @public
 */
export interface ExamplesManifest {
  /** Array of example definitions */
  readonly examples: ReadonlyArray<Example>;
}


/**
 * Load examples manifest.
 * 
 * @public
 * @returns Examples manifest with all example definitions
 * 
 * @remarks
 * - In browser: fetches from {base}/examples/manifest.json
 * - In Node/build: reads from ../../examples/manifest.json
 * 
 * @example
 * ```typescript
 * const manifest = await loadExamplesManifest();
 * console.log(`Loaded ${manifest.examples.length} examples`);
 * ```
 */
export async function loadExamplesManifest(): Promise<ExamplesManifest> {
  if (isBrowser()) {
    // Browser: fetch from public/examples
    const baseUrl = getBaseUrl();
    const response = await fetch(`${baseUrl}/examples/manifest.json`);
    if (!response.ok) {
      throw new Error(`Failed to load examples manifest: ${response.status}`);
    }
    return await response.json();
  } else {
    // Node/build: read from file system
    const fsName = 'fs/promises';
    const pathName = 'path';
    const urlName = 'url';
    const fs = await import(/* @vite-ignore */ fsName);
    const path = await import(/* @vite-ignore */ pathName);
    const { fileURLToPath } = await import(/* @vite-ignore */ urlName);
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const manifestPath = path.resolve(__dirname, '../../../../examples/manifest.json');
    const content = await fs.readFile(manifestPath, 'utf-8');
    return JSON.parse(content);
  }
}

/**
 * Load example file content.
 * 
 * @public
 * @param filename - Example file name to load
 * @returns Example file content as string
 * 
 * @remarks
 * - In browser: fetches from {base}/examples/{file} with cache-busting
 * - In Node/build: reads from ../../examples/{file}
 * 
 * @example
 * ```typescript
 * const content = await loadExampleFile('hello-world.sruja');
 * console.log(content);
 * ```
 */
export async function loadExampleFile(filename: string): Promise<string> {
  if (isBrowser()) {
    // Browser: fetch from public/examples
    // Add cache-busting and prevent all caching
    const baseUrl = getBaseUrl();
    const cacheBuster = `?v=${Date.now()}&_=${Math.random()}`;
    const response = await fetch(`${baseUrl}/examples/${filename}${cacheBuster}`, {
      cache: 'no-store',
      headers: {
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'Pragma': 'no-cache',
        'Expires': '0'
      }
    });
    if (!response.ok) {
      throw new Error(`Failed to load example file: ${filename} (${response.status})`);
    }
    const text = await response.text();
    return text;
  } else {
    // Node/build: read from file system
    const fsName = 'fs/promises';
    const pathName = 'path';
    const urlName = 'url';
    const fs = await import(/* @vite-ignore */ fsName);
    const path = await import(/* @vite-ignore */ pathName);
    const { fileURLToPath } = await import(/* @vite-ignore */ urlName);
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const examplePath = path.resolve(__dirname, '../../../../examples', filename);
    return await fs.readFile(examplePath, 'utf-8');
  }
}

/**
 * Get available examples (filtered and sorted).
 * 
 * @public
 * @returns Array of examples available for playground (filtered and sorted by order)
 * 
 * @remarks
 * Filters out examples with skipPlayground flag and sorts by order.
 * 
 * @example
 * ```typescript
 * const examples = await getAvailableExamples();
 * examples.forEach(ex => console.log(ex.name));
 * ```
 */
export async function getAvailableExamples(): Promise<ReadonlyArray<Example>> {
  const manifest = await loadExamplesManifest();
  return manifest.examples
    .filter(ex => !ex.skipPlayground)
    .sort((a, b) => a.order - b.order);
}


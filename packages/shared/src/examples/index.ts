// @sruja/shared/src/examples/index.ts
// Shared examples loader for Studio and Viewer

export interface Example {
  file: string;
  name: string;
  order: number;
  category: string;
  description: string;
  skipPlayground?: boolean;
  skipOrphanCheck?: boolean;
  expectedFailure?: string;
}

export interface ExamplesManifest {
  examples: Example[];
}

// Load examples manifest
// In browser: fetches from /examples/manifest.json
// In Node/build: reads from ../../examples/manifest.json
export async function loadExamplesManifest(): Promise<ExamplesManifest> {
  if (typeof window !== 'undefined') {
    // Browser: fetch from public/examples
    const response = await fetch('/examples/manifest.json');
    if (!response.ok) {
      throw new Error(`Failed to load examples manifest: ${response.status}`);
    }
    return await response.json();
  } else {
    // Node/build: read from file system
    const fs = await import('fs/promises');
    const path = await import('path');
    const { fileURLToPath } = await import('url');
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const manifestPath = path.resolve(__dirname, '../../../../examples/manifest.json');
    const content = await fs.readFile(manifestPath, 'utf-8');
    return JSON.parse(content);
  }
}

// Load example file content
// In browser: fetches from /examples/{file}
// In Node/build: reads from ../../examples/{file}
export async function loadExampleFile(filename: string): Promise<string> {
  if (typeof window !== 'undefined') {
    // Browser: fetch from public/examples
    const response = await fetch(`/examples/${filename}`);
    if (!response.ok) {
      throw new Error(`Failed to load example file: ${filename} (${response.status})`);
    }
    return await response.text();
  } else {
    // Node/build: read from file system
    const fs = await import('fs/promises');
    const path = await import('path');
    const { fileURLToPath } = await import('url');
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const examplePath = path.resolve(__dirname, '../../../../examples', filename);
    return await fs.readFile(examplePath, 'utf-8');
  }
}

// Get available examples (filtered and sorted)
export async function getAvailableExamples(): Promise<Example[]> {
  const manifest = await loadExamplesManifest();
  return manifest.examples
    .filter(ex => !ex.skipPlayground)
    .sort((a, b) => a.order - b.order);
}


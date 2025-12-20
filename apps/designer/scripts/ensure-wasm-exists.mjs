// apps/designer/scripts/ensure-wasm-exists.mjs
// Check if WASM exists, if not wait a bit for website to build it
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const websiteWasmPath = join(__dirname, '..', '..', 'website', 'public', 'wasm', 'sruja.wasm');

// Check if WASM exists (website's ensure:wasm should have built it)
if (!existsSync(websiteWasmPath)) {
  console.warn('⚠️  WASM not found at', websiteWasmPath);
  console.warn('⚠️  Make sure @sruja/website has run ensure:wasm first');
  console.warn('⚠️  Designer will serve WASM from website directory when available');
  // Don't exit - let vite start, it will serve WASM when website builds it
}

// Check if WASM exists, if not wait a bit for website to build it
import { existsSync, mkdirSync, createWriteStream } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { get } from 'node:https';

const __dirname = dirname(fileURLToPath(import.meta.url));
const websiteWasmPath = join(__dirname, '..', '..', 'website', 'public', 'wasm', 'sruja.wasm');

// Check if WASM exists (website's ensure:wasm should have built it)
if (!existsSync(websiteWasmPath)) {
  console.warn('⚠️  WASM not found at', websiteWasmPath);
  console.warn('⚠️  Make sure @sruja/website has run ensure:wasm first');
}

const graphvizWasmDest = join(__dirname, '..', 'public', 'wasm', 'graphvizlib.wasm');
const graphvizWasmUrl = 'https://unpkg.com/@hpcc-js/wasm@1.17.0/dist/graphvizlib.wasm';

async function ensureGraphvizWasm() {
  if (existsSync(graphvizWasmDest)) {
    console.log('✅ graphvizlib.wasm already exists');
    return;
  }

  console.log('⬇️  Downloading graphvizlib.wasm from unpkg...');
  mkdirSync(dirname(graphvizWasmDest), { recursive: true });

  const file = createWriteStream(graphvizWasmDest);
  get(graphvizWasmUrl, (response) => {
    if (response.statusCode !== 200) {
      console.error(`❌ Failed to download graphvizlib.wasm: Status ${response.statusCode}`);
      return;
    }
    response.pipe(file);
    file.on('finish', () => {
      file.close();
      console.log('✅ Downloaded graphvizlib.wasm');
    });
  }).on('error', (err) => {
    console.error('❌ Error downloading graphvizlib.wasm:', err);
  });
}

ensureGraphvizWasm();

#!/usr/bin/env zx
// apps/vscode-extension/scripts/deploy.mts
/// <reference path="./zx.d.ts" />
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const vsix = resolve(__dirname, '..', 'sruja-language-support.vsix');

const installers = [
  { cmd: 'code', args: ['--install-extension', vsix] },
  { cmd: 'cursor', args: ['--install-extension', vsix] },
  { cmd: 'antigravity', args: ['--install-extension', vsix] },
  { cmd: 'trae', args: ['--install-extension', vsix] },
];

let any = false;
for (const i of installers) {
  try {
    await $`${i.cmd} --version`.quiet();
    console.log(`Installing with ${i.cmd}...`);
    try {
      await $`${i.cmd} ${i.args}`;
      console.log(`✅ Installed to ${i.cmd}`);
      any = true;
    } catch (installErr) {
      console.log(`⚠️  ${i.cmd} installation failed (CLI bug or permission issue)`);
      any = true; // VSIX was still built successfully
    }
  } catch {
    // Command not available, skip silently
  }
}

if (!any) {
  console.log(`No supported editor CLI found. VSIX built at: ${vsix}`);
}


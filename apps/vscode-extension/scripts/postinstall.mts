#!/usr/bin/env zx
// apps/vscode-extension/scripts/postinstall.mts
/// <reference path="./zx.d.ts" />
import { join, dirname } from 'path';
import { homedir } from 'os';
import { existsSync } from 'fs';
import { fileURLToPath } from 'url';

const isCI = process.env.CI === 'true' || process.env.CI === '1';
if (isCI) process.exit(0);
if (process.platform === 'win32') process.exit(0);
if (process.env.SRUJA_SKIP_POSTINSTALL === 'true') process.exit(0);

// Check if CLI already exists
try {
  await $`command -v sruja`.quiet();
  process.exit(0); // Already installed
} catch {
  // Continue with installation
}

const installDir = join(homedir(), '.local', 'bin');
await $`mkdir -p ${installDir}`;

// Get script directory - zx provides path utilities
const scriptDir = dirname(fileURLToPath(import.meta.url));
const localScript = join(scriptDir, '..', '..', '..', 'scripts', 'install.sh');
try {
  if (existsSync(localScript)) {
    await $`bash "${localScript}"`.env({ SRUJA_INSTALL_DIR: installDir });
  } else {
    await $`bash -lc "curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash"`.env({ SRUJA_INSTALL_DIR: installDir });
  }
} catch {
  // Ignore errors
}

// Check if installation succeeded
try {
  await $`command -v sruja`.quiet();
  console.log('Sruja CLI installed');
} catch {
  if (existsSync(join(installDir, 'sruja'))) {
    console.log('Sruja CLI installed');
  } else {
    console.log('Sruja CLI install skipped or failed');
  }
}


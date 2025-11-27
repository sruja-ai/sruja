// .vscode-extension/scripts/install-cursor.js
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const extensionDir = path.resolve(__dirname, '..');
const cursorExtensionsDir = path.join(process.env.HOME || process.env.USERPROFILE, '.cursor', 'extensions', 'sruja');

console.log('Installing Sruja extension to Cursor...');
console.log(`Source: ${extensionDir}`);
console.log(`Target: ${cursorExtensionsDir}`);

// Create target directory if it doesn't exist
const targetParent = path.dirname(cursorExtensionsDir);
if (!fs.existsSync(targetParent)) {
  fs.mkdirSync(targetParent, { recursive: true });
}

// Remove existing installation if it exists
if (fs.existsSync(cursorExtensionsDir)) {
  console.log('Removing existing installation...');
  fs.rmSync(cursorExtensionsDir, { recursive: true, force: true });
}

// Copy extension files
console.log('Copying extension files...');
fs.cpSync(extensionDir, cursorExtensionsDir, {
  recursive: true,
  filter: (src) => {
    const relativePath = path.relative(extensionDir, src);
    // Exclude .git and other unnecessary files, but include node_modules for dependencies
    if (relativePath.includes('.git')) return false;
    if (relativePath.startsWith('.DS_Store')) return false;
    if (relativePath.includes('scripts/install-cursor.js')) return false;
    if (relativePath.includes('scripts/install-trae.js')) return false;
    if (relativePath.includes('scripts/install-antigravity.js')) return false;
    if (relativePath.includes('scripts/verify-install.js')) return false;
    // Include node_modules - needed for vscode-languageclient
    return true;
  }
});

// Install dependencies in target location
console.log('Installing dependencies...');
try {
  execSync('npm install --production', { 
    cwd: cursorExtensionsDir,
    stdio: 'inherit'
  });
  console.log('✅ Dependencies installed');
} catch (err) {
  console.warn('⚠️ Failed to install dependencies - extension will work but LSP features may be disabled');
}

console.log('✅ Extension installed successfully!');
console.log('');
console.log('Next steps:');
console.log('1. Reload Cursor: Cmd+Shift+P → "Developer: Reload Window"');
console.log('2. Open a .sruja file');
console.log('3. Use command: Cmd+Shift+P → "Sruja: Show Diagram Preview"');




// .vscode-extension/scripts/install-antigravity.js
const fs = require('fs');
const path = require('path');
const os = require('os');

const extensionDir = path.resolve(__dirname, '..');

// Common Antigravity editor extension locations
function getAntigravityExtensionsDir() {
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  const platform = os.platform();
  
  // Try common locations
  const possibleLocations = [
    path.join(homeDir, '.antigravity', 'extensions', 'sruja'),
    path.join(homeDir, '.config', 'antigravity', 'extensions', 'sruja'),
    path.join(homeDir, 'Library', 'Application Support', 'Antigravity', 'extensions', 'sruja'), // macOS
    path.join(homeDir, 'AppData', 'Roaming', 'Antigravity', 'extensions', 'sruja'), // Windows
    path.join(homeDir, '.local', 'share', 'antigravity', 'extensions', 'sruja'), // Linux
    // If Antigravity uses VS Code-compatible extensions
    path.join(homeDir, '.antigravity', 'extensions', 'sruja'),
  ];
  
  // Check if any of these directories exist
  for (const loc of possibleLocations) {
    const parentDir = path.dirname(loc);
    if (fs.existsSync(parentDir)) {
      return loc;
    }
  }
  
  // Default to most common location
  return path.join(homeDir, '.antigravity', 'extensions', 'sruja');
}

const antigravityExtensionsDir = getAntigravityExtensionsDir();

console.log('Installing Sruja extension to Antigravity editor...');
console.log(`Source: ${extensionDir}`);
console.log(`Target: ${antigravityExtensionsDir}`);
console.log('');

// Create target directory if it doesn't exist
const targetParent = path.dirname(antigravityExtensionsDir);
if (!fs.existsSync(targetParent)) {
  console.log(`Creating directory: ${targetParent}`);
  fs.mkdirSync(targetParent, { recursive: true });
}

// Remove existing installation if it exists
if (fs.existsSync(antigravityExtensionsDir)) {
  console.log('Removing existing installation...');
  fs.rmSync(antigravityExtensionsDir, { recursive: true, force: true });
}

// Copy extension files
console.log('Copying extension files...');
fs.cpSync(extensionDir, antigravityExtensionsDir, {
  recursive: true,
  filter: (src) => {
    const relativePath = path.relative(extensionDir, src);
    // Exclude node_modules, .git, and other unnecessary files
    if (relativePath.includes('node_modules')) return false;
    if (relativePath.includes('.git')) return false;
    if (relativePath.startsWith('.DS_Store')) return false;
    if (relativePath.includes('scripts/install-antigravity.js')) return false;
    if (relativePath.includes('scripts/install-cursor.js')) return false;
    if (relativePath.includes('scripts/install-trae.js')) return false;
    return true;
  }
});

console.log('✅ Extension installed successfully!');
console.log('');
console.log('Next steps:');
console.log('1. Restart Antigravity editor');
console.log('2. Open a .sruja file');
console.log('3. If Antigravity supports VS Code extensions, use command: Cmd+Shift+P → "Sruja: Show Diagram Preview"');
console.log('');
console.log('Note: If Antigravity doesn\'t support VS Code extensions, you may need to configure LSP manually.');
console.log('See: docs/development/trae-editor-lsp-setup.md (similar setup for Antigravity)');


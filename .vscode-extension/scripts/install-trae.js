// .vscode-extension/scripts/install-trae.js
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const extensionDir = path.resolve(__dirname, '..');

// Common Trae editor extension locations
function getTraeExtensionsDir() {
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  const platform = os.platform();
  
  // Try common locations
  const possibleLocations = [
    path.join(homeDir, '.trae', 'extensions', 'sruja'),
    path.join(homeDir, '.config', 'trae', 'extensions', 'sruja'),
    path.join(homeDir, 'Library', 'Application Support', 'Trae', 'extensions', 'sruja'), // macOS
    path.join(homeDir, 'AppData', 'Roaming', 'Trae', 'extensions', 'sruja'), // Windows
    path.join(homeDir, '.local', 'share', 'trae', 'extensions', 'sruja'), // Linux
    // If Trae uses VS Code-compatible extensions
    path.join(homeDir, '.trae', 'extensions', 'sruja'),
  ];
  
  // Check if any of these directories exist
  for (const loc of possibleLocations) {
    const parentDir = path.dirname(loc);
    if (fs.existsSync(parentDir)) {
      return loc;
    }
  }
  
  // Default to most common location
  return path.join(homeDir, '.trae', 'extensions', 'sruja');
}

const traeExtensionsDir = getTraeExtensionsDir();

console.log('Installing Sruja extension to Trae editor...');
console.log(`Source: ${extensionDir}`);
console.log(`Target: ${traeExtensionsDir}`);
console.log('');

// Create target directory if it doesn't exist
const targetParent = path.dirname(traeExtensionsDir);
if (!fs.existsSync(targetParent)) {
  console.log(`Creating directory: ${targetParent}`);
  fs.mkdirSync(targetParent, { recursive: true });
}

// Remove existing installation if it exists
if (fs.existsSync(traeExtensionsDir)) {
  console.log('Removing existing installation...');
  fs.rmSync(traeExtensionsDir, { recursive: true, force: true });
}

// Copy extension files
console.log('Copying extension files...');
fs.cpSync(extensionDir, traeExtensionsDir, {
  recursive: true,
  filter: (src) => {
    const relativePath = path.relative(extensionDir, src);
    // Exclude .git and other unnecessary files, but include node_modules for dependencies
    if (relativePath.includes('.git')) return false;
    if (relativePath.startsWith('.DS_Store')) return false;
    if (relativePath.includes('scripts/install-trae.js')) return false;
    if (relativePath.includes('scripts/install-cursor.js')) return false;
    if (relativePath.includes('scripts/install-antigravity.js')) return false;
    if (relativePath.includes('scripts/verify-install.js')) return false;
    return true;
  }
});

// Install dependencies in target location
console.log('Installing dependencies...');
try {
  execSync('npm install --production', {
    cwd: traeExtensionsDir,
    stdio: 'inherit'
  });
  console.log('✅ Dependencies installed');
} catch (err) {
  console.warn('⚠️ Failed to install dependencies - extension will work but LSP features may be disabled');
}

console.log('✅ Extension installed successfully!');
console.log('');
console.log('Next steps:');
console.log('1. Restart Trae editor');
console.log('2. Open a .sruja file');
console.log('3. If Trae supports VS Code extensions, use command: Cmd+Shift+P → "Sruja: Show Diagram Preview"');
console.log('');
console.log('Note: If Trae doesn\'t support VS Code extensions, you may need to configure LSP manually.');
console.log('See: docs/development/trae-editor-lsp-setup.md');



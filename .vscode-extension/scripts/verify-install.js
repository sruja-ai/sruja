// .vscode-extension/scripts/verify-install.js
const fs = require('fs');
const path = require('path');
const os = require('os');

const homeDir = process.env.HOME || process.env.USERPROFILE;

console.log('Verifying Sruja extension installations...\n');

const editors = [
  { name: 'Cursor', path: path.join(homeDir, '.cursor', 'extensions', 'sruja') },
  { name: 'Trae', path: path.join(homeDir, '.trae', 'extensions', 'sruja') },
  { name: 'Antigravity', path: path.join(homeDir, '.antigravity', 'extensions', 'sruja') },
  { name: 'VS Code', path: path.join(homeDir, '.vscode', 'extensions', 'sruja') },
];

let found = false;

for (const editor of editors) {
  console.log(`Checking ${editor.name}...`);
  if (fs.existsSync(editor.path)) {
    console.log(`  ✅ Found at: ${editor.path}`);
    
    // Check key files
    const packageJson = path.join(editor.path, 'package.json');
    const extensionJs = path.join(editor.path, 'out', 'extension.js');
    
    if (fs.existsSync(packageJson)) {
      console.log(`  ✅ package.json exists`);
      const pkg = JSON.parse(fs.readFileSync(packageJson, 'utf8'));
      console.log(`     Version: ${pkg.version}`);
      console.log(`     Commands: ${pkg.contributes?.commands?.length || 0} registered`);
      
      // Check activation events
      const activationEvents = pkg.activationEvents || [];
      console.log(`     Activation: ${activationEvents.join(', ')}`);
    } else {
      console.log(`  ❌ package.json missing`);
    }
    
    if (fs.existsSync(extensionJs)) {
      const stats = fs.statSync(extensionJs);
      console.log(`  ✅ extension.js exists (${(stats.size / 1024).toFixed(1)} KB)`);
      console.log(`     Last modified: ${stats.mtime.toLocaleString()}`);
    } else {
      console.log(`  ❌ extension.js missing - need to compile!`);
    }
    
    found = true;
  } else {
    console.log(`  ❌ Not installed`);
  }
  console.log('');
}

if (!found) {
  console.log('❌ Extension not found in any editor!');
  console.log('\nTo install:');
  console.log('  npm run install:cursor    # For Cursor');
  console.log('  npm run install:trae     # For Trae');
  console.log('  npm run install:antigravity  # For Antigravity');
} else {
  console.log('✅ Extension found! Make sure to:');
  console.log('  1. Reload/restart your editor');
  console.log('  2. Open a .sruja file');
  console.log('  3. Check Output panel for "Sruja extension activating..." message');
}


// packages/html-viewer/scripts/bundle.ts
// Bundle all TypeScript compiled components into a single file
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const distDir = path.join(__dirname, '../dist');
const outputDir = path.join(__dirname, '../../../pkg/export/html/embed/components');
const outputFile = path.join(outputDir, 'sruja-components.js');

// Ensure output directory exists
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Component files in order (dependencies first)
const components = [
  'sruja-node-index.js',
  'sruja-svg-viewer.js',
  'sruja-info-panel.js',
  'sruja-viewer.js'
];

// V2 Components
const componentsV2 = [
  'sruja-node-index.js', // Reused
  'v2-layout.js',
  'v2-renderer.js',
  'v2-interaction.js',
  'v2-viewer.js'
];

const bundleFiles = (fileList: string[], outputName: string, stripModules = false): void => {
  let bundled = '// packages/html-viewer - Bundled web components\n';
  bundled += '// Auto-generated from TypeScript sources\n\n';

  fileList.forEach(component => {
    const filePath = path.join(distDir, component);
    if (fs.existsSync(filePath)) {
      let content = fs.readFileSync(filePath, 'utf8');

      if (stripModules) {
        // Remove import lines
        content = content.replace(/^import .*[\r\n]+/gm, '');
        // Remove export keyword from class/function definitions
        content = content.replace(/^export /gm, '');
      }

      bundled += `// === ${component} ===\n`;
      bundled += content;
      bundled += '\n\n';
    } else {
      console.warn(`Warning: ${component} not found in dist/`);
    }
  });

  const outPath = path.join(outputDir, outputName);
  fs.writeFileSync(outPath, bundled, 'utf8');
  console.log(`✅ Bundled components to ${outPath}`);
};

bundleFiles(components, 'sruja-components.js', true);
bundleFiles(componentsV2, 'sruja-v2-components.js', true);

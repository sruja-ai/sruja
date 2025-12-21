// packages/ui/scripts/copy-css.cjs
// Copy CSS files from src to dist maintaining directory structure
const fs = require("fs");
const path = require("path");

function copyDir(src, dest) {
  if (!fs.existsSync(dest)) {
    fs.mkdirSync(dest, { recursive: true });
  }

  const entries = fs.readdirSync(src, { withFileTypes: true });

  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);

    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
    } else if (entry.name.endsWith(".css")) {
      const destDir = path.dirname(destPath);
      if (!fs.existsSync(destDir)) {
        fs.mkdirSync(destDir, { recursive: true });
      }
      fs.copyFileSync(srcPath, destPath);
      console.log(`Copied ${srcPath} -> ${destPath}`);
    }
  }
}

const srcDir = path.join(__dirname, "../src");
const distDir = path.join(__dirname, "../dist");

if (fs.existsSync(srcDir)) {
  copyDir(srcDir, distDir);
  console.log("CSS files copied successfully");
} else {
  console.error("Source directory does not exist:", srcDir);
  process.exit(1);
}

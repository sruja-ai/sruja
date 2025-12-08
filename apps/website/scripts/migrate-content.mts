// apps/website/scripts/migrate-content.mts
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync, statSync } from 'fs';
import { join, dirname, relative } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '../../..');
const websiteDir = join(rootDir, 'apps/website');

// Files/directories to exclude
const excludePatterns = [
  'index.md',
  'courses.md',
  'tutorials.md',
  'node_modules',
  '.git',
  'authors.yml',
];

function shouldExclude(filePath: string): boolean {
  return excludePatterns.some(pattern => filePath.includes(pattern));
}

interface Frontmatter {
  title?: string;
  summary?: string;
  weight?: number;
  [key: string]: string | number | undefined;
}

function convertFrontmatter(content: string, filePath: string): { frontmatter: Frontmatter; body: string } {
  // Extract frontmatter
  const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)$/;
  const match = content.match(frontmatterRegex);
  
  if (!match) {
    return { frontmatter: {}, body: content };
  }

  const [, frontmatterText, body] = match;
  const frontmatter: Frontmatter = {};
  
  // Parse frontmatter
  frontmatterText.split('\n').forEach(line => {
    const colonIndex = line.indexOf(':');
    if (colonIndex > 0) {
      const key = line.substring(0, colonIndex).trim();
      let value = line.substring(colonIndex + 1).trim();
      
      // Remove quotes if present
      if ((value.startsWith('"') && value.endsWith('"')) || 
          (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1);
      }
      
      // Convert to appropriate type
      if (key === 'sidebar_position' || key === 'weight') {
        frontmatter[key === 'sidebar_position' ? 'weight' : key] = parseInt(value) || 999;
      } else if (key === 'title') {
        frontmatter.title = value;
      } else if (key === 'summary' || key === 'description') {
        frontmatter[key === 'description' ? 'summary' : key] = value;
      }
    }
  });

  // Ensure title exists
  if (!frontmatter.title) {
    // Extract from first heading or filename
    const headingMatch = body.match(/^#\s+(.+)$/m);
    if (headingMatch) {
      frontmatter.title = headingMatch[1].trim();
    } else {
      const filename = filePath.split('/').pop()?.replace('.md', '') || 'untitled';
      frontmatter.title = filename.charAt(0).toUpperCase() + filename.slice(1).replace(/-/g, ' ');
    }
  }

  return { frontmatter, body };
}

function processFile(sourcePath: string, targetPath: string, collectionType: string): void {
  if (!existsSync(sourcePath)) {
    console.log(`Skipping missing file: ${sourcePath}`);
    return;
  }

  const content = readFileSync(sourcePath, 'utf-8');
  const { frontmatter, body } = convertFrontmatter(content, sourcePath);

  // Clean up body - remove Docusaurus-specific components
  let cleanedBody = body
    .replace(/import\s+.*?from\s+['"]@site\/.*?['"];?\s*/g, '')
    .replace(/<SrujaCodeBlock[^>]*>[\s\S]*?<\/SrujaCodeBlock>/g, (match) => {
      // Extract code from SrujaCodeBlock
      const codeMatch = match.match(/code=\{`([\s\S]*?)`\}/);
      const filenameMatch = match.match(/filename="([^"]+)"/);
      if (codeMatch) {
        const code = codeMatch[1];
        const filename = filenameMatch ? filenameMatch[1] : 'example.sruja';
        return `\`\`\`sruja:${filename}\n${code}\n\`\`\``;
      }
      return '';
    });

  // Create frontmatter string
  const frontmatterString = Object.entries(frontmatter)
    .map(([key, value]) => {
      if (typeof value === 'string') {
        return `${key}: "${value}"`;
      }
      return `${key}: ${value}`;
    })
    .join('\n');

  const newContent = `---\n${frontmatterString}\n---\n\n${cleanedBody}`;

  // Ensure target directory exists
  mkdirSync(dirname(targetPath), { recursive: true });

  writeFileSync(targetPath, newContent, 'utf-8');
  console.log(`Migrated: ${relative(websiteDir, sourcePath)} -> ${relative(websiteDir, targetPath)}`);
}

function migrateDirectory(sourceDir: string, targetDir: string, collectionType: string): void {
  if (!existsSync(sourceDir)) {
    console.log(`Source directory does not exist: ${sourceDir}`);
    return;
  }

  const items = readdirSync(sourceDir);

  for (const item of items) {
    const sourcePath = join(sourceDir, item);
    const stat = statSync(sourcePath);

    if (stat.isDirectory()) {
      if (!shouldExclude(sourcePath)) {
        const targetPath = join(targetDir, item);
        migrateDirectory(sourcePath, targetPath, collectionType);
      }
    } else if (item.endsWith('.md') && !shouldExclude(sourcePath)) {
      const targetPath = join(targetDir, item);
      processFile(sourcePath, targetPath, collectionType);
    }
  }
}

// Migrate docs
console.log('Migrating docs...');
const docsSource = join(websiteDir, 'docs');
const docsTarget = join(websiteDir, 'src/content/docs');
migrateDirectory(docsSource, docsTarget, 'docs');

// Migrate blog
console.log('\nMigrating blog...');
const blogSource = join(websiteDir, 'blog');
const blogTarget = join(websiteDir, 'src/content/blog');
if (existsSync(blogSource)) {
  migrateDirectory(blogSource, blogTarget, 'blog');
}

// Migrate courses (if they exist in docs/courses)
console.log('\nMigrating courses...');
const coursesSource = join(websiteDir, 'docs/courses');
const coursesTarget = join(websiteDir, 'src/content/courses');
if (existsSync(coursesSource)) {
  migrateDirectory(coursesSource, coursesTarget, 'courses');
}

// Migrate tutorials (if they exist in docs/tutorials)
console.log('\nMigrating tutorials...');
const tutorialsSource = join(websiteDir, 'docs/tutorials');
const tutorialsTarget = join(websiteDir, 'src/content/tutorials');
if (existsSync(tutorialsSource)) {
  migrateDirectory(tutorialsSource, tutorialsTarget, 'tutorials');
}

console.log('\nMigration complete!');


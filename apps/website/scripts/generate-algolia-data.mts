#!/usr/bin/env node
/**
 * Generate Algolia import data file from Astro content collections
 * 
 * This script reads all content collections (docs, blog, courses, tutorials, challenges)
 * and generates a JSON file that can be imported into Algolia for search indexing.
 * 
 * Usage:
 *   npm run generate:algolia
 *   or
 *   node scripts/generate-algolia-data.mts
 */

import { writeFileSync, readFileSync, readdirSync, statSync } from 'fs';
import { join, extname, relative } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import yaml from 'js-yaml';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');
const contentDir = join(rootDir, 'src', 'content');

// Site URL - can be overridden via SITE_URL env var
const SITE_URL = process.env.SITE_URL || 'https://sruja.ai';

interface AlgoliaRecord {
  objectID: string;
  title: string;
  content: string;
  url: string;
  type: string;
  category?: string;
  summary?: string;
  description?: string;
  difficulty?: string;
  tags?: string[];
  weight?: number;
  pubDate?: string;
  authors?: Array<{ name: string; url?: string }>;
  topic?: string;
}

interface ContentItem {
  slug: string;
  data: unknown;
  body: string;
}

/**
 * Mock for Astro's getCollection to run standalone
 */
async function getCollection(collection: string): Promise<ContentItem[]> {
  const dir = join(contentDir, collection);
  const items: ContentItem[] = [];

  try {
    const files = getAllFiles(dir);
    for (const file of files) {
      if (extname(file) !== '.md' && extname(file) !== '.mdx') continue;

      const content = readFileSync(file, 'utf-8');
      const parts = content.split('---');

      if (parts.length < 3) continue;

      const frontmatter = yaml.load(parts[1]) as Record<string, unknown>;
      const body = parts.slice(2).join('---').trim();

      // Generate slug from file path relative to collection dir
      const relativePath = relative(dir, file);
      const slug = relativePath
        .replace(/\\/g, '/')
        .replace(/\.mdx?$/, '')
        .replace(/\/index$/, '');

      items.push({
        slug,
        data: frontmatter,
        body
      });
    }
  } catch (err) {
    console.warn(`Warning: Could not read collection ${collection}:`, err);
  }

  return items;
}

function getAllFiles(dirPath: string, arrayOfFiles: string[] = []): string[] {
  const files = readdirSync(dirPath);

  files.forEach((file) => {
    if (statSync(join(dirPath, file)).isDirectory()) {
      arrayOfFiles = getAllFiles(join(dirPath, file), arrayOfFiles);
    } else {
      arrayOfFiles.push(join(dirPath, file));
    }
  });

  return arrayOfFiles;
}

/**
 * Strip markdown formatting and extract plain text
 */
function stripMarkdown(content: string): string {
  return content
    // Remove code blocks
    .replace(/```[\s\S]*?```/g, "")
    // Remove inline code
    .replace(/`[^`]+`/g, "")
    // Remove links but keep text
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    // Remove images
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, "")
    // Remove headers
    .replace(/^#{1,6}\s+/gm, "")
    // Remove bold/italic
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/__([^_]+)__/g, "$1")
    .replace(/_([^_]+)_/g, "$1")
    // Remove horizontal rules
    .replace(/^---$/gm, "")
    // Remove list markers
    .replace(/^[*\-+]\s+/gm, "")
    .replace(/^\d+\.\s+/gm, "")
    // Remove blockquotes
    .replace(/^>\s+/gm, "")
    // Clean up extra whitespace
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/**
 * Generate URL for a content item based on its collection and slug
 */
function generateUrl(collection: string, slug: string): string {
  const basePath = collection === 'docs' ? '/docs' :
    collection === 'blog' ? '/blogs' :
      collection === 'courses' ? '/courses' :
        collection === 'tutorials' ? '/tutorials' :
          collection === 'challenges' ? '/challenges' :
            '';

  return `${SITE_URL}${basePath}/${slug}`;
}

/**
 * Process docs collection
 */
async function processDocs(): Promise<AlgoliaRecord[]> {
  const docs = await getCollection('docs');
  return docs.map((doc: ContentItem) => {
    const content = stripMarkdown(doc.body || "");
    const category = doc.slug.includes('/')
      ? doc.slug.split('/')[0]
      : 'docs';

    return {
      objectID: `docs-${doc.slug}`,
      title: doc.data?.title || doc.slug,
      content,
      url: generateUrl('docs', doc.slug),
      type: 'documentation',
      category,
      summary: doc.data?.summary,
      description: doc.data?.description,
      difficulty: doc.data?.difficulty,
      weight: doc.data?.weight,
    };
  });
}

/**
 * Process blog collection
 */
async function processBlog(): Promise<AlgoliaRecord[]> {
  const posts = await getCollection('blog');
  return posts.map((post: ContentItem) => {
    const content = stripMarkdown(post.body || "");
    const pubDate = post.data?.pubDate
      ? new Date(post.data.pubDate).toISOString().split('T')[0]
      : undefined;

    return {
      objectID: `blog-${post.slug}`,
      title: post.data?.title || post.slug,
      content,
      url: generateUrl('blog', post.slug),
      type: 'blog',
      summary: post.data?.description,
      description: post.data?.description,
      tags: post.data?.tags,
      pubDate,
      authors: post.data?.authors,
    };
  });
}

/**
 * Process courses collection
 */
async function processCourses(): Promise<AlgoliaRecord[]> {
  const courses = await getCollection('courses');
  return courses.map((course: ContentItem) => {
    const content = stripMarkdown(course.body || "");
    const slugParts = course.slug.split('/');
    const category = slugParts.length > 0 ? slugParts[0] : 'courses';

    return {
      objectID: `course-${course.slug}`,
      title: course.data?.title || course.slug,
      content,
      url: generateUrl('courses', course.slug),
      type: 'course',
      category,
      summary: course.data?.summary,
      description: course.data?.description,
      difficulty: course.data?.difficulty,
      weight: course.data?.weight,
      topic: course.data?.topic,
    };
  });
}

/**
 * Process tutorials collection
 */
async function processTutorials(): Promise<AlgoliaRecord[]> {
  const tutorials = await getCollection('tutorials');
  return tutorials.map((tutorial: ContentItem) => {
    const content = stripMarkdown(tutorial.body || "");
    const slugParts = tutorial.slug.split('/');
    const category = slugParts.length > 0 ? slugParts[0] : 'tutorials';

    return {
      objectID: `tutorial-${tutorial.slug}`,
      title: tutorial.data?.title || tutorial.slug,
      content,
      url: generateUrl('tutorials', tutorial.slug),
      type: 'tutorial',
      category,
      summary: tutorial.data?.summary,
      description: tutorial.data?.description,
      difficulty: tutorial.data?.difficulty,
      tags: tutorial.data?.tags,
      weight: tutorial.data?.weight,
      topic: tutorial.data?.topic,
    };
  });
}

/**
 * Process challenges collection
 */
async function processChallenges(): Promise<AlgoliaRecord[]> {
  const challenges = await getCollection('challenges');
  return challenges.map((challenge: ContentItem) => {
    const content = stripMarkdown(challenge.body || "");

    return {
      objectID: `challenge-${challenge.slug}`,
      title: challenge.data?.title || challenge.slug,
      content,
      url: generateUrl('challenges', challenge.slug),
      type: 'challenge',
      summary: challenge.data?.summary,
      difficulty: challenge.data?.difficulty,
      topic: challenge.data?.topic,
    };
  });
}

/**
 * Main function to generate Algolia data
 */
async function generateAlgoliaData() {
  console.log('🚀 Generating Algolia import data...\n');

  try {
    // Process all collections
    console.log('📚 Processing docs...');
    const docs = await processDocs();
    console.log(`   ✓ Found ${docs.length} docs`);

    console.log('📝 Processing blog posts...');
    const blog = await processBlog();
    console.log(`   ✓ Found ${blog.length} blog posts`);

    console.log('🎓 Processing courses...');
    const courses = await processCourses();
    console.log(`   ✓ Found ${courses.length} course items`);

    console.log('📖 Processing tutorials...');
    const tutorials = await processTutorials();
    console.log(`   ✓ Found ${tutorials.length} tutorials`);

    console.log('🎯 Processing challenges...');
    const challenges = await processChallenges();
    console.log(`   ✓ Found ${challenges.length} challenges`);

    // Combine all records
    const allRecords: AlgoliaRecord[] = [
      ...docs,
      ...blog,
      ...courses,
      ...tutorials,
      ...challenges,
    ];

    console.log(`\n✅ Total records: ${allRecords.length}`);

    // Write to file
    const outputPath = join(rootDir, 'algolia-data.json');
    writeFileSync(outputPath, JSON.stringify(allRecords, null, 2), 'utf-8');

    console.log(`\n📄 Algolia data written to: ${outputPath}`);
    console.log(`\n💡 Next steps:`);
    console.log(`   1. Review the generated file: ${outputPath}`);
    console.log(`   2. Import to Algolia using:`);
    console.log(`      algolia import -a <APP_ID> -k <ADMIN_API_KEY> -n <INDEX_NAME> -f ${outputPath}`);
    console.log(`   3. Or use Algolia Dashboard > Search > Indices > Import`);

    // Print summary by type
    console.log(`\n📊 Summary by type:`);
    const byType = allRecords.reduce((acc, record) => {
      acc[record.type] = (acc[record.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    Object.entries(byType).forEach(([type, count]) => {
      console.log(`   ${type}: ${count}`);
    });

  } catch (error) {
    console.error('❌ Error generating Algolia data:', error);
    process.exit(1);
  }
}

// Run the generator
generateAlgoliaData();

export { generateAlgoliaData };


// packages/shared/src/documentation/loader.ts
// Utility to load and parse markdown documentation files from learn app

export interface DocSection {
  id: string;
  title: string;
  content: string;
  url: string;
  summary?: string;
}

// Map node types to markdown file paths
const DOC_FILE_MAP: Record<string, { file: string; url: string }> = {
  person: { file: '../../apps/learn/docs/docs/concepts/person.md', url: '/learn/docs/docs/concepts/person' },
  system: { file: '../../apps/learn/docs/docs/concepts/system.md', url: '/learn/docs/docs/concepts/system' },
  container: { file: '../../apps/learn/docs/docs/concepts/container.md', url: '/learn/docs/docs/concepts/container' },
  datastore: { file: '../../apps/learn/docs/docs/concepts/datastore.md', url: '/learn/docs/docs/concepts/datastore' },
  queue: { file: '../../apps/learn/docs/docs/concepts/queue.md', url: '/learn/docs/docs/concepts/queue' },
  component: { file: '../../apps/learn/docs/docs/concepts/component.md', url: '/learn/docs/docs/concepts/component' },
  requirement: { file: '../../apps/learn/docs/docs/concepts/relations.md', url: '/learn/docs/docs/concepts/relations' },
  adr: { file: '../../apps/learn/docs/docs/concepts/adr.md', url: '/learn/docs/docs/concepts/adr' },
  deployment: { file: '../../apps/learn/docs/docs/concepts/deployment.md', url: '/learn/docs/docs/concepts/deployment' },
};

// Parse markdown frontmatter and content
function parseMarkdown(markdown: string): { frontmatter: Record<string, any>; content: string } {
  const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n/;
  const match = markdown.match(frontmatterRegex);
  
  if (!match) {
    return { frontmatter: {}, content: markdown };
  }

  const frontmatterText = match[1];
  const content = markdown.slice(match[0].length);
  
  const frontmatter: Record<string, any> = {};
  frontmatterText.split('\n').forEach(line => {
    const colonIndex = line.indexOf(':');
    if (colonIndex > 0) {
      const key = line.slice(0, colonIndex).trim();
      let value = line.slice(colonIndex + 1).trim();
      // Remove quotes if present
      if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1);
      }
      frontmatter[key] = value;
    }
  });

  return { frontmatter, content: content.trim() };
}

// Load markdown file (for build-time or runtime)
export async function loadDocSection(nodeType: string): Promise<DocSection | null> {
  const fileInfo = DOC_FILE_MAP[nodeType];
  if (!fileInfo) return null;

  try {
    // In browser/runtime, we'll need to fetch the markdown
    // For now, we'll use a fallback to the static content
    // In a production setup, this could fetch from a CDN or API
    const response = await fetch(fileInfo.file);
    if (!response.ok) {
      return null;
    }
    const markdown = await response.text();
    const { frontmatter, content } = parseMarkdown(markdown);
    
    return {
      id: nodeType,
      title: frontmatter.title || nodeType,
      content,
      url: fileInfo.url,
      summary: frontmatter.summary,
    };
  } catch (error) {
    console.warn(`Failed to load doc for ${nodeType}:`, error);
    return null;
  }
}

// Get all doc sections (for build-time generation)
export function getAllDocSections(): DocSection[] {
  return Object.keys(DOC_FILE_MAP).map(nodeType => ({
    id: nodeType,
    title: nodeType.charAt(0).toUpperCase() + nodeType.slice(1),
    content: '', // Will be loaded dynamically
    url: DOC_FILE_MAP[nodeType].url,
  }));
}


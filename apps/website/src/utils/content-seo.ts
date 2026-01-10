// apps/website/src/utils/content-seo.ts
// Utilities for content SEO improvements and validation

/**
 * Calculate content word count
 */
export function getWordCount(content: string): number {
  return content.trim().split(/\s+/).filter(Boolean).length;
}

/**
 * Check if content meets minimum SEO word count
 */
export function meetsMinimumWordCount(content: string, minimum: number = 300): boolean {
  return getWordCount(content) >= minimum;
}

/**
 * Extract headings from markdown content
 */
export function extractHeadings(content: string): Array<{ level: number; text: string }> {
  const headingRegex = /^(#{1,6})\s+(.+)$/gm;
  const headings: Array<{ level: number; text: string }> = [];
  let match;

  while ((match = headingRegex.exec(content)) !== null) {
    headings.push({
      level: match[1].length,
      text: match[2].trim(),
    });
  }

  return headings;
}

/**
 * Validate heading hierarchy (H1 should come before H2, etc.)
 */
export function validateHeadingHierarchy(
  headings: Array<{ level: number; text: string }>
): boolean {
  if (headings.length === 0) return false;

  // First heading should be H1 or H2 (H1 might be in frontmatter)
  if (headings[0].level > 2) return false;

  for (let i = 1; i < headings.length; i++) {
    const prev = headings[i - 1].level;
    const curr = headings[i].level;

    // Allow same level or one level deeper
    if (curr > prev + 1) {
      return false;
    }
  }

  return true;
}

/**
 * Count internal links in markdown content
 */
export function countInternalLinks(content: string, baseUrl: string = "https://sruja.ai"): number {
  // Match markdown links: [text](/path) or [text](https://sruja.ai/path)
  const linkRegex = /\[([^\]]+)\]\((https?:\/\/[^\)]+|\/[^\)]+)\)/g;
  let count = 0;
  let match;

  while ((match = linkRegex.exec(content)) !== null) {
    const url = match[2];
    // Check if it's an internal link
    if (url.startsWith("/") || url.includes(baseUrl)) {
      count++;
    }
  }

  return count;
}

/**
 * Extract keywords from content (simple frequency analysis)
 */
export function extractKeywords(
  content: string,
  minLength: number = 4,
  minFrequency: number = 2
): Array<{ word: string; frequency: number }> {
  // Remove markdown syntax, code blocks, URLs
  const cleanContent = content
    .replace(/```[\s\S]*?```/g, "") // Remove code blocks
    .replace(/`[^`]+`/g, "") // Remove inline code
    .replace(/\[([^\]]+)\]\([^\)]+\)/g, "$1") // Convert links to text
    .replace(/[#*_~`]/g, "") // Remove markdown formatting
    .toLowerCase();

  // Extract words
  const words = cleanContent.match(new RegExp(`\\b[a-z]{${minLength},}\\b`, "g")) || [];

  // Count frequencies
  const frequencyMap = new Map<string, number>();
  for (const word of words) {
    frequencyMap.set(word, (frequencyMap.get(word) || 0) + 1);
  }

  // Filter and sort
  return Array.from(frequencyMap.entries())
    .filter(([_, freq]) => freq >= minFrequency)
    .map(([word, frequency]) => ({ word, frequency }))
    .sort((a, b) => b.frequency - a.frequency)
    .slice(0, 10); // Top 10 keywords
}

/**
 * Generate FAQ schema from content (extracts questions from headings)
 */
export function generateFAQFromHeadings(
  headings: Array<{ level: number; text: string }>,
  _content: string
): Array<{ question: string; answer: string }> {
  const faqs: Array<{ question: string; answer: string }> = [];

  // Look for question-like headings (containing ?, How, What, Why, etc.)
  const questionPattern = /^(How|What|Why|When|Where|Which|Can|Should|Will|Is|Are|Does|Do)\s+.+\?/i;

  for (const heading of headings) {
    if (questionPattern.test(heading.text)) {
      // Extract answer from content following the heading
      // This is a simplified version - you'd need more sophisticated parsing
      faqs.push({
        question: heading.text,
        answer: "Answer extracted from content", // Placeholder
      });
    }
  }

  return faqs;
}

/**
 * Content SEO score calculator
 */
export interface ContentSEOScore {
  wordCount: number;
  hasHeadings: boolean;
  validHeadingHierarchy: boolean;
  internalLinks: number;
  keywordCount: number;
  score: number; // 0-100
  recommendations: string[];
}

function scoreWordCount(wordCount: number, recommendations: string[]): number {
  if (wordCount >= 300) {
    return 30;
  }
  if (wordCount >= 200) {
    recommendations.push(
      "Content is below recommended 300 words. Consider expanding with more details."
    );
    return 20;
  }
  recommendations.push("Content is too short for SEO. Minimum 300 words recommended.");
  return 10;
}

function scoreHeadings(
  headings: Array<{ level: number; text: string }>,
  recommendations: string[]
): number {
  if (headings.length >= 3) {
    return 20;
  }
  if (headings.length > 0) {
    recommendations.push("Add more headings to improve content structure and readability.");
    return 10;
  }
  recommendations.push("Add headings to improve content structure (H2, H3).");
  return 0;
}

function scoreInternalLinks(internalLinks: number, recommendations: string[]): number {
  if (internalLinks >= 3) {
    return 15;
  }
  if (internalLinks > 0) {
    recommendations.push("Add more internal links to related content.");
    return 10;
  }
  recommendations.push("Add internal links to improve navigation and SEO.");
  return 0;
}

function scoreMetaDescription(
  metaDescription: string | undefined,
  recommendations: string[]
): number {
  if (!metaDescription) {
    recommendations.push("Add a meta description for better search visibility.");
    return 0;
  }
  if (metaDescription.length >= 150 && metaDescription.length <= 160) {
    return 10;
  }
  recommendations.push("Optimize meta description to 150-160 characters.");
  return 5;
}

function scoreKeywords(
  keywords: Array<{ word: string; frequency: number }>,
  recommendations: string[]
): number {
  if (keywords.length >= 5) {
    return 10;
  }
  recommendations.push("Content could benefit from more keyword-rich phrases.");
  return 5;
}

function scoreHeadingHierarchy(validHierarchy: boolean, recommendations: string[]): number {
  if (validHierarchy) {
    return 15;
  }
  recommendations.push("Fix heading hierarchy - ensure H1 comes before H2, etc.");
  return 0;
}

export function calculateSEOScore(
  content: string,
  _title?: string,
  metaDescription?: string
): ContentSEOScore {
  const recommendations: string[] = [];
  const wordCount = getWordCount(content);
  const headings = extractHeadings(content);
  const internalLinks = countInternalLinks(content);
  const keywords = extractKeywords(content);
  const validHierarchy = validateHeadingHierarchy(headings);

  const score =
    scoreWordCount(wordCount, recommendations) +
    scoreHeadings(headings, recommendations) +
    scoreHeadingHierarchy(validHierarchy, recommendations) +
    scoreInternalLinks(internalLinks, recommendations) +
    scoreKeywords(keywords, recommendations) +
    scoreMetaDescription(metaDescription, recommendations);

  return {
    wordCount,
    hasHeadings: headings.length > 0,
    validHeadingHierarchy: validHierarchy,
    internalLinks,
    keywordCount: keywords.length,
    score,
    recommendations,
  };
}

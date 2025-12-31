// packages/shared/src/utils/__tests__/markdown.test.ts
// Unit tests for markdown utility functions

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  copyToClipboard,
  extractMermaidBlocks,
  extractCodeBlocks,
  sanitizeMarkdown,
  getMarkdownWordCount,
  getReadingTime,
  formatMarkdownForPreview,
} from "../markdown";

describe("markdown utilities", () => {
  describe("copyToClipboard", () => {
    beforeEach(() => {
      // Mock navigator.clipboard
      Object.defineProperty(navigator, "clipboard", {
        value: {
          writeText: vi.fn().mockResolvedValue(undefined),
        },
        writable: true,
        configurable: true,
      });
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("should copy text to clipboard using Clipboard API", async () => {
      const text = "test content";
      const result = await copyToClipboard(text);

      expect(result).toBe(true);
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(text);
    });

    it("should return false for empty text", async () => {
      const result = await copyToClipboard("");

      expect(result).toBe(false);
      expect(navigator.clipboard.writeText).not.toHaveBeenCalled();
    });

    it("should return false if clipboard API fails", async () => {
      const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (navigator.clipboard.writeText as any).mockRejectedValue(new Error("Clipboard error"));

      const result = await copyToClipboard("test");

      expect(result).toBe(false);
      consoleErrorSpy.mockRestore();
    });
  });

  describe("extractMermaidBlocks", () => {
    it("should extract mermaid code blocks", () => {
      const markdown = `
# Title

\`\`\`mermaid
graph TD
    A --> B
\`\`\`

Some text

\`\`\`mermaid
sequenceDiagram
    A->>B: Hello
\`\`\`
`;

      const blocks = extractMermaidBlocks(markdown);

      expect(blocks).toHaveLength(2);
      expect(blocks[0]).toContain("graph TD");
      expect(blocks[1]).toContain("sequenceDiagram");
    });

    it("should return empty array if no mermaid blocks", () => {
      const markdown = "# Title\n\nSome text";
      const blocks = extractMermaidBlocks(markdown);

      expect(blocks).toHaveLength(0);
    });

    it("should handle mermaid blocks with extra whitespace", () => {
      const markdown = "```mermaid\n  graph TD\n    A --> B\n  ```";
      const blocks = extractMermaidBlocks(markdown);

      expect(blocks).toHaveLength(1);
      expect(blocks[0].trim()).toContain("graph TD");
    });
  });

  describe("extractCodeBlocks", () => {
    it("should extract code blocks with language", () => {
      const markdown = `
\`\`\`javascript
console.log('hello');
\`\`\`

\`\`\`typescript
const x = 1;
\`\`\`
`;

      const blocks = extractCodeBlocks(markdown);

      expect(blocks).toHaveLength(2);
      expect(blocks[0].language).toBe("javascript");
      expect(blocks[1].language).toBe("typescript");
    });

    it("should extract code blocks without language", () => {
      const markdown = "```\ncode here\n```";
      const blocks = extractCodeBlocks(markdown);

      expect(blocks).toHaveLength(1);
      expect(blocks[0].language).toBeNull();
    });

    it("should filter by language when specified", () => {
      const markdown = `
\`\`\`javascript
code1
\`\`\`

\`\`\`typescript
code2
\`\`\`
`;

      const blocks = extractCodeBlocks(markdown, "javascript");

      expect(blocks).toHaveLength(1);
      expect(blocks[0].language).toBe("javascript");
    });
  });

  describe("sanitizeMarkdown", () => {
    it("should remove script tags", () => {
      const markdown = '# Title\n<script>alert("xss")</script>\nText';
      const sanitized = sanitizeMarkdown(markdown);

      expect(sanitized).not.toContain("<script>");
      expect(sanitized).toContain("# Title");
      expect(sanitized).toContain("Text");
    });

    it("should remove iframe tags", () => {
      const markdown = '# Title\n<iframe src="evil.com"></iframe>\nText';
      const sanitized = sanitizeMarkdown(markdown);

      expect(sanitized).not.toContain("<iframe>");
    });

    it("should remove javascript: protocol", () => {
      const markdown = '[Link](javascript:alert("xss"))';
      const sanitized = sanitizeMarkdown(markdown);

      expect(sanitized).not.toContain("javascript:");
    });

    it("should preserve normal markdown", () => {
      const markdown = "# Title\n\n**Bold** and *italic*";
      const sanitized = sanitizeMarkdown(markdown);

      expect(sanitized).toBe(markdown);
    });
  });

  describe("getMarkdownWordCount", () => {
    it("should count words excluding code blocks", () => {
      const markdown = `
# Title

This is a paragraph with words.

\`\`\`javascript
const code = "should not be counted";
\`\`\`

More text here.
`;

      const count = getMarkdownWordCount(markdown);

      expect(count).toBeGreaterThan(0);
      expect(count).toBeLessThan(20); // Should not include code block words
    });

    it("should exclude inline code", () => {
      const markdown = "This has `code` and more text.";
      const count = getMarkdownWordCount(markdown);

      expect(count).toBe(5); // "This has and more text" = 5 words (excluding "code")
    });

    it("should handle empty markdown", () => {
      const count = getMarkdownWordCount("");

      expect(count).toBe(0);
    });
  });

  describe("getReadingTime", () => {
    it("should calculate reading time", () => {
      const markdown = "Word ".repeat(200); // 200 words
      const time = getReadingTime(markdown, 200); // 200 words per minute

      expect(time).toBe(1); // 200 words / 200 wpm = 1 minute
    });

    it("should round up", () => {
      const markdown = "Word ".repeat(250); // 250 words
      const time = getReadingTime(markdown, 200); // 200 words per minute

      expect(time).toBe(2); // 250 words / 200 wpm = 1.25, rounded up to 2
    });

    it("should use default words per minute", () => {
      const markdown = "Word ".repeat(200);
      const time = getReadingTime(markdown);

      expect(time).toBe(1); // Default is 200 wpm
    });
  });

  describe("formatMarkdownForPreview", () => {
    it("should normalize line endings", () => {
      const markdown = "Line 1\r\nLine 2\rLine 3\nLine 4";
      const formatted = formatMarkdownForPreview(markdown);

      expect(formatted).not.toContain("\r\n");
      expect(formatted).not.toContain("\r");
      expect(formatted).toContain("\n");
    });

    it("should preserve content", () => {
      const markdown = "# Title\n\nContent here";
      const formatted = formatMarkdownForPreview(markdown);

      expect(formatted).toContain("# Title");
      expect(formatted).toContain("Content here");
    });
  });
});

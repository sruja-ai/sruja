// apps/website/src/pages/sitemap.xml.ts
// Dynamic sitemap generation for all content collections

import { getCollection, type CollectionEntry } from "astro:content";
import type { APIRoute } from "astro";
import { envConfig } from "@/config/env";

const siteUrl = envConfig.siteUrl || "https://sruja.ai";

export const GET: APIRoute = async () => {
  // Get all content collections
  const [blogs, docs, courses, tutorials, challenges] = await Promise.all([
    getCollection("blog"),
    getCollection("docs"),
    getCollection("courses"),
    getCollection("tutorials"),
    getCollection("challenges"),
  ]);

  // Static pages
  const staticPages = [
    "",
    "/blogs",
    "/docs",
    "/courses",
    "/tutorials",
    "/challenges",
    "/learn",
    "/designer",
    "/viewer",
  ];

  // Helper to format date
  const formatDate = (date: Date | undefined): string => {
    if (!date) return new Date().toISOString();
    return date.toISOString();
  };

  // Helper to get last modified date for content
  const getLastMod = (
    entry:
      | CollectionEntry<"blog">
      | CollectionEntry<"docs">
      | CollectionEntry<"courses">
      | CollectionEntry<"tutorials">
      | CollectionEntry<"challenges">
  ): string => {
    return formatDate(
      entry.data.modifiedDate ||
        (entry.data as { pubDate?: Date; publishedDate?: Date }).pubDate ||
        (entry.data as { publishedDate?: Date }).publishedDate
    );
  };

  // Build sitemap entries
  const sitemapEntries: Array<{
    loc: string;
    lastmod: string;
    changefreq: string;
    priority: number;
  }> = [];

  // Add static pages
  staticPages.forEach((path) => {
    sitemapEntries.push({
      loc: `${siteUrl}${path}`,
      lastmod: new Date().toISOString(),
      changefreq: path === "" ? "daily" : "weekly",
      priority: path === "" ? 1.0 : 0.8,
    });
  });

  // Add blog posts
  blogs.forEach((post: CollectionEntry<"blog">) => {
    sitemapEntries.push({
      loc: `${siteUrl}/blogs/${post.id}`,
      lastmod: getLastMod(post),
      changefreq: "monthly",
      priority: 0.7,
    });
  });

  // Add docs
  docs.forEach((doc: CollectionEntry<"docs">) => {
    sitemapEntries.push({
      loc: `${siteUrl}/docs/${doc.id}`,
      lastmod: getLastMod(doc),
      changefreq: "weekly",
      priority: 0.9,
    });
  });

  // Add courses
  courses.forEach((course: CollectionEntry<"courses">) => {
    sitemapEntries.push({
      loc: `${siteUrl}/courses/${course.id}`,
      lastmod: getLastMod(course),
      changefreq: "monthly",
      priority: 0.8,
    });
  });

  // Add tutorials
  tutorials.forEach((tutorial: CollectionEntry<"tutorials">) => {
    sitemapEntries.push({
      loc: `${siteUrl}/tutorials/${tutorial.id}`,
      lastmod: getLastMod(tutorial),
      changefreq: "monthly",
      priority: 0.8,
    });
  });

  // Add challenges
  challenges.forEach((challenge: CollectionEntry<"challenges">) => {
    sitemapEntries.push({
      loc: `${siteUrl}/challenges/${challenge.id}`,
      lastmod: getLastMod(challenge),
      changefreq: "monthly",
      priority: 0.7,
    });
  });

  // Generate XML
  const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${sitemapEntries
  .map(
    (entry) => `  <url>
    <loc>${escapeXml(entry.loc)}</loc>
    <lastmod>${entry.lastmod}</lastmod>
    <changefreq>${entry.changefreq}</changefreq>
    <priority>${entry.priority}</priority>
  </url>`
  )
  .join("\n")}
</urlset>`;

  return new Response(sitemap, {
    headers: {
      "Content-Type": "application/xml; charset=utf-8",
      "Cache-Control": "public, max-age=3600", // Cache for 1 hour
    },
  });
};

function escapeXml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

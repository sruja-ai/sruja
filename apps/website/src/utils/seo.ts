// apps/website/src/utils/seo.ts
// SEO utility functions for generating meta tags and structured data

import { envConfig } from "@/config/env";

export interface SEOConfig {
  title: string;
  description: string;
  url?: string;
  image?: string;
  type?: "website" | "article" | "profile";
  publishedTime?: string;
  modifiedTime?: string;
  authors?: Array<{ name: string; url?: string }>;
  tags?: string[];
  noindex?: boolean;
  canonical?: string;
}

const defaultSiteUrl = envConfig.siteUrl || "https://sruja.ai";
const defaultSiteName = "Sruja";
const defaultDescription =
  "Developer-friendly language for defining, visualizing, and validating software architecture. Architecture-as-Code for the AI era.";
const defaultImage = `${defaultSiteUrl}/sruja-logo.svg`;

/**
 * Generate full page title with site name
 */
export function getPageTitle(title: string, siteName: string = defaultSiteName): string {
  if (title === siteName) {
    return title;
  }
  return `${title} | ${siteName}`;
}

/**
 * Generate canonical URL
 */
export function getCanonicalUrl(path: string, baseUrl: string = defaultSiteUrl): string {
  const cleanPath = path.startsWith("/") ? path : `/${path}`;
  return `${baseUrl}${cleanPath}`;
}

/**
 * Generate Open Graph image URL
 */
export function getOGImageUrl(image?: string, baseUrl: string = defaultSiteUrl): string {
  if (!image) {
    return defaultImage;
  }
  if (image.startsWith("http://") || image.startsWith("https://")) {
    return image;
  }
  const cleanImage = image.startsWith("/") ? image : `/${image}`;
  return `${baseUrl}${cleanImage}`;
}

/**
 * Truncate description to optimal length for meta tags
 */
export function truncateDescription(description: string, maxLength: number = 160): string {
  if (description.length <= maxLength) {
    return description;
  }
  // Try to truncate at word boundary
  const truncated = description.substring(0, maxLength);
  const lastSpace = truncated.lastIndexOf(" ");
  if (lastSpace > maxLength * 0.8) {
    return truncated.substring(0, lastSpace) + "...";
  }
  return truncated + "...";
}

/**
 * Generate Article JSON-LD structured data
 */
export function generateArticleSchema(config: SEOConfig): object {
  const url = config.url || config.canonical || "";
  const image = getOGImageUrl(config.image);

  return {
    "@context": "https://schema.org",
    "@type": "Article",
    headline: config.title,
    description: config.description,
    image: image,
    url: url,
    datePublished: config.publishedTime,
    dateModified: config.modifiedTime || config.publishedTime,
    author: config.authors?.map((author) => ({
      "@type": "Person",
      name: author.name,
      url: author.url,
    })),
    publisher: {
      "@type": "Organization",
      name: defaultSiteName,
      logo: {
        "@type": "ImageObject",
        url: defaultImage,
      },
    },
    keywords: config.tags?.join(", "),
    mainEntityOfPage: {
      "@type": "WebPage",
      "@id": url,
    },
  };
}

/**
 * Generate Organization JSON-LD structured data
 */
export function generateOrganizationSchema(): object {
  return {
    "@context": "https://schema.org",
    "@type": "Organization",
    name: defaultSiteName,
    url: defaultSiteUrl,
    logo: `${defaultSiteUrl}/sruja-logo.svg`,
    sameAs: ["https://github.com/sruja-ai/sruja"],
    description: defaultDescription,
  };
}

/**
 * Generate Website JSON-LD structured data
 */
export function generateWebsiteSchema(): object {
  return {
    "@context": "https://schema.org",
    "@type": "WebSite",
    name: defaultSiteName,
    url: defaultSiteUrl,
    description: defaultDescription,
    potentialAction: {
      "@type": "SearchAction",
      target: {
        "@type": "EntryPoint",
        urlTemplate: `${defaultSiteUrl}/search?q={search_term_string}`,
      },
      "query-input": "required name=search_term_string",
    },
  };
}

/**
 * Generate BreadcrumbList JSON-LD structured data
 */
export function generateBreadcrumbSchema(items: Array<{ name: string; url: string }>): object {
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((item, index) => ({
      "@type": "ListItem",
      position: index + 1,
      name: item.name,
      item: item.url.startsWith("http") ? item.url : `${defaultSiteUrl}${item.url}`,
    })),
  };
}

/**
 * Generate FAQPage JSON-LD structured data
 */
export function generateFAQSchema(faqs: Array<{ question: string; answer: string }>): object {
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faqs.map((faq) => ({
      "@type": "Question",
      name: faq.question,
      acceptedAnswer: {
        "@type": "Answer",
        text: faq.answer,
      },
    })),
  };
}

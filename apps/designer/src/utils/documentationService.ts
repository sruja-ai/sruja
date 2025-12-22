// apps/designer/src/utils/documentationService.ts
// Documentation mapping service for builder integration

import { getWebsiteUrl } from "./website-url";

/**
 * Maps builder wizard steps to relevant documentation topics
 */
export const DOCUMENTATION_MAP: Record<string, string[]> = {
  goals: ["getting-started", "concepts/requirements", "concepts/overview"],
  context: ["concepts/system", "concepts/person", "concepts/c4-model"],
  containers: ["concepts/container", "concepts/datastore", "concepts/queue"],
  components: ["concepts/component", "concepts/layering"],
  flows: ["concepts/scenario", "concepts/behavior-and-depends-on"],
};

/**
 * Get documentation topics for a builder step
 *
 * @param stepId - Builder step ID (goals, context, containers, components, flows)
 * @returns Array of documentation topic paths
 *
 * @example
 * ```ts
 * const docs = getDocumentationForStep("context");
 * // Returns ["concepts/system", "concepts/person", "concepts/c4-model"]
 * ```
 */
export function getDocumentationForStep(stepId: string): string[] {
  return DOCUMENTATION_MAP[stepId] || [];
}

/**
 * Get full URL for a documentation page
 *
 * @param docPath - Documentation path (e.g., "concepts/system")
 * @param baseUrl - Optional base URL for documentation (default: environment-specific website URL)
 * @returns Full documentation URL
 *
 * @example
 * ```ts
 * const url = getDocumentationUrl("concepts/system");
 * // Returns environment-specific URL, e.g., "https://sruja.ai/docs/concepts/system" or "https://staging.sruja.ai/docs/concepts/system"
 * ```
 */
export function getDocumentationUrl(docPath: string, baseUrl?: string): string {
  // Use provided baseUrl or get environment-specific website URL
  const docsBaseUrl = baseUrl || getWebsiteUrl("/docs");
  // Remove trailing slash if present and ensure docPath doesn't start with /
  const cleanPath = docPath.startsWith("/") ? docPath.slice(1) : docPath;
  return `${docsBaseUrl}/${cleanPath}`;
}

/**
 * Get primary documentation URL for a step (first recommended doc)
 *
 * @param stepId - Builder step ID
 * @returns Primary documentation URL or null
 */
export function getPrimaryDocumentationUrl(stepId: string): string | null {
  const docs = getDocumentationForStep(stepId);
  return docs.length > 0 ? getDocumentationUrl(docs[0]) : null;
}

/**
 * Documentation topic metadata
 */
export interface DocumentationTopic {
  path: string;
  title: string;
  description?: string;
  /** Short excerpt/content to display inline */
  excerpt?: string;
  /** Full markdown content (optional, for expanded view) */
  content?: string;
}

/**
 * Short excerpts for inline documentation display
 */
const DOCUMENTATION_EXCERPTS: Record<string, string> = {
  "getting-started":
    "Sruja allows you to define your software architecture as code. No more dragging boxes around. No more outdated PNGs on a wiki. **You write code, Sruja draws the maps.**",
  "concepts/requirements":
    "Use `requirement` to capture functional, performance, security, and constraint requirements. Requirements are declared at the architecture root only.",
  "concepts/overview":
    "Use `overview` to provide a concise system description shown in docs/exports. Keep summary short and practical; avoid marketing language.",
  "concepts/system":
    "A **System** represents a software system, which is the highest level of abstraction in the C4 model. A system delivers value to its users, whether they are human or other systems.",
  "concepts/person":
    'A **Person** represents a human user of your software system (e.g., "Customer", "Admin", "Employee").',
  "concepts/c4-model":
    "Sruja follows the C4 model for visualizing software architecture. The C4 model uses four levels of abstraction: System, Container, Component, and Code.",
  "concepts/container":
    'A **Container** represents an application or a data store. It is something that needs to be running in order for the overall software system to work. In C4, "Container" does *not* mean a Docker container—it means a deployable unit.',
  "concepts/datastore":
    "A **Data Store** represents a database, file system, or other storage mechanism. Data stores persist data and are accessed by containers.",
  "concepts/queue":
    "A **Queue** represents an asynchronous message queue or event stream. Queues enable decoupled communication between containers.",
  "concepts/component":
    "A **Component** is a grouping of related functionality encapsulated behind a well-defined interface. Components reside inside Containers.",
  "concepts/layering":
    "Layering helps organize components into logical layers (e.g., presentation, business logic, data access). This improves maintainability and separation of concerns.",
  "concepts/scenario":
    "Scenarios describe behavioral flows as ordered steps. They focus on interactions rather than data pipelines. Use scenarios to model user stories and use cases.",
  "concepts/behavior-and-depends-on":
    "Use `behavior` and `depends-on` to model component interactions and dependencies. This helps document how components collaborate and what they rely on.",
};

/**
 * Get documentation topics with metadata for a step
 *
 * @param stepId - Builder step ID
 * @returns Array of documentation topics with metadata
 */
export function getDocumentationTopics(stepId: string): DocumentationTopic[] {
  const paths = getDocumentationForStep(stepId);

  // Topic titles mapping
  const titles: Record<string, string> = {
    "getting-started": "Getting Started",
    "concepts/requirements": "Requirements",
    "concepts/overview": "Overview",
    "concepts/system": "Systems",
    "concepts/person": "Persons/Actors",
    "concepts/c4-model": "C4 Model",
    "concepts/container": "Containers",
    "concepts/datastore": "Data Stores",
    "concepts/queue": "Queues",
    "concepts/component": "Components",
    "concepts/layering": "Layering",
    "concepts/scenario": "Scenarios",
    "concepts/behavior-and-depends-on": "Behavior & Dependencies",
  };

  return paths.map((path) => ({
    path,
    title: titles[path] || path.split("/").pop() || path,
    description: `Learn more about ${titles[path] || path}`,
    excerpt: DOCUMENTATION_EXCERPTS[path],
  }));
}

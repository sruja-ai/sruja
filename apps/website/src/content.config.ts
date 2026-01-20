// apps/website/src/content.config.ts
import { defineCollection } from "astro:content";
import { z } from "astro/zod";
import { glob } from "astro/loaders";

const docs = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs" }),
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
    description: z.string().optional(),
  }),
});

const blog = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/blog" }),
  schema: z.object({
    title: z.string(),
    authors: z
      .array(
        z.object({
          name: z.string(),
          title: z.string().optional(),
          url: z.string().optional(),
          image_url: z.string().optional(),
        })
      )
      .optional(),
    tags: z.array(z.string()).optional(),
    description: z.string().optional(),
    summary: z.string().optional(),
    image: z.string().optional(),
    pubDate: z.coerce.date().optional(),
    modifiedDate: z.coerce.date().optional(),
  }),
});

const courses = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/courses" }),
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    description: z.string().optional(),
    difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
  }),
});

const tutorials = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/tutorials" }),
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    tags: z.array(z.string()).optional(),
    aliases: z.array(z.string()).optional(),
    description: z.string().optional(),
    difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
  }),
});

const quizzes = defineCollection({
  loader: glob({ pattern: "**/*.json", base: "./src/content/quizzes" }),
  schema: z.object({
    title: z.string(),
    slug: z.string(),
    summary: z.string().optional(),
    questions: z.array(
      z.object({
        id: z.string(),
        prompt: z.string(),
        options: z.array(z.object({ id: z.string(), label: z.string() })),
        answer: z.string(),
        explanation: z.string().optional(),
      })
    ),
  }),
});

const challenges = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/challenges" }),
  schema: z.object({
    title: z.string(),
    summary: z.string().optional(),
    difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
    initialDsl: z.string(),
    checks: z.array(
      z.object({
        type: z.enum(["relationExists", "noErrors", "elementExists"]),
        source: z.string().optional(),
        target: z.string().optional(),
        label: z.string().optional(),
        name: z.string().optional(),
        message: z.string().optional(),
      })
    ),
    hints: z.array(z.string()).optional(),
    solution: z.string().optional(),
  }),
});

const investors = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/investors" }),
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    description: z.string().optional(),
  }),
});

const templates = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/templates" }),
  schema: z.object({
    title: z.string(),
    summary: z.string().optional(),
    estimated_time: z.string().optional(),
    difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    tags: z.array(z.string()).optional(),
    prerequisites: z.array(z.string()).optional(),
    learning_objectives: z.array(z.string()).optional(),
  }),
});

export const collections = {
  docs,
  blog,
  courses,
  tutorials,
  quizzes,
  challenges,
  investors,
  templates,
};

// Note: These collections are defined but may be empty initially.
// Content will be added as the site grows.

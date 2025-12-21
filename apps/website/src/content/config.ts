// apps/website/src/content/config.ts
import { defineCollection, z } from 'astro:content';

const docs = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
    description: z.string().optional(),
  }),
});

const blog = defineCollection({
  type: 'content',
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
    pubDate: z.coerce.date().optional(),
  }),
});

const courses = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    description: z.string().optional(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
  }),
});

const tutorials = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    tags: z.array(z.string()).optional(),
    aliases: z.array(z.string()).optional(),
    description: z.string().optional(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
  }),
});

const quizzes = defineCollection({
  type: 'data',
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
  type: 'content',
  schema: z.object({
    title: z.string(),
    summary: z.string().optional(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']).optional(),
    topic: z.string().optional(),
    estimatedTime: z.string().optional(),
    initialDsl: z.string(),
    checks: z.array(
      z.object({
        type: z.enum(['relationExists', 'noErrors', 'elementExists']),
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
  type: 'content',
  schema: z.object({
    title: z.string(),
    weight: z.number().optional(),
    summary: z.string().optional(),
    description: z.string().optional(),
  }),
});

const templates = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    summary: z.string().optional(),
    estimated_time: z.string().optional(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']).optional(),
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

// Type definitions for Sruja Learn App

export interface NavLink {
  href: string;
  label: string;
}

export type Section = 'playground' | 'about' | 'learn' | 'community' | 'home' | 'courses' | 'docs' | 'tutorials' | 'blogs';

export interface CourseState {
  visited: string[];
  quizResults: Record<string, unknown>;
  lastVisited: string | null;
}

export interface CompileResult {
  error?: string;
  svg?: string;
  image?: string;
  png?: string;
  jpg?: string;
  jpeg?: string;
  html?: string;
}

export interface PlaygroundExample {
  name: string;
  code: string;
}

// Global types live in global.d.ts

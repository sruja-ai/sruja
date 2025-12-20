// apps/designer/src/types/layout.ts
// Type definitions for layout-related types

import type { C4Level } from "../types";

/**
 * Type guard to check if a string is a valid C4Level
 */
export function isC4Level(level: string | undefined): level is C4Level {
  if (!level) return false;
  return ["L1", "L2", "L3", "L4"].includes(level);
}

/**
 * Safely convert string to C4Level, with fallback
 */
export function toC4Level(level: string | undefined, fallback: C4Level = "L1"): C4Level {
  if (isC4Level(level)) {
    return level;
  }
  return fallback;
}

/**
 * Layout direction options
 */
export type LayoutDirection = "DOWN" | "RIGHT" | "UP" | "LEFT";

// Layout engine types removed - LikeC4 handles all layout internally

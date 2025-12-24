/**
 * Layout Refinement System
 *
 * This module implements iterative refinement of layout constraints
 * based on quality metrics feedback.
 *
 * Architecture:
 * - Takes quality metrics as input
 * - Adjusts layout parameters (spacing, constraints, etc.)
 * - Returns refined configuration for next iteration
 */

import type { LayoutQuality } from "./qualityMetrics";
import type { LayoutOptions } from "./types";

/**
 * Refined layout configuration
 */
export interface RefinedLayoutConfig extends LayoutOptions {
  /** Iteration number */
  iteration: number;
  /** Quality score from previous iteration */
  previousScore: number;
  /** Maximum iterations to attempt */
  maxIterations: number;
  /** Node count for adaptive spacing */
  nodeCount?: number;
}

/**
 * Default refinement configuration
 */
const DEFAULT_REFINEMENT: RefinedLayoutConfig = {
  rankdir: "TB",
  nodesep: 75, // Increased base spacing (was 60, increased to match Go backend improvements)
  ranksep: 105, // Increased base spacing (was 90, increased to match Go backend improvements)
  iteration: 0,
  previousScore: 0,
  maxIterations: 3,
};

/**
 * Calculate adaptive spacing based on node count (logarithmic scaling)
 * FAANG-pattern: Prevent overlaps in dense diagrams, optimize space in sparse ones
 */
export function calculateAdaptiveSpacing(nodeCount: number): { nodesep: number; ranksep: number } {
  // Logarithmic scaling: grows slowly for large diagrams
  const scaleFactor = 1 + 0.15 * Math.log10(Math.max(1, nodeCount));

  // Base spacing with logarithmic growth
  const nodesep = Math.round(DEFAULT_REFINEMENT.nodesep * scaleFactor);
  const ranksep = Math.round(DEFAULT_REFINEMENT.ranksep * scaleFactor);

  // Clamp to reasonable bounds - increased max values to prevent overlaps
  return {
    nodesep: Math.max(60, Math.min(180, nodesep)), // Increased max from 150 to 180
    ranksep: Math.max(85, Math.min(220, ranksep)), // Increased max from 180 to 220, min from 70 to 85
  };
}

/**
 * Refine layout configuration based on quality metrics
 */
export function refineLayoutConfig(
  currentConfig: LayoutOptions,
  quality: LayoutQuality,
  previousConfig?: RefinedLayoutConfig
): RefinedLayoutConfig {
  const config: RefinedLayoutConfig = {
    ...currentConfig,
    iteration: previousConfig ? previousConfig.iteration + 1 : 1,
    previousScore: quality.score,
    maxIterations: previousConfig?.maxIterations ?? DEFAULT_REFINEMENT.maxIterations,
  };

  // If node overlaps detected, increase spacing more aggressively
  if (quality.nodeOverlaps > 0) {
    config.nodesep = Math.min(
      config.nodesep * 1.3, // More aggressive (was 1.2)
      config.nodesep + 40 // Larger cap (was 30)
    );
    config.ranksep = Math.min(
      config.ranksep * 1.3, // More aggressive (was 1.2)
      config.ranksep + 50 // Larger cap (was 40)
    );
  }

  // If any edge crossings, be more aggressive about fixing them
  // FAANG pattern: Try direction change first, then spacing increases
  if (quality.edgeCrossings > 0 && config.iteration === 1) {
    // Try switching rank direction on first iteration if crossings exist
    config.rankdir = config.rankdir === "TB" ? "LR" : "TB";
  } else if (quality.edgeCrossings > 0) {
    // If still crossings after direction change, increase spacing more aggressively
    const crossingFactor = 1 + Math.min(quality.edgeCrossings * 0.1, 0.4);
    config.nodesep = Math.min(config.nodesep * crossingFactor, config.nodesep + 50);
    config.ranksep = Math.min(config.ranksep * crossingFactor, config.ranksep + 60);
  }

  // If rank alignment is poor, increase rank separation more aggressively
  // LikeC4 requires near-perfect alignment (95%+)
  if (quality.rankAlignment < 0.95) {
    const alignmentGap = 0.95 - quality.rankAlignment;
    const adjustment = 1.0 + alignmentGap * 0.5; // Scale adjustment by gap
    config.ranksep = Math.min(
      config.ranksep * adjustment,
      config.ranksep + 40 // Larger cap
    );
  }

  // If label overlaps, increase spacing to give labels more room
  if (quality.labelOverlaps > 0) {
    // Increase both separations to open up the graph
    config.nodesep = Math.min(config.nodesep * 1.25, config.nodesep + 40);
    config.ranksep = Math.min(config.ranksep * 1.25, config.ranksep + 40);
  }

  // If score is improving, continue with smaller adjustments
  if (previousConfig && quality.score > previousConfig.previousScore) {
    // Score improved - make smaller adjustments
    config.nodesep = Math.min(config.nodesep, previousConfig.nodesep * 1.1);
    config.ranksep = Math.min(config.ranksep, previousConfig.ranksep * 1.1);
  }

  return config;
}

/**
 * Check if refinement should continue
 * More strict thresholds for LikeC4-level aesthetics
 */
export function shouldContinueRefinement(
  config: RefinedLayoutConfig,
  quality: LayoutQuality
): boolean {
  // Stop if max iterations reached
  if (config.iteration >= config.maxIterations) {
    return false;
  }

  // Stop if quality is acceptable (LikeC4-level: score >= 0.90, perfect alignment, no issues)
  if (
    quality.score >= 0.9 &&
    quality.nodeOverlaps === 0 &&
    quality.labelOverlaps === 0 &&
    quality.edgeCrossings === 0 &&
    quality.rankAlignment >= 0.95
  ) {
    return false;
  }

  // Stop if quality is degrading (no improvement after 2 iterations)
  if (config.iteration >= 2 && quality.score <= config.previousScore) {
    return false;
  }

  return true;
}

/**
 * Get initial layout configuration
 */
export function getInitialLayoutConfig(options?: Partial<LayoutOptions>): RefinedLayoutConfig {
  return {
    ...DEFAULT_REFINEMENT,
    ...options,
    iteration: 0,
    previousScore: 0,
  };
}

/**
 * Validation Phase
 * Final validation and quality assessment with comprehensive checks
 */

import type { LayoutPhase, LayoutContext, LayoutNode } from "../core/types";
import { MIN_PARENT_PADDING, LABEL_HEIGHT_BUFFER } from "../constants";
import { classifyC4Node } from "../algorithms/c4-classifier";

import { applyOverlapRemoval } from "./optimization";

export function createValidationPhase(): LayoutPhase {
  return {
    name: "validation",
    description: "Validate final layout quality",
    dependencies: ["optimization"],
    execute: (context: LayoutContext): LayoutContext => {
      // CRITICAL: Run final overlap removal before validation
      // This ensures no overlaps remain after all optimizations
      let validatedContext = applyOverlapRemoval(context);

      // Post-layout validation checks
      const warnings: string[] = [];

      // Check containment violations
      const containmentViolations = validateContainment(validatedContext);
      if (containmentViolations.length > 0) {
        warnings.push(`Found ${containmentViolations.length} containment violations`);
        if (validatedContext.options.debug?.enabled) {
          containmentViolations.slice(0, 3).forEach((v) => {
            console.warn(`[Validation] Containment: ${v}`);
          });
        }
      }

      // Check overlaps
      const overlaps = validateOverlaps(validatedContext);
      if (overlaps.length > 0) {
        warnings.push(`Found ${overlaps.length} node overlaps`);
        if (validatedContext.options.debug?.enabled) {
          overlaps.slice(0, 3).forEach((o) => {
            console.warn(`[Validation] Overlap: ${o}`);
          });
        }
      }

      // Check edge crossings
      const edgeCrossings = validateEdgeCrossings(validatedContext);
      if (edgeCrossings > 20) {
        warnings.push(`High edge crossings: ${edgeCrossings} (target <20)`);
      }

      // Check semantic positioning
      const semanticCompliance = validateSemanticPositioning(validatedContext);
      if (semanticCompliance < 80) {
        warnings.push(`Semantic positioning: ${semanticCompliance.toFixed(1)}% (target ≥80%)`);
      }

      // Store warnings in debug info
      if (warnings.length > 0 && validatedContext.options.debug) {
        if (!validatedContext.debug) {
          validatedContext.debug = {
            phases: [],
            warnings: [],
            errors: [],
            metrics: new Map(),
          };
        }
        validatedContext.debug.warnings = warnings;
      }

      if (validatedContext.qualityEvaluator && validatedContext.metricsCalculator) {
        const updatedContext = {
          ...validatedContext,
          qualityScore: validatedContext.qualityEvaluator(validatedContext),
          metrics: validatedContext.metricsCalculator(validatedContext),
        };
        return updatedContext;
      }
      return validatedContext;
    },
  };
}

function validateContainment(context: LayoutContext): string[] {
  const violations: string[] = [];
  // CRITICAL: Sort nodes for deterministic processing order
  const nodes = Array.from(context.nodes.values())
    .filter((n) => n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  for (const node of nodes) {
    if (!node.parent) continue;
    const parent = context.nodes.get(node.parent.id);
    if (!parent) continue;

    const effectivePadding = Math.max(MIN_PARENT_PADDING, context.options.spacing.parentChildPadding);
    const minX = parent.bbox.x + effectivePadding;
    const minY = parent.bbox.y + effectivePadding + LABEL_HEIGHT_BUFFER;
    const maxX = parent.bbox.x + parent.bbox.width - effectivePadding;
    const maxY = parent.bbox.y + parent.bbox.height - effectivePadding;

    const childRight = node.bbox.x + node.bbox.width;
    const childBottom = node.bbox.y + node.bbox.height;

    if (node.bbox.x < minX || node.bbox.y < minY || childRight > maxX || childBottom > maxY) {
      violations.push(`${node.id} violates containment in ${parent.id}`);
    }
  }

  return violations;
}

function validateOverlaps(context: LayoutContext): string[] {
  const overlaps: string[] = [];
  // CRITICAL: Sort nodes for deterministic processing order
  const nodes = Array.from(context.nodes.values())
    .filter((n) => n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const a = nodes[i];
      const b = nodes[j];

      // Skip parent-child (containment handles that)
      if (a.parent?.id === b.id || b.parent?.id === a.id) continue;

      if (nodesOverlap(a, b)) {
        overlaps.push(`${a.id} overlaps ${b.id}`);
      }
    }
  }

  return overlaps;
}

function validateEdgeCrossings(context: LayoutContext): number {
  const edges = Array.from(context.edges.values());
  let crossings = 0;

  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      if (edgesCross(edges[i], edges[j])) {
        crossings++;
      }
    }
  }

  return crossings;
}

function validateSemanticPositioning(context: LayoutContext): number {
  // CRITICAL: Sort nodes for deterministic processing order
  const nodes = Array.from(context.nodes.values())
    .filter((n) => !n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (nodes.length < 2) return 100;

  const classified = nodes.map((node) => ({
    node,
    classification: classifyC4Node({ id: node.id, original: node.original as any }),
    centerY: node.bbox.y + node.bbox.height / 2,
  }));

  const actors = classified.filter((c) => c.classification.tier === "actor");
  const dataNodes = classified.filter((c) => c.classification.tier === "data");
  const presentationNodes = classified.filter((c) => c.classification.tier === "presentation");
  const logicNodes = classified.filter((c) => c.classification.tier === "logic");

  let correctPositions = 0;
  let totalChecks = 0;

  const avgActorY = actors.length > 0 ? actors.reduce((s, a) => s + a.centerY, 0) / actors.length : Infinity;
  const avgDataY = dataNodes.length > 0 ? dataNodes.reduce((s, d) => s + d.centerY, 0) / dataNodes.length : -Infinity;
  const avgPresentationY = presentationNodes.length > 0 ? presentationNodes.reduce((s, p) => s + p.centerY, 0) / presentationNodes.length : Infinity;
  const avgLogicY = logicNodes.length > 0 ? logicNodes.reduce((s, l) => s + l.centerY, 0) / logicNodes.length : 0;

  if (actors.length > 0 && (dataNodes.length > 0 || logicNodes.length > 0)) {
    totalChecks++;
    const otherAvgY = dataNodes.length > 0 ? avgDataY : avgLogicY;
    if (avgActorY < otherAvgY) correctPositions++;
  }

  if (dataNodes.length > 0 && (actors.length > 0 || logicNodes.length > 0)) {
    totalChecks++;
    const otherAvgY = actors.length > 0 ? avgActorY : avgLogicY;
    if (avgDataY > otherAvgY) correctPositions++;
  }

  if (presentationNodes.length > 0 && logicNodes.length > 0) {
    totalChecks++;
    if (avgPresentationY < avgLogicY) correctPositions++;
  }

  if (presentationNodes.length > 0 && dataNodes.length > 0) {
    totalChecks++;
    if (avgPresentationY < avgDataY) correctPositions++;
  }

  if (logicNodes.length > 0 && dataNodes.length > 0) {
    totalChecks++;
    if (avgLogicY < avgDataY) correctPositions++;
  }

  return totalChecks > 0 ? (correctPositions / totalChecks) * 100 : 100;
}

function nodesOverlap(node1: LayoutNode, node2: LayoutNode): boolean {
  return !(
    node1.bbox.x + node1.bbox.width <= node2.bbox.x ||
    node2.bbox.x + node2.bbox.width <= node1.bbox.x ||
    node1.bbox.y + node1.bbox.height <= node2.bbox.y ||
    node2.bbox.y + node2.bbox.height <= node1.bbox.y
  );
}

function edgesCross(edge1: any, edge2: any): boolean {
  if (
    edge1.source.id === edge2.source.id ||
    edge1.source.id === edge2.target.id ||
    edge1.target.id === edge2.source.id ||
    edge1.target.id === edge2.target.id
  ) {
    return false; // Connected edges don't count as crossings
  }

  for (let i = 0; i < edge1.points.length - 1; i++) {
    for (let j = 0; j < edge2.points.length - 1; j++) {
      if (
        lineSegmentsIntersect(
          edge1.points[i],
          edge1.points[i + 1],
          edge2.points[j],
          edge2.points[j + 1]
        )
      ) {
        return true;
      }
    }
  }
  return false;
}

function lineSegmentsIntersect(p1: { x: number; y: number }, p2: { x: number; y: number }, p3: { x: number; y: number }, p4: { x: number; y: number }): boolean {
  const det = (p2.x - p1.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(det) < 1e-10) return false;

  const lambda = ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
  const gamma = ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;

  return 0 < lambda && lambda < 1 && 0 < gamma && gamma < 1;
}

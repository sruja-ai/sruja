// apps/designer/src/utils/antiPatternDetector.ts
import type { SrujaModelDump } from "@sruja/shared";

export interface AntiPattern {
  type: "cycle" | "god-object" | "god-component" | "circular-dependency";
  severity: "critical" | "warning" | "info";
  description: string;
  affectedElements: string[]; // Element IDs
  suggestion?: string;
  explanation?: string; // Detailed explanation of why this is problematic
  refactoring?: string; // Specific refactoring approach
}

/**
 * Detect anti-patterns in the architecture model.
 */
export function detectAntiPatterns(model: SrujaModelDump | null): AntiPattern[] {
  if (!model) return [];

  const antiPatterns: AntiPattern[] = [];

  // Detect cycles using relations
  const cycles = detectCycles(model);
  antiPatterns.push(...cycles);

  // Detect God objects (high fan-in/fan-out)
  const godObjects = detectGodObjects(model);
  antiPatterns.push(...godObjects);

  return antiPatterns;
}

/**
 * Detect cyclic dependencies in the architecture.
 * Uses a simplified cycle detection algorithm.
 */
function detectCycles(model: SrujaModelDump): AntiPattern[] {
  const antiPatterns: AntiPattern[] = [];
  const relations = model.relations || [];
  const elements = model.elements || {};

  // Build adjacency list
  const adjacencyList = new Map<string, string[]>();
  for (const rel of relations) {
    // Robust extraction of source and target from FqnRef (string or {model: string})
    const r = rel as Record<string, unknown>;
    const source = (
      r.source && typeof r.source === "object" && "model" in r.source
        ? (r.source as { model: string }).model
        : r.source
    ) as string;
    const target = (
      r.target && typeof r.target === "object" && "model" in r.target
        ? (r.target as { model: string }).model
        : r.target
    ) as string;

    if (source && target) {
      if (!adjacencyList.has(source)) {
        adjacencyList.set(source, []);
      }
      adjacencyList.get(source)!.push(target);
    }
  }

  // DFS to detect cycles
  const visited = new Set<string>();
  const recStack = new Set<string>();
  const cyclePaths: string[][] = [];

  function dfs(node: string, path: string[]): void {
    visited.add(node);
    recStack.add(node);
    path.push(node);

    const neighbors = adjacencyList.get(node) || [];
    for (const neighbor of neighbors) {
      if (!visited.has(neighbor)) {
        dfs(neighbor, [...path]);
      } else if (recStack.has(neighbor)) {
        // Found a cycle
        const cycleStart = path.indexOf(neighbor);
        if (cycleStart !== -1) {
          const cycle = path.slice(cycleStart).concat(neighbor);
          cyclePaths.push(cycle);
        }
      }
    }

    recStack.delete(node);
  }

  // Check all nodes
  for (const nodeId of Object.keys(elements)) {
    if (!visited.has(nodeId)) {
      dfs(nodeId, []);
    }
  }

  // Convert cycles to anti-patterns
  for (const cycle of cyclePaths) {
    const cycleStr = cycle.join(" → ");
    antiPatterns.push({
      type: "cycle",
      severity: cycle.length > 3 ? "critical" : "warning",
      description: `Circular dependency detected: ${cycleStr}`,
      affectedElements: cycle,
      explanation: `Components ${cycleStr} form a circular dependency where each component depends on the next, creating tight coupling and making the system difficult to maintain, test, and evolve. Changes to any component in the cycle can ripple through the entire cycle.`,
      suggestion:
        "Break the cycle by introducing an intermediate element or using event-driven patterns.",
      refactoring: `1. Extract shared functionality into a separate service/component\n2. Use dependency inversion - depend on abstractions, not concretions\n3. Introduce an event bus or message queue for communication\n4. Apply the Dependency Inversion Principle to reverse one or more dependencies`,
    });
  }

  return antiPatterns;
}

/**
 * Detect God objects (components with high fan-in/fan-out).
 */
function detectGodObjects(model: SrujaModelDump): AntiPattern[] {
  const antiPatterns: AntiPattern[] = [];
  const relations = model.relations || [];
  const elements = model.elements || {};

  // Count fan-in and fan-out for each element
  const fanIn = new Map<string, number>();
  const fanOut = new Map<string, number>();

  for (const rel of relations) {
    // Simplified extraction for count purposes
    const r = rel as Record<string, unknown>;
    const source = (
      r.source && typeof r.source === "object" && "model" in r.source
        ? (r.source as { model: string }).model
        : r.source
    ) as string;
    const target = (
      r.target && typeof r.target === "object" && "model" in r.target
        ? (r.target as { model: string }).model
        : r.target
    ) as string;

    if (source && target) {
      fanOut.set(source, (fanOut.get(source) || 0) + 1);
      fanIn.set(target, (fanIn.get(target) || 0) + 1);
    }
  }

  // Threshold for God object detection
  const FAN_THRESHOLD = 5; // Elements with >5 connections

  for (const [elementId, outCount] of fanOut.entries()) {
    const inCount = fanIn.get(elementId) || 0;
    const totalConnections = inCount + outCount;

    if (totalConnections > FAN_THRESHOLD) {
      const element = elements[elementId];
      const elementType = (element as { kind?: string })?.kind || "element";

      const isCritical = totalConnections > 10;
      const patternType = elementType === "component" ? "god-component" : "god-object";

      antiPatterns.push({
        type: patternType,
        severity: isCritical ? "critical" : "warning",
        description: `${elementId} has ${totalConnections} connections (${inCount} incoming, ${outCount} outgoing). This indicates a ${patternType.replace("-", " ")}.`,
        affectedElements: [elementId],
        explanation: `${elementId} is highly coupled with ${totalConnections} dependencies, making it a bottleneck and single point of failure. ${isCritical ? "This is a critical issue" : "This violates the Single Responsibility Principle"} - the component is doing too much and has too many responsibilities, making it difficult to maintain, test, and evolve.`,
        suggestion: "Consider splitting this element into smaller, more focused components.",
        refactoring: `1. Identify distinct responsibilities within ${elementId}\n2. Extract each responsibility into a separate, focused component\n3. Use composition to combine these smaller components if needed\n4. Apply the Single Responsibility Principle - each component should have one reason to change\n5. Consider using the Facade pattern if the complexity is justified`,
      });
    }
  }

  return antiPatterns;
}

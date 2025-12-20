/**
 * Example usage of the new modular layout engine
 */

import { createLayoutEngine, type LayoutOptions } from "./core/engine";
import type { C4Graph, C4Node, C4Relationship, C4ViewState } from "./core/types";

// Example C4 graph
const exampleNodes: C4Node[] = [
  {
    id: "user",
    type: "Person",
    level: "L1",
    label: "User",
    description: "System user",
  },
  {
    id: "web-app",
    type: "System",
    level: "L1",
    label: "Web Application",
    description: "Main web application",
    technology: "React",
  },
  {
    id: "api",
    type: "System",
    level: "L1",
    label: "API Service",
    description: "Backend API service",
    technology: "Node.js",
  },
  {
    id: "database",
    type: "System",
    level: "L1",
    label: "Database",
    description: "Data storage",
    technology: "PostgreSQL",
  },
];

const exampleRelationships: C4Relationship[] = [
  {
    id: "r1",
    from: "user",
    to: "web-app",
    label: "uses",
    type: "Dependency",
  },
  {
    id: "r2",
    from: "web-app",
    to: "api",
    label: "calls",
    type: "Communication",
  },
  {
    id: "r3",
    from: "api",
    to: "database",
    label: "reads/writes",
    type: "DataFlow",
  },
];

const exampleGraph: C4Graph = {
  nodes: new Map(exampleNodes.map((n) => [n.id, n])),
  relationships: exampleRelationships,
};

// Test the new layout engine
async function testNewLayoutEngine() {
  const engine = createLayoutEngine();

  const options: Partial<LayoutOptions> = {
    strategy: "L1-context",
    quality: {
      targetGrade: "B",
      strictMode: false,
      validateConstraints: true,
      enforceMetrics: false,
      earlyExit: true,
    },
    debug: {
      enabled: true,
      saveIntermediates: false,
      showMetrics: true,
      showHeatmap: false,
      showPortUsage: false,
      verboseLogging: false,
    },
  };

  const view: C4ViewState = {
    level: "L1",
    expandedNodes: new Set<string>(),
    hiddenNodes: new Set<string>(),
    gridSize: 10,
    snapToGrid: false,
  };

  try {
    console.log("Testing new layout engine...");
    const result = await engine.layout(exampleGraph, view, options);

    console.log("Layout completed successfully!");
    console.log(`Quality Grade: ${result.quality.grade} (Score: ${result.quality.score})`);
    console.log(`Processing Time: ${result.metrics.processingTime}ms`);
    console.log(`Total Nodes: ${result.metrics.totalNodes}`);
    console.log(`Total Edges: ${result.metrics.totalEdges}`);
    console.log(`Edge Crossings: ${result.metrics.edgeCrossings}`);
    console.log(`Aspect Ratio: ${result.metrics.aspectRatio.toFixed(2)}`);

    if (result.debug.warnings.length > 0) {
      console.log("Warnings:", result.debug.warnings);
    }

    return result;
  } catch (error) {
    console.error("Layout failed:", error);
    throw error;
  }
}

// Export for testing
export { testNewLayoutEngine, exampleGraph, exampleNodes, exampleRelationships };

// Run test if this file is executed directly
if (require.main === module) {
  testNewLayoutEngine();
}

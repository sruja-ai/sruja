// Metrics collection utility for layout optimization
import * as fs from "fs";
import type { Page } from "@playwright/test";
import type { DiagramQualityMetrics } from "../../src/utils/diagramQuality";
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from "../../src/utils/diagramQuality";
import { selectLayoutConfig } from "../../src/utils/layoutRules";

export interface LayoutMetrics {
  // Example info
  exampleName: string;
  category: string;
  timestamp: string;

  // Quality scores
  weightedScore: number;
  overallScore: number;
  grade: "A" | "B" | "C" | "D" | "F";

  // Specific metrics
  overlappingNodes: number;
  edgeCrossings: number;
  edgesOverNodes: number;
  edgeBends: number;
  spacingViolations: number;
  parentChildViolations: number;

  // Layout characteristics
  nodeCount: number;
  edgeCount: number;
  hasHierarchy: boolean;
  currentLevel: string;
  selectedEngine: "sruja" | "c4level";
  selectedDirection: string;

  // Performance
  layoutTime: number; // milliseconds
  renderTime: number;

  // Visual metrics
  viewportUtilization: number;
  aspectRatio: number;
  diagramBounds: { x: number; y: number; width: number; height: number };

  // Full quality metrics
  qualityMetrics: DiagramQualityMetrics;
}

export interface MetricsCollectionOptions {
  includePerformance?: boolean;
  includeFullMetrics?: boolean;
}

/**
 * Get nodes from React Flow
 */
async function getReactFlowNodes(page: Page): Promise<any[]> {
  return await page.evaluate(() => {
    // Prefer exposed state for accuracy
    const exposed = (window as any).__CYBER_GRAPH__;
    if (exposed?.nodes) {
      return exposed.nodes;
    }

    // Fallback to DOM (legacy) - shouldn't happen in test harness
    const nodes: any[] = [];
    return nodes;
  });
}

async function getReactFlowEdges(page: Page): Promise<any[]> {
  return await page.evaluate(() => {
    // Prefer exposed state for accuracy
    const exposed = (window as any).__CYBER_GRAPH__;
    if (exposed?.edges) {
      return exposed.edges;
    }
    return [];
  });
}

async function waitForLayoutStable(
  page: Page,
  exampleName: string,
  timeout = 30000
): Promise<void> {
  // Check for explicit error message in the app first
  // Our App component renders <div>Error: {message}</div> on failure
  const errorElement = await page.$("text=/^Error:/");
  if (errorElement) {
    const text = await errorElement.innerText();
    throw new Error(`App encountered error: ${text}`);
  }

  try {
    // Wait for loading overlay to disappear
    await page.waitForSelector(".loading-overlay", { state: "hidden", timeout: timeout / 2 });
  } catch (e) {
    // Double check for error if loading timeout
    const errorElement = await page.$("text=/^Error:/");
    if (errorElement) {
      const text = await errorElement.innerText();
      throw new Error(`App encountered error: ${text}`);
    }
    throw new Error(`Timed out waiting for loading overlay to disappear.`);
  }

  try {
    // Wait for React Flow to be ready
    await page.waitForSelector(".react-flow", { timeout: timeout / 2 });
  } catch (e) {
    // Log body content for debugging
    const body = await page.textContent("body");
    console.log(`[DEBUG] Page body content: ${body}`);

    await page.screenshot({ path: `error-${exampleName.replace(/[^a-z0-9]/gi, "_")}.png` });

    // Double check for error
    const errorElement = await page.$("text=/^Error:/");
    if (errorElement) {
      const text = await errorElement.innerText();
      throw new Error(`App encountered error: ${text}`);
    }
    throw new Error(`Timed out waiting for React Flow to appear.`);
  }

  // Additional wait for layout to stabilize
  await page.waitForTimeout(500);
}

/**
 * Collect comprehensive layout metrics for an example
 */
export async function collectLayoutMetrics(
  page: Page,
  exampleName: string,
  category: string,
  options: MetricsCollectionOptions = {}
): Promise<LayoutMetrics> {
  const { includePerformance = true, includeFullMetrics = true } = options;

  const startTime = Date.now();

  // Wait for layout to stabilize
  await waitForLayoutStable(page, exampleName);

  const renderTime = Date.now() - startTime;

  // Get nodes and edges
  const nodes = await getReactFlowNodes(page);
  const edges = await getReactFlowEdges(page);

  if (nodes.length === 0) {
    throw new Error(`No nodes found for example: ${exampleName}`);
  }

  // Get viewport size
  const viewportSize = (await page.viewportSize()) || { width: 1920, height: 1080 };

  // Calculate quality metrics
  const qualityMetrics = calculateDiagramQuality(
    nodes,
    edges,
    viewportSize,
    DEFAULT_QUALITY_WEIGHTS
  );

  // Determine selected layout config
  const config = selectLayoutConfig(nodes, edges, "L2", undefined, undefined, new Set());

  // Calculate diagram bounds
  const diagramBounds = nodes.reduce(
    (bounds, node) => {
      const right = node.position.x + node.width;
      const bottom = node.position.y + node.height;
      return {
        x: Math.min(bounds.x, node.position.x),
        y: Math.min(bounds.y, node.position.y),
        width: Math.max(bounds.width, right - bounds.x),
        height: Math.max(bounds.height, bottom - bounds.y),
      };
    },
    { x: Infinity, y: Infinity, width: 0, height: 0 }
  );

  const aspectRatio = diagramBounds.width / diagramBounds.height || 1;
  const viewportUtilization = Math.min(
    (diagramBounds.width * diagramBounds.height) / (viewportSize.width * viewportSize.height),
    1
  );

  // Check for hierarchy
  const hasHierarchy = nodes.some((n) => n.parentId);

  // Measure layout time (if performance tracking enabled)
  let layoutTime = 0;
  if (includePerformance) {
    layoutTime = await page.evaluate(() => {
      const perfEntries = performance.getEntriesByType("measure");
      const layoutEntry = perfEntries.find((e) => e.name.includes("layout"));
      return layoutEntry ? layoutEntry.duration : 0;
    });
  }

  return {
    exampleName,
    category,
    timestamp: new Date().toISOString(),
    weightedScore: qualityMetrics.weightedScore,
    overallScore: qualityMetrics.overallScore,
    grade: qualityMetrics.grade,
    overlappingNodes: qualityMetrics.overlappingNodes.length,
    edgeCrossings: qualityMetrics.edgeCrossings,
    edgesOverNodes: qualityMetrics.edgesOverNodes,
    edgeBends: qualityMetrics.edgeBends,
    spacingViolations: qualityMetrics.spacingViolations.length,
    parentChildViolations: qualityMetrics.parentChildContainment.length,
    nodeCount: nodes.length,
    edgeCount: edges.length,
    hasHierarchy,
    currentLevel: "L2",
    selectedEngine: config.engine,
    selectedDirection: config.direction,
    layoutTime,
    renderTime,
    viewportUtilization,
    aspectRatio,
    diagramBounds,
    qualityMetrics: includeFullMetrics ? qualityMetrics : ({} as DiagramQualityMetrics),
  };
}

/**
 * Collect baseline metrics for all examples
 */
export async function collectBaselineForAllExamples(
  page: Page,
  examples: Array<{ name: string; category: string }>,
  options: MetricsCollectionOptions = {}
): Promise<Map<string, LayoutMetrics>> {
  const results = new Map<string, LayoutMetrics>();

  for (const example of examples) {
    try {
      console.log(`Collecting metrics for: ${example.name}`);

      // Navigate to example
      await page.goto(`/?example=${encodeURIComponent(example.name)}`);
      await waitForLayoutStable(page, example.name);

      // Collect metrics
      const metrics = await collectLayoutMetrics(page, example.name, example.category, options);
      results.set(example.name, metrics);

      console.log(`  Score: ${metrics.weightedScore.toFixed(1)}/100 (${metrics.grade})`);
    } catch (error) {
      console.error(`Failed to collect metrics for ${example.name}:`, error);
    }
  }

  return results;
}

/**
 * Save metrics to JSON file
 */
export function saveMetricsToFile(metrics: Map<string, LayoutMetrics>, filepath: string): void {
  const data = {
    timestamp: new Date().toISOString(),
    metrics: Array.from(metrics.entries()).map(([_, m]) => ({
      ...m,
    })),
  };

  fs.writeFileSync(filepath, JSON.stringify(data, null, 2));
  console.log(`Saved ${metrics.size} metrics to ${filepath}`);
}

import { test } from "@playwright/test";
import { writeFileSync, mkdirSync, readdirSync, statSync, readFileSync } from "fs";
import { join, extname } from "path";
import LZString from "lz-string";

interface QualityMetrics {
  grade: string;
  weightedScore: number;
  overallScore: number;
  edgeCrossings: number;
  overlappingNodes: number;
  parentChildContainment: number;
  spacingViolations: number;
  edgeLabelOverlaps: number;
  clippedNodeLabels: number;
  exampleName: string;

  // Fields required by analyze-metrics.ts
  category: string;
  edgesOverNodes: number;
  edgeBends: number;
  nodeCount: number;
  edgeCount: number;
  hasHierarchy: boolean;
  currentLevel: string;
  selectedEngine: string;
  selectedDirection: string;
  viewportUtilization: number;
  aspectRatio: number;

  // Error tracking
  errors?: string[];
  consoleErrors?: string[];
  pageErrors?: string[];
  hasErrors?: boolean;
}

// Helper to recursively find all .sruja files
function findSrujaFiles(dir: string, fileList: string[] = [], relativePath: string = "") {
  const files = readdirSync(dir);

  files.forEach((file) => {
    const filePath = join(dir, file);
    const relPath = relativePath ? join(relativePath, file) : file;

    // Skip node_modules and dot files/dirs
    if (file.startsWith(".") || file === "node_modules") return;

    const stat = statSync(filePath);
    if (stat.isDirectory()) {
      findSrujaFiles(filePath, fileList, relPath);
    } else if (extname(file) === ".sruja") {
      fileList.push(relPath);
    }
  });

  return fileList;
}

test.describe.serial("All Examples Quality Measurement", () => {
  // Collection of all metrics
  const allMetrics: QualityMetrics[] = [];

  // Find all examples
  const examplesDir = join(process.cwd(), "public/examples");

  let allExamples: string[] = [];

  try {
    allExamples = findSrujaFiles(examplesDir);
  } catch (_e) {
    // Fallback for different CWD
    try {
      allExamples = findSrujaFiles(join(process.cwd(), "apps/designer/public/examples"));
    } catch (_e2) {
      console.error("Could not find examples directory");
    }
  }

  // Filter to only complex examples for focused testing
  // Complex examples are those with substantial architecture (150+ lines typically):
  // - pattern_* files (pattern_agentic_ai, pattern_microservices, pattern_rag_pipeline) - architectural patterns
  // - project_* files (project_ecommerce, project_iot_platform, project_saas_platform) - full project architectures
  // - demo_implied_relationships.sruja - complex microservices with implied relationships
  // - sruja_architecture_v2.sruja - architecture example
  // Excluded:
  // - demo_views_customization.sruja - Custom views not supported by designer UI (only L1/L2/L3)
  // - demo_governance.sruja (36 lines - too simple)
  // - reference_c4_model.sruja (57 lines - reference, not complex)
  // - concept_systems_thinking.sruja (118 lines - medium complexity, but simpler conceptually)
  const complexExamples = allExamples.filter((example) => {
    const fileName = example.split("/").pop() || example;
    return (
      fileName.startsWith("pattern_") ||
      fileName.startsWith("project_") ||
      fileName === "demo_implied_relationships.sruja" ||
      fileName === "sruja_architecture_v2.sruja"
    );
  });

  const examples = complexExamples;

  console.log(
    `Found ${allExamples.length} total examples, filtering to ${examples.length} complex examples for testing`
  );
  console.log(`Complex examples: ${examples.join(", ")}`);

  // Test all three levels (L1, L2, L3) for each example
  const levels = ["L1", "L2", "L3"] as const;

  for (const example of examples) {
    for (const level of levels) {
      test(`measure quality for ${example} at ${level}`, async ({ page }) => {
        // Track errors and console messages
        const consoleErrors: string[] = [];
        const pageErrors: string[] = [];
        const allErrors: string[] = [];

        // Listen to console messages (including errors)
        page.on("console", (msg) => {
          const text = msg.text();
          const type = msg.type();
          console.log(`PAGE LOG [${example} @ ${level}] [${type}]:`, text);

          // Capture console errors and warnings
          if (type === "error" || type === "warning") {
            consoleErrors.push(`[${type}] ${text}`);
            allErrors.push(`Console ${type}: ${text}`);
          }
        });

        // Listen to page errors (JavaScript errors)
        page.on("pageerror", (error) => {
          const errorMsg = error.message || String(error);
          pageErrors.push(errorMsg);
          allErrors.push(`Page Error: ${errorMsg}`);
          console.error(`PAGE ERROR [${example} @ ${level}]:`, error);
        });

        // Listen to request failures
        page.on("requestfailed", (request) => {
          const errorMsg = `${request.method()} ${request.url()} - ${request.failure()?.errorText || "Failed"}`;
          allErrors.push(`Request Failed: ${errorMsg}`);
          console.error(`REQUEST FAILED [${example} @ ${level}]:`, errorMsg);
        });

        // Set base URL
        const baseURL = process.env.PLAYWRIGHT_BASE_URL || "http://localhost:4321";
        const isProduction = baseURL.includes("4322") || baseURL.includes("preview");
        const designerPath = isProduction ? "/designer" : "/designer";

        let codeUrlParam = "";

        if (example === "MANUAL_TRULY_FLAT_CODE") {
          const compressed = LZString.compressToBase64(trulyFlatDsl);
          codeUrlParam = `code=${encodeURIComponent(compressed)}`;
        } else {
          // Read file content
          let fullPath;
          try {
            fullPath = join(examplesDir, example);
            // Ensure it exists (might fail if we are in wrong CWD for initial find)
            if (!statSync(fullPath).isFile()) throw new Error("Not a file");
          } catch {
            fullPath = join(process.cwd(), "apps/designer/public/examples", example);
          }

          try {
            const content = readFileSync(fullPath, "utf-8");
            // Use compressToBase64 to match app's expectation (useProjectSync.ts lines 219)
            const compressed = LZString.compressToBase64(content);
            codeUrlParam = `code=${encodeURIComponent(compressed)}`;
          } catch (_e) {
            console.error(`Failed to read content for ${example}: ${_e}`);
            test.skip();
            return;
          }
        }

        const urlWithParams = `${baseURL}${designerPath}?level=${level}&tab=diagram&${codeUrlParam}&autorun=true`;

        console.log(
          `Testing: ${example} at ${level} (via code injection, length ${codeUrlParam.length})`
        );

        // Navigate to the example
        // We set a shorter timeout because if it hangs, we want to move on and mark as failed/0 score
        try {
          await page.goto(urlWithParams, { waitUntil: "networkidle", timeout: 30000 });
        } catch (_e) {
          console.log(
            `Timeout loading ${example} at ${level}, trying to proceed anyway in case it's just a network idle issue`
          );
        }

        // Wait for diagram to appear
        try {
          await page.waitForSelector(".react-flow svg", { timeout: 20000 });

          // Wait for layout stability
          await page.waitForTimeout(3000);

          // Wait for metrics
          let diagramQuality: {
            score?: number;
            edgeCrossings?: number;
            nodeOverlaps?: number;
            labelOverlaps?: number;
            nodeCount?: number;
            edgeCount?: number;
            spacingConsistency?: number;
          } | null = null;
          for (let i = 0; i < 20; i++) {
            // Increased retries
            diagramQuality = (await page.evaluate(() => {
              return (window as unknown as any).__DIAGRAM_QUALITY__;
            })) as {
              score?: number;
              edgeCrossings?: number;
              nodeOverlaps?: number;
              labelOverlaps?: number;
              nodeCount?: number;
              edgeCount?: number;
              spacingConsistency?: number;
            } | null;
            if (diagramQuality) break;
            await page.waitForTimeout(500);
          }

          if (diagramQuality) {
            const score = (diagramQuality.score || 0) * 100;
            let grade = "F";
            if (score >= 90) grade = "A";
            else if (score >= 80) grade = "B";
            else if (score >= 70) grade = "C";
            else if (score >= 60) grade = "D";

            const metrics: QualityMetrics = {
              exampleName: example,
              grade: grade,
              weightedScore: score,
              overallScore: score,
              edgeCrossings: diagramQuality.edgeCrossings || 0,
              overlappingNodes: diagramQuality.nodeOverlaps || 0,
              parentChildContainment: 0, // Not provided by LayoutQuality yet
              spacingViolations: (1 - (diagramQuality.spacingConsistency || 0)) * 10, // heuristic
              edgeLabelOverlaps: diagramQuality.labelOverlaps || 0,
              clippedNodeLabels: 0, // Not provided

              // Defaults for fields not provided by __DIAGRAM_QUALITY__
              category: "unknown",
              edgesOverNodes: 0,
              edgeBends: 0,
              nodeCount: diagramQuality.nodeCount || 0,
              edgeCount: diagramQuality.edgeCount || 0,
              hasHierarchy: false,
              currentLevel: level,
              selectedEngine: "sruja",
              selectedDirection: "TB",
              viewportUtilization: 0.5,
              aspectRatio: 1.0,

              // Error tracking
              errors: allErrors.length > 0 ? allErrors : undefined,
              consoleErrors: consoleErrors.length > 0 ? consoleErrors : undefined,
              pageErrors: pageErrors.length > 0 ? pageErrors : undefined,
              hasErrors: allErrors.length > 0,
            };

            allMetrics.push(metrics);
          } else {
            console.error(`No metrics found for ${example} at ${level}`);
            // Push a failed state record
            allMetrics.push({
              exampleName: example,
              grade: "N/A",
              weightedScore: 0,
              overallScore: 0,
              edgeCrossings: -1,
              overlappingNodes: -1,
              parentChildContainment: -1,
              spacingViolations: -1,
              edgeLabelOverlaps: -1,
              clippedNodeLabels: -1,
              category: "unknown",
              edgesOverNodes: 0,
              edgeBends: 0,
              nodeCount: 0,
              edgeCount: 0,
              hasHierarchy: false,
              currentLevel: level,
              selectedEngine: "sruja",
              selectedDirection: "TB",
              viewportUtilization: 0,
              aspectRatio: 0,
              // Error tracking
              errors: allErrors.length > 0 ? allErrors : ["No metrics found"],
              consoleErrors: consoleErrors.length > 0 ? consoleErrors : undefined,
              pageErrors: pageErrors.length > 0 ? pageErrors : undefined,
              hasErrors: true,
            });
          }
        } catch (e) {
          const errorMessage = e instanceof Error ? e.message : String(e);
          const errorStack = e instanceof Error ? e.stack : undefined;
          console.error(`Failed to visualize ${example} at ${level}: ${e}`);

          // Add the caught error to the error list
          allErrors.push(`Test Error: ${errorMessage}`);
          if (errorStack) {
            allErrors.push(`Stack: ${errorStack}`);
          }

          allMetrics.push({
            exampleName: example,
            grade: "ERROR",
            weightedScore: 0,
            overallScore: 0,
            edgeCrossings: -1,
            overlappingNodes: -1,
            parentChildContainment: -1,
            spacingViolations: -1,
            edgeLabelOverlaps: -1,
            clippedNodeLabels: -1,
            category: "unknown",
            edgesOverNodes: 0,
            edgeBends: 0,
            nodeCount: 0,
            edgeCount: 0,
            hasHierarchy: false,
            currentLevel: level,
            selectedEngine: "sruja",
            selectedDirection: "TB",
            viewportUtilization: 0,
            aspectRatio: 0,
            // Error tracking
            errors: allErrors.length > 0 ? allErrors : [errorMessage],
            consoleErrors: consoleErrors.length > 0 ? consoleErrors : undefined,
            pageErrors: pageErrors.length > 0 ? pageErrors : undefined,
            hasErrors: true,
          });
        }
      });
    }
  }

  test.afterAll(() => {
    // Save aggregated results
    const resultsDir = join(process.cwd(), "tests", "results");
    mkdirSync(resultsDir, { recursive: true });
    const reportFile = join(resultsDir, "all-examples-metrics.json");

    // Wrap in expected structure for analyze-metrics.ts
    // The existing script expects `metrics` array
    const output = {
      timestamp: new Date().toISOString(),
      metrics: allMetrics,
    };

    writeFileSync(reportFile, JSON.stringify(output, null, 2));
    console.log(`Saved aggregated metrics to ${reportFile}`);
  });
});

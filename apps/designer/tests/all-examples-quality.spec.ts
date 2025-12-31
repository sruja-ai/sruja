import { test, expect } from "@playwright/test";
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

  let examples: string[] = [];

  try {
    examples = findSrujaFiles(examplesDir);
  } catch (e) {
    // Fallback for different CWD
    try {
      examples = findSrujaFiles(join(process.cwd(), "apps/designer/public/examples"));
    } catch (e2) {
      console.error("Could not find examples directory");
    }
  }

  // Add a manual test case for nested syntax via code injection
  const trulyFlatDsl = `person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Backend = system "Backend" {
  description "A backend system"
  API = container "API"
}

User -> Backend "uses"`;

  // We'll run this as a special "example" name
  // Note: logic below handles this specifically
  examples.push("MANUAL_TRULY_FLAT_CODE");

  console.log(`Found ${examples.length} examples to test`);

  for (const example of examples) {
    test(`measure quality for ${example}`, async ({ page }) => {
      // Listen to console logs
      page.on("console", (msg) => console.log(`PAGE LOG [${example}]:`, msg.text()));

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
        } catch (e) {
          console.error(`Failed to read content for ${example}: ${e}`);
          test.skip();
          return;
        }
      }

      const urlWithParams = `${baseURL}${designerPath}?level=L1&tab=diagram&${codeUrlParam}&autorun=true`;

      console.log(`Testing: ${example} (via code injection, length ${codeUrlParam.length})`);

      // Navigate to the example
      // We set a shorter timeout because if it hangs, we want to move on and mark as failed/0 score
      try {
        await page.goto(urlWithParams, { waitUntil: "networkidle", timeout: 30000 });
      } catch (e) {
        console.log(
          `Timeout loading ${example}, trying to proceed anyway in case it's just a network idle issue`
        );
      }

      // Wait for diagram to appear
      try {
        await page.waitForSelector(".react-flow svg", { timeout: 20000 });

        // Wait for layout stability
        await page.waitForTimeout(3000);

        // Wait for metrics
        let diagramQuality: any = null;
        for (let i = 0; i < 20; i++) {
          // Increased retries
          diagramQuality = await page.evaluate(() => {
            return (window as any).__DIAGRAM_QUALITY__ as any;
          });
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
            currentLevel: "L1",
            selectedEngine: "sruja",
            selectedDirection: "TB",
            viewportUtilization: 0.5,
            aspectRatio: 1.0,
          };

          allMetrics.push(metrics);
        } else {
          console.error(`No metrics found for ${example}`);
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
            currentLevel: "L1",
            selectedEngine: "sruja",
            selectedDirection: "TB",
            viewportUtilization: 0,
            aspectRatio: 0,
          });
        }
      } catch (e) {
        console.error(`Failed to visualize ${example}: ${e}`);
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
          currentLevel: "L1",
          selectedEngine: "sruja",
          selectedDirection: "TB",
          viewportUtilization: 0,
          aspectRatio: 0,
        });
      }
    });
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

/**
 * Text Measurement Utility
 *
 * Canvas-based text measurement for accurate node sizing.
 * Uses OffscreenCanvas when available for better performance.
 */

// Cache for text measurements to avoid repeated calculations
const measureCache = new Map<string, { width: number; height: number }>();

// Font configuration interface
interface FontConfig {
  family: string;
  size: number;
  weight: number;
  lineHeight: number;
}

// Font configurations matching node styles
export const FONT_CONFIG: Record<"title" | "technology" | "description", FontConfig> = {
  title: {
    family: "Inter, -apple-system, BlinkMacSystemFont, sans-serif",
    size: 14,
    weight: 600,
    lineHeight: 1.3,
  },
  technology: {
    family: "Inter, -apple-system, BlinkMacSystemFont, sans-serif",
    size: 11,
    weight: 500,
    lineHeight: 1.2,
  },
  description: {
    family: "Inter, -apple-system, BlinkMacSystemFont, sans-serif",
    size: 11,
    weight: 400,
    lineHeight: 1.4,
  },
};

// Canvas for text measurement
let measureCanvas: HTMLCanvasElement | OffscreenCanvas | null = null;
let measureCtx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D | null = null;

function getContext() {
  if (!measureCtx) {
    if (typeof OffscreenCanvas !== "undefined") {
      measureCanvas = new OffscreenCanvas(1, 1);
      measureCtx = measureCanvas.getContext("2d");
    } else if (typeof document !== "undefined") {
      measureCanvas = document.createElement("canvas");
      measureCtx = measureCanvas.getContext("2d");
    }
  }
  return measureCtx;
}

/**
 * Measure text width and height for a given string and font configuration
 */
export function measureText(
  text: string,
  fontConfig: FontConfig
): { width: number; height: number } {
  if (!text) return { width: 0, height: 0 };

  const cacheKey = `${text}|${fontConfig.size}|${fontConfig.weight}`;
  const cached = measureCache.get(cacheKey);
  if (cached) return cached;

  const ctx = getContext();
  if (!ctx) {
    // Fallback: estimate based on character count
    const width = text.length * fontConfig.size * 0.6;
    const height = fontConfig.size * fontConfig.lineHeight;
    return { width, height };
  }

  ctx.font = `${fontConfig.weight} ${fontConfig.size}px ${fontConfig.family}`;
  const metrics = ctx.measureText(text);

  const width = metrics.width;
  const height = fontConfig.size * fontConfig.lineHeight;

  const result = { width, height };
  measureCache.set(cacheKey, result);
  return result;
}

/**
 * Measure multi-line text with word wrapping
 */
export function measureMultilineText(
  text: string,
  maxWidth: number,
  fontConfig: FontConfig
): { width: number; height: number; lines: string[] } {
  if (!text) return { width: 0, height: 0, lines: [] };

  const words = text.split(" ");
  const lines: string[] = [];
  let currentLine = "";
  let maxLineWidth = 0;

  const ctx = getContext();
  if (ctx) {
    ctx.font = `${fontConfig.weight} ${fontConfig.size}px ${fontConfig.family}`;
  }

  for (const word of words) {
    const testLine = currentLine ? `${currentLine} ${word}` : word;
    const testWidth = ctx
      ? ctx.measureText(testLine).width
      : testLine.length * fontConfig.size * 0.5;

    if (testWidth > maxWidth && currentLine) {
      lines.push(currentLine);
      const lineWidth = ctx
        ? ctx.measureText(currentLine).width
        : currentLine.length * fontConfig.size * 0.5;
      maxLineWidth = Math.max(maxLineWidth, lineWidth);
      currentLine = word;
    } else {
      currentLine = testLine;
    }
  }

  if (currentLine) {
    lines.push(currentLine);
    const lineWidth = ctx
      ? ctx.measureText(currentLine).width
      : currentLine.length * fontConfig.size * 0.5;
    maxLineWidth = Math.max(maxLineWidth, lineWidth);
  }

  const height = lines.length * fontConfig.size * fontConfig.lineHeight;

  return { width: maxLineWidth, height, lines };
}

/**
 * Calculate optimal node size based on content
 */
export function calculateNodeSize(
  title: string,
  technology?: string,
  description?: string,
  nodeType: string = "container"
): { width: number; height: number } {
  // Padding based on node type
  const padding = {
    horizontal: nodeType === "person" ? 24 : 20,
    vertical: nodeType === "person" ? 20 : 16,
  };

  // Measure title
  const titleMeasure = measureText(title, FONT_CONFIG.title);

  // Measure technology tag
  const techMeasure = technology
    ? measureText(`[${technology}]`, FONT_CONFIG.technology)
    : { width: 0, height: 0 };

  // Calculate max width for description wrapping
  const contentWidth = Math.max(titleMeasure.width, techMeasure.width);
  const maxDescWidth = Math.max(contentWidth, 300); // Increased min width for wrapping to 300px

  // Measure description with wrapping (max 2 lines)
  const descMeasure = description
    ? measureMultilineText(description, maxDescWidth, FONT_CONFIG.description)
    : { width: 0, height: 0, lines: [] };

  // Limit description to reasonable max (e.g. 10 lines) instead of 2
  // We want to show as much as possible without breaking the layout completely
  const descHeight = descMeasure.height; // Use full measured height

  // Calculate total dimensions
  const totalContentWidth = Math.max(titleMeasure.width, techMeasure.width, descMeasure.width);

  const width = totalContentWidth + padding.horizontal * 2;

  const height =
    titleMeasure.height +
    (techMeasure.height > 0 ? techMeasure.height + 4 : 0) +
    (descHeight > 0 ? descHeight + 8 : 0) +
    padding.vertical * 2 +
    24; // Icon space

  // Apply min/max constraints based on node type
  // Significantly relaxed constraints to fit content
  const constraints = {
    person: { minWidth: 140, maxWidth: 300, minHeight: 180, maxHeight: 400 },
    system: { minWidth: 150, maxWidth: 400, minHeight: 100, maxHeight: 400 },
    container: { minWidth: 160, maxWidth: 400, minHeight: 100, maxHeight: 400 },
    component: { minWidth: 140, maxWidth: 400, minHeight: 90, maxHeight: 400 },
    datastore: { minWidth: 140, maxWidth: 300, minHeight: 100, maxHeight: 400 },
    queue: { minWidth: 140, maxWidth: 300, minHeight: 100, maxHeight: 400 },
  };

  const nodeConstraints =
    constraints[nodeType as keyof typeof constraints] || constraints.container;

  return {
    width: Math.max(nodeConstraints.minWidth, Math.min(nodeConstraints.maxWidth, Math.ceil(width))),
    height: Math.max(
      nodeConstraints.minHeight,
      Math.min(nodeConstraints.maxHeight, Math.ceil(height))
    ),
  };
}

/**
 * Clear measurement cache (call when font styles change)
 */
export function clearMeasureCache(): void {
  measureCache.clear();
}

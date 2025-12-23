import type { LikeC4Model } from "@likec4/core/model";
import { getNodeColors } from "../../utils/colorScheme";

/**
 * Configuration for node styling operations.
 */
export interface NodeStylerConfig {
  containerRef: React.RefObject<HTMLDivElement | null>;
  likec4Model: LikeC4Model<any> | null;
  selectedNodeId: string | null;
}

/**
 * Applies node colors and selection highlighting to LikeC4 diagram elements.
 * 
 * @param config - Configuration object containing container ref, model, and selected node ID
 * 
 * @remarks
 * This function:
 * 1. Iterates through all elements in the LikeC4 model
 * 2. Finds corresponding DOM elements using multiple strategies
 * 3. Applies `data-element-kind` attributes for CSS styling
 * 4. Applies inline styles as fallback
 * 5. Handles selection highlighting
 * 
 * Uses multiple element-finding strategies to handle LikeC4's varying DOM structures:
 * - Direct attribute selectors (data-element-id, data-node-id, etc.)
 * - Text content matching (by element title)
 * - Group traversal and matching
 */
export function applyNodeColors(config: NodeStylerConfig): void {
  const { containerRef, likec4Model, selectedNodeId } = config;

  if (!containerRef.current || !likec4Model) return;

  const svg = containerRef.current.querySelector('svg');
  if (!svg) return;

  try {
    const elements = [...likec4Model.elements()];
    const processedElements = new Set<string>();

    elements.forEach((modelElement) => {
      const elementId = modelElement.id;
      const kind = modelElement.kind || '';
      if (!kind || processedElements.has(elementId)) return;

      const element = findElementInDOM(svg, elementId, modelElement);

      if (element) {
        processedElements.add(elementId);
        applyStylingToElement(element, kind, modelElement, elementId === selectedNodeId);
      } else {
        console.debug(`[LikeC4Canvas] Could not find DOM element for model element: ${elementId} (${kind})`);
      }
    });
  } catch (e) {
    console.warn('[LikeC4Canvas] Error applying node colors:', e);
  }
}

/**
 * Finds a DOM element corresponding to a model element using multiple strategies.
 * 
 * @param svg - The SVG root element
 * @param elementId - The model element ID
 * @param modelElement - The model element object
 * @returns The found DOM element or null
 */
function findElementInDOM(
  svg: SVGElement,
  elementId: string,
  modelElement: any
): Element | null {
  // Strategy 1: Try direct attribute selectors
  const attributeSelectors = [
    `[data-element-id="${elementId}"]`,
    `[data-node-id="${elementId}"]`,
    `[data-id="${elementId}"]`,
    `[id="${elementId}"]`,
    `[id="likec4-${elementId}"]`,
    `[aria-label*="${elementId}"]`,
    `[aria-label*="${modelElement.title}"]`,
  ];

  for (const selector of attributeSelectors) {
    const element = svg.querySelector(selector);
    if (element) return element;
  }

  // Strategy 2: Try to find by title text content
  const textElements = svg.querySelectorAll('text');
  for (const textEl of textElements) {
    const text = textEl.textContent?.trim();
    if (text === modelElement.title || text === elementId) {
      // Find the parent group (LikeC4 typically wraps nodes in <g> elements)
      let parent: Element | null = textEl.parentElement;
      let depth = 0;
      while (parent && parent !== (svg as Element) && depth < 5) {
        if (parent.tagName === 'g' && parent.querySelector('rect, circle, ellipse, path')) {
          return parent;
        }
        parent = parent.parentElement;
        depth++;
      }
    }
  }

  // Strategy 3: Search all groups and try to match by various means
  const allGroups = svg.querySelectorAll('g');
  for (const group of allGroups) {
    // Check if this group contains shapes (likely a node)
    const hasShapes = group.querySelector('rect, circle, ellipse, path, polygon');
    if (!hasShapes) continue;

    // Check various attributes
    const groupId = group.getAttribute('id') ||
      group.getAttribute('data-id') ||
      group.getAttribute('data-element-id') ||
      group.getAttribute('data-node-id') ||
      group.getAttribute('aria-label');

    if (groupId && (groupId.includes(elementId) || groupId.includes(modelElement.title))) {
      return group;
    }

    // Check if text content matches
    const textEl = group.querySelector('text');
    if (textEl && (textEl.textContent?.trim() === modelElement.title || textEl.textContent?.trim() === elementId)) {
      return group;
    }
  }

  return null;
}

/**
 * Applies styling (colors and selection) to a DOM element.
 * 
 * @param element - The DOM element to style
 * @param kind - The element kind (person, system, container, etc.)
 * @param modelElement - The model element object
 * @param isSelected - Whether this element is currently selected
 */
function applyStylingToElement(
  element: Element,
  kind: string,
  modelElement: any,
  isSelected: boolean
): void {
  // Apply kind attribute for CSS styling
  element.setAttribute('data-element-kind', kind);

  // Apply to all child shapes
  const shapes = element.querySelectorAll('rect, circle, ellipse, path, polygon, line');
  const colors = getNodeColors(kind, modelElement.isExternal);

  shapes.forEach((shape) => {
    shape.setAttribute('data-element-kind', kind);

    // Apply inline styles as fallback (CSS with !important will override)
    if (shape instanceof SVGElement) {
      shape.style.fill = colors.bg;
      // For filled style, border might be same as BG or darker.
      // If border is same as BG, use it for stroke too.
      shape.style.stroke = colors.border || (colors as any).bg;
      if (colors.border) {
        shape.style.strokeWidth = '2';
      }
    }
  });

  // Apply to direct shape children if element is a group
  if (element.tagName === 'g') {
    const directShapes = Array.from(element.children).filter(
      child => ['rect', 'circle', 'ellipse', 'path', 'polygon', 'line'].includes(child.tagName.toLowerCase())
    );
    directShapes.forEach((shape) => {
      shape.setAttribute('data-element-kind', kind);
      if (shape instanceof SVGElement) {
        shape.style.fill = colors.bg;
        shape.style.stroke = colors.border || (colors as any).bg;
        if (colors.border) {
          shape.style.strokeWidth = '2';
        }
      }
    });
  }

  // Handle selection highlighting
  const allShapes = element.querySelectorAll('rect, circle, ellipse, path, polygon, line');
  if (isSelected) {
    element.classList.add('selected');
    allShapes.forEach((shape) => {
      shape.classList.add('selected');
    });
  } else {
    element.classList.remove('selected');
    allShapes.forEach((shape) => {
      shape.classList.remove('selected');
    });
  }
}


// SVG processing utilities - extracted from duplicate code

/**
 * Normalizes SVG by removing fixed dimensions and setting responsive attributes
 */
export function normalizeSvg(svgString: string): string {
  if (!svgString || typeof svgString !== 'string') {
    return '';
  }

  try {
    const parser = new DOMParser();
    const doc = parser.parseFromString(svgString, 'image/svg+xml');
    const svg = doc.querySelector('svg');
    
    if (!svg) {
      return svgString; // Return original if no SVG found
    }

    // Remove fixed dimensions
    svg.removeAttribute('width');
    svg.removeAttribute('height');
    
    // Set responsive attributes
    svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
    svg.setAttribute('style', 'width:100%;height:auto;max-height:100%;display:block');
    svg.classList.add('canvas-svg');
    
    return svg.outerHTML;
  } catch (error) {
    console.error('Failed to normalize SVG:', error);
    return svgString; // Return original on error
  }
}

/**
 * Processes SVG output from compilation result
 * Handles multiple output formats (svg, html, image)
 * 
 * Note: Higher complexity (16) is intentional for multi-format support -
 * processing SVG, HTML (with SVG extraction), and multiple image formats
 * requires branching logic that cannot be easily reduced.
 */
// codacy-ignore: complexity - Multi-format processor requires branching for each format type
export function processCompileOutput(result: {
  svg?: string;
  html?: string;
  image?: string;
  png?: string;
  jpg?: string;
  jpeg?: string;
  error?: string;
} | null): string {
  if (!result) {
    return '';
  }
  
  if (result.error) {
    return '';
  }
  if (result.svg) {
    return normalizeSvg(result.svg);
  }
  
  if (result.html) {
    // Try to extract SVG from HTML
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(result.html, 'text/html');
      const svg = doc.querySelector('svg');
      if (svg) {
        return normalizeSvg(svg.outerHTML);
      }
      return result.html;
    } catch {
      return result.html;
    }
  }
  
  if (result.image || result.png || result.jpg || result.jpeg) {
    const src = result.image || result.png || result.jpg || result.jpeg || '';
    if (src) {
      return `<img src="${src}" alt="Diagram" style="width:100%;height:auto;max-height:100%;display:block"/>`;
    }
  }
  
  return '';
}


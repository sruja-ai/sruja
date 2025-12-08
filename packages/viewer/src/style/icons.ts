// packages/viewer/src/style/icons.ts
import type { NodeSingular } from 'cytoscape';

/**
 * Generate SVG icon data URI
 */
function getIconDataUri(svgContent: string): string {
  const cleaned = svgContent.replace(/\s+/g, ' ').trim();
  return `data:image/svg+xml,${encodeURIComponent(cleaned)}`;
}

/**
 * Get C4 model icons as SVG data URIs
 */
export function getC4Icons() {
  const personIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>'
  );
  const systemIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line><line x1="7" y1="8" x2="7" y2="8.01"></line><line x1="12" y1="8" x2="12" y2="8.01"></line><line x1="17" y1="8" x2="17" y2="8.01"></line><line x1="7" y1="12" x2="17" y2="12"></line></svg>'
  );
  const containerIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>'
  );
  return { personIcon, systemIcon, containerIcon };
}

/**
 * Get icon from metadata or return default
 */
export function getIconFromMetadata(
  ele: NodeSingular,
  defaultIcon: string,
  getStyleValue: (ele: NodeSingular, key: string, defaultVal: string | number | undefined) => string | number | undefined
): string {
  try {
    const iconName = getStyleValue(ele, 'style.icon', '');
    if (!iconName || typeof iconName !== 'string') {
      return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
    }

    const icons = getC4Icons();
    switch (iconName) {
      case 'person':
        return icons.personIcon || defaultIcon || 'none';
      case 'system':
        return icons.systemIcon || defaultIcon || 'none';
      case 'container':
        return icons.containerIcon || defaultIcon || 'none';
      case 'database':
        return (
          getIconDataUri(
            '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>'
          ) || defaultIcon || 'none'
        );
      case 'cloud':
        return (
          getIconDataUri(
            '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"></path></svg>'
          ) || defaultIcon || 'none'
        );
      case 'server':
        return (
          getIconDataUri(
            '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>'
          ) || defaultIcon || 'none'
        );
      case 'mobile':
        return (
          getIconDataUri(
            '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect><line x1="12" y1="18" x2="12.01" y2="18"></line></svg>'
          ) || defaultIcon || 'none'
        );
      case 'browser':
        return (
          getIconDataUri(
            '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>'
          ) || defaultIcon || 'none'
        );
      default:
        return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
    }
  } catch {
    return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none';
  }
}

// packages/viewer/src/style/helpers.ts
import type { NodeSingular } from 'cytoscape';

/**
 * Get style value from node metadata
 */
export function getStyleValue(
  ele: NodeSingular,
  key: string,
  defaultVal: string | number | undefined
): string | number | undefined {
  try {
    const meta = ele.data('metadata') as Array<{ key: string; value?: string }> | undefined;
    if (!meta || !Array.isArray(meta)) return defaultVal;
    const entry = meta.find((m) => m && m.key === key);
    if (!entry) return defaultVal;
    const value = entry.value;
    return value != null && value !== '' ? value : defaultVal;
  } catch {
    return defaultVal;
  }
}

/**
 * Calculate node width based on label and type
 */
export function calculateNodeWidth(ele: NodeSingular): number {
  const label = ele.data('label') || '';
  const type = ele.data('type');
  const charWidth = 7.5;
  const textMaxWidth = type === 'person' ? 100 : type === 'system' ? 110 : 120;
  const labelWidth = label.length * charWidth;
  const textWidth = Math.min(labelWidth, textMaxWidth);

  let paddingX = 0;
  switch (type) {
    case 'person':
      paddingX = 24;
      break;
    case 'system':
      paddingX = 66;
      break;
    case 'container': {
      const iconName = getStyleValue(ele, 'style.icon', '');
      const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
      paddingX = hasIcon ? 52 : 24;
      break;
    }
    case 'datastore':
    case 'queue':
      paddingX = 24;
      break;
    default:
      paddingX = 24;
  }

  const minWidth = type === 'system' ? 120 : 80;
  return Math.max(minWidth, textWidth + paddingX);
}

/**
 * Calculate node height based on label and type
 */
export function calculateNodeHeight(ele: NodeSingular): number {
  const label = ele.data('label') || '';
  const type = ele.data('type');
  const charWidth = 7.5;
  const textMaxWidth = type === 'person' ? 100 : type === 'system' ? 110 : 120;
  const lineHeight = 16;

  const words = label.split(' ');
  let lineCount = 1;
  let currentLineLen = 0;

  for (const word of words) {
    const wordLen = word.length * charWidth;
    if (currentLineLen + wordLen > textMaxWidth) {
      lineCount++;
      currentLineLen = wordLen;
    } else {
      currentLineLen += wordLen + charWidth;
    }
  }

  let paddingY = 0;
  switch (type) {
    case 'person':
      paddingY = 56;
      break;
    case 'system':
      paddingY = 32;
      break;
    case 'container':
    case 'datastore':
    case 'queue':
      paddingY = 24;
      break;
    default:
      paddingY = 24;
  }

  const minHeight = type === 'person' ? 70 : type === 'system' ? 50 : 40;
  return Math.max(minHeight, lineCount * lineHeight + paddingY);
}

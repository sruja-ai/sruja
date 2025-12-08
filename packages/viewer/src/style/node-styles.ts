// packages/viewer/src/style/node-styles.ts
import type { NodeSingular } from 'cytoscape';
import { colors } from './colors';
import { Colors } from '@sruja/shared/utils/cssVars';
import { getStyleValue, calculateNodeWidth, calculateNodeHeight } from './helpers';
import { getC4Icons, getIconFromMetadata } from './icons';

/**
 * Build base node style
 */
export function buildBaseNodeStyle() {
  return {
    selector: 'node',
    style: {
      'text-valign': 'center',
      'text-halign': 'center',
      'text-wrap': 'wrap',
      'text-max-width': 120,
      'font-family': 'Inter, system-ui, sans-serif',
      'font-size': 12,
      'font-weight': 500,
      'border-width': 2,
      'overlay-opacity': 0,
      'transition-property': 'background-color, border-color, text-rotation',
      'transition-duration': 300,
      'width': (ele: NodeSingular) => calculateNodeWidth(ele),
      'height': (ele: NodeSingular) => calculateNodeHeight(ele),
      'label': 'data(label)',
      'text-overflow-wrap': 'anywhere',
    },
  };
}

/**
 * Build person node style
 */
export function buildPersonStyle() {
  const icons = getC4Icons();
  return {
    selector: 'node[type="person"]',
    style: {
      'background-color': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.color', colors.person) || colors.person,
      'border-color': colors.personBorder,
      'color': colors.textLight,
      'label': 'data(label)',
      'shape': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
      'padding': 12,
      'text-max-width': 100,
      'background-image': (ele: NodeSingular) =>
        getIconFromMetadata(ele, icons.personIcon, getStyleValue) || icons.personIcon,
      'background-width': '20px',
      'background-height': '20px',
      'background-position-x': '50%',
      'background-position-y': '20%',
      'background-repeat': 'no-repeat',
      'text-valign': 'center',
      'text-halign': 'center',
      'text-margin-y': 14,
      'text-margin-x': 0,
      'text-background-padding': '2px',
      'font-weight': 600,
      'font-size': 11,
    },
  };
}

/**
 * Build system node style
 */
export function buildSystemStyle() {
  const icons = getC4Icons();
  return {
    selector: 'node[type="system"]',
    style: {
      'background-color': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.color', colors.system) || colors.system,
      'border-color': colors.systemBorder,
      'color': colors.textLight,
      'label': 'data(label)',
      'shape': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
      'padding': 16,
      'padding-left': 50,
      'padding-right': 16,
      'padding-top': 16,
      'padding-bottom': 16,
      'text-max-width': 110,
      'background-image': (ele: NodeSingular) =>
        getIconFromMetadata(ele, icons.systemIcon, getStyleValue) || icons.systemIcon,
      'background-width': '18px',
      'background-height': '18px',
      'background-position-x': '16px',
      'background-position-y': '50%',
      'background-repeat': 'no-repeat',
      'text-halign': 'left',
      'text-valign': 'center',
      'text-margin-x': 0,
      'text-margin-y': 0,
      'font-size': 14,
      'font-weight': 700,
      'text-transform': 'uppercase',
    },
  };
}

/**
 * Build container node style
 */
export function buildContainerStyle() {
  return {
    selector: 'node[type="container"]',
    style: {
      'background-color': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.color', colors.container) || colors.container,
      'border-color': colors.containerBorder,
      'color': colors.textDark,
      'label': 'data(label)',
      'shape': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
      'padding': 12,
      'padding-left': (ele: NodeSingular) => {
        const iconName = getStyleValue(ele, 'style.icon', '');
        const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
        return hasIcon ? 40 : 12;
      },
      'padding-right': 12,
      'padding-top': 12,
      'padding-bottom': 12,
      'text-max-width': 120,
      'background-image': (ele: NodeSingular) => {
        const iconName = getStyleValue(ele, 'style.icon', '');
        if (!iconName || typeof iconName !== 'string') return 'none';
        return getIconFromMetadata(ele, '', getStyleValue);
      },
      'background-width': '16px',
      'background-height': '16px',
      'background-position-x': '12px',
      'background-position-y': '50%',
      'background-repeat': 'no-repeat',
      'text-halign': (ele: NodeSingular) => {
        const iconName = getStyleValue(ele, 'style.icon', '');
        const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
        return hasIcon ? 'left' : 'center';
      },
      'text-valign': 'center',
      'text-margin-x': (ele: NodeSingular) => {
        const iconName = getStyleValue(ele, 'style.icon', '');
        const hasIcon = iconName && typeof iconName === 'string' && iconName !== '';
        return hasIcon ? 8 : 0;
      },
      'text-margin-y': 0,
    },
  };
}

/**
 * Build datastore node style
 */
export function buildDatastoreStyle() {
  return {
    selector: 'node[type="datastore"]',
    style: {
      'background-color': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.color', colors.database) || colors.database,
      'border-color': colors.databaseBorder,
      'color': colors.textDark,
      'label': 'data(label)',
      'shape': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.shape', 'barrel') || 'barrel',
      'padding': 12,
      'padding-left': 12,
      'padding-right': 12,
      'padding-top': 12,
      'padding-bottom': 12,
      'text-max-width': 120,
      'border-width': 2,
      'background-image': (ele: NodeSingular) => {
        const icon = getIconFromMetadata(ele, '', getStyleValue);
        return icon && icon !== '' ? icon : 'none';
      },
      'background-width': '16px',
      'background-height': '16px',
      'background-position-x': '50%',
      'background-position-y': '30%',
      'background-repeat': 'no-repeat',
      'text-halign': 'center',
      'text-valign': 'center',
      'text-margin-y': 8,
      'text-margin-x': 0,
    },
  };
}

/**
 * Build queue node style
 */
export function buildQueueStyle() {
  return {
    selector: 'node[type="queue"]',
    style: {
      'background-color': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.color', colors.container) || colors.container,
      'border-color': colors.containerBorder,
      'color': colors.textDark,
      'label': 'data(label)',
      'shape': (ele: NodeSingular) =>
        getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
      'padding': 12,
      'padding-left': 12,
      'padding-right': 12,
      'padding-top': 12,
      'padding-bottom': 12,
      'text-max-width': 120,
      'background-image': (ele: NodeSingular) => {
        const icon = getIconFromMetadata(ele, '', getStyleValue);
        return icon && icon !== '' ? icon : 'none';
      },
      'background-width': '16px',
      'background-height': '16px',
      'background-position-x': '50%',
      'background-position-y': '30%',
      'background-repeat': 'no-repeat',
      'text-halign': 'center',
      'text-valign': 'center',
      'text-margin-y': 8,
      'text-margin-x': 0,
    },
  };
}

/**
 * Build edge style
 */
export function buildEdgeStyle() {
  return {
    selector: 'edge',
    style: {
      'width': 2,
      'line-color': Colors.neutral500(),
      'target-arrow-color': Colors.neutral500(),
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
      'label': 'data(label)',
      'font-size': '10px',
      'color': colors.edge,
      'text-background-color': Colors.background(),
      'text-background-opacity': 1,
      'text-background-padding': '2px',
      'text-background-shape': 'round-rectangle',
      'text-border-opacity': 0,
      'text-rotation': 'autorotate',
      'arrow-scale': 1.2,
      'opacity': 0.95,
    },
  };
}

/**
 * Build interaction state styles
 */
export function buildInteractionStyles() {
  return [
    {
      selector: 'node:selected',
      style: {
        'border-width': 4,
        'border-color': colors.edgeActive,
        'border-opacity': 1,
        'background-opacity': 1,
      },
    },
    {
      selector: 'edge:selected',
      style: {
        'line-color': colors.edgeActive,
        'target-arrow-color': colors.edgeActive,
        'color': colors.edgeActive,
        'width': 3,
        'opacity': 1,
      },
    },
    {
      selector: 'node:active',
      style: {
        'overlay-opacity': 0.2,
      },
    },
  ];
}

/**
 * Build parent/compound node styles
 */
export function buildParentStyles() {
  return [
    {
      selector: ':parent',
      style: {
        'background-color': Colors.surface(),
        'border-color': Colors.border(),
        'border-width': 1,
        'border-style': 'dashed',
        'label': 'data(label)',
        'text-valign': 'top',
        'text-halign': 'center',
        'text-margin-y': -10,
        'color': Colors.textTertiary(),
        'font-size': 12,
        'font-weight': 600,
        'padding': 24,
        'text-transform': 'uppercase',
      },
    },
    {
      selector: '.collapsed',
      style: {
        'background-color': Colors.border(),
        'border-style': 'dashed',
        'label': 'data(label)',
        'text-valign': 'center',
        'text-halign': 'center',
        'width': 60,
        'height': 60,
      },
    },
  ];
}

/**
 * Build hidden node styles (requirement, adr)
 */
export function buildHiddenNodeStyles() {
  return [
    { selector: 'node[type="requirement"]', style: { display: 'none' } },
    { selector: 'node[type="adr"]', style: { display: 'none' } },
  ];
}

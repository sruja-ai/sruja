// packages/viewer/src/types/layout.ts
import type { CollectionArgument } from 'cytoscape';

/**
 * Typed layout options for Cytoscape layouts
 */
export interface LayoutOptions {
  name: string;
  fit?: boolean;
  padding?: number;
  animate?: boolean;
  animationDuration?: number;
  rankDir?: 'TB' | 'BT' | 'LR' | 'RL';
  nodeSep?: number;
  rankSep?: number;
  randomize?: boolean;
  nodeDimensionsIncludeLabels?: boolean;
  [key: string]: unknown;
}

/**
 * Typed animate options for Cytoscape animations
 */
export interface AnimateOptions {
  fit?: {
    eles: CollectionArgument;
    padding?: number;
  };
  duration?: number;
  [key: string]: unknown;
}

// viewer-core.js entry point
// Exports SrujaViewerCore as a global window object

import { createViewer } from './viewer';

export * from './viewer';
export * from './types';

// Export as global for IIFE bundle
import cytoscape from 'cytoscape';

if (typeof window !== 'undefined') {
  (window as any).SrujaViewerCore = {
    createViewer,
  };
  (window as any).cytoscape = cytoscape;
}

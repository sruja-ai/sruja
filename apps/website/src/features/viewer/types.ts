import type { ArchitectureJSON } from '@sruja/viewer';
import type { Example } from '@sruja/shared';

// Re-export types for use within the viewer feature
export type { ArchitectureJSON, Example };

export interface InteractiveViewerProps {
  initialDsl?: string;
  initialData?: ArchitectureJSON | null;
}

export type PaneType = 'split' | 'editor' | 'diagram' | 'json' | 'preview' | 'markdown';
export type PreviewFormat = 'diagram' | 'json' | 'preview' | 'markdown';

export interface ValidationStatus {
  isValid: boolean;
  errors: number;
  warnings: number;
  lastError?: string;
}

export interface MermaidDiagramProps {
  code: string;
  onExpand?: (svg: string, code: string) => void;
}

export interface ExpandedMermaid {
  svg: string;
  code: string;
}

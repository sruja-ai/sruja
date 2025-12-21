import type { SrujaModelDump } from "@sruja/shared";
import type { LikeC4Model } from "@likec4/core/model";

/**
 * Reference interface for LikeC4Canvas component methods.
 * 
 * Provides programmatic access to canvas operations like export,
 * viewport control, and node focus.
 */
export interface ArchitectureCanvasRef {
  exportAsPNG: (filename?: string) => Promise<void>;
  exportAsSVG: (filename?: string) => Promise<void>;
  getReactFlowInstance: () => null; // Not available in LikeC4
  fitView: (options?: { padding?: number; includeHiddenNodes?: boolean }) => void;
  zoomToSelection: () => void;
  zoomToActualSize: () => void;
  focusNode: (nodeId: string) => void;
}

/**
 * Props for LikeC4Canvas component.
 */
export interface LikeC4CanvasProps {
  model: SrujaModelDump | null;
  dragEnabled?: boolean; // Not used in LikeC4, kept for API compatibility
  viewId?: string;
  /**
   * Node click handler. Note: LikeC4View doesn't support this prop directly.
   * Kept for API compatibility. To handle node clicks, implement custom event handling
   * or use LikeC4's internal event system if available.
   */
  onNodeClick?: (nodeId: string) => void;
  /**
   * Edge click handler. Note: LikeC4View doesn't support this prop directly.
   * Kept for API compatibility. To handle edge clicks, implement custom event handling
   * or use LikeC4's internal event system if available.
   */
  onEdgeClick?: (edgeId: string) => void;
  /**
   * Canvas click handler. Note: LikeC4View doesn't support this prop directly.
   * Kept for API compatibility. To handle canvas clicks, implement custom event handling
   * or use LikeC4's internal event system if available.
   */
  onCanvasClick?: () => void;
  /**
   * @deprecated This prop is no longer used. LikeC4 internal navigation is always disabled.
   * All LikeC4 navigation features (browser, focus mode, details panels) are disabled
   * to allow full control via the designer app's navigation system.
   */
  enableDynamicViewWalk?: boolean;
}

/**
 * C4 hierarchy level types for view generation.
 */
export type C4Level = "L1" | "L2" | "L3";

/**
 * Model conversion result with error handling.
 */
export interface ModelConversionResult {
  model: LikeC4Model<any> | null;
  error: string | null;
  isComputing: boolean;
}


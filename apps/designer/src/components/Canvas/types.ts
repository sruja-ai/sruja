export interface ArchitectureCanvasRef {
  exportAsPNG: (filename?: string) => Promise<void>;
  exportAsSVG: (filename?: string) => Promise<void>;
  fitView: (options?: { padding?: number; includeHiddenNodes?: boolean }) => void;
  zoomToSelection: () => void;
  zoomToActualSize: () => void;
  focusNode: (nodeId: string) => void;
}

export type C4Level = "L1" | "L2" | "L3";

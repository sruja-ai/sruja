// packages/shared/src/diagram/types.ts
// Shared types for diagram layout and rendering

export interface GraphvizCluster {
  /** Bounding box as "llx,lly,urx,ury" string from Graphviz */
  bb?: string;
  /** Array of child node IDs within this cluster */
  children: string[];
}

export interface GraphvizResult {
  nodes: Array<{
    id: string;
    x: number;
    y: number;
    width: number;
    height: number;
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
    /** Bezier spline points from Graphviz - optional, React Flow will route if not provided */
    points?: Array<[number, number]>;
    /** Label position (center x,y) from Graphviz */
    labelPos?: { x: number; y: number };
  }>;
  width: number;
  height: number;
  /** Cluster information for parent-child relationships (parentId -> cluster info) */
  clusters?: Record<string, GraphvizCluster>;
}

export type C4Level = 1 | 2 | 3;
export type C4NodeKind =
  | "system"
  | "container"
  | "component"
  | "person"
  | "external"
  | "actor"
  | "datastore"
  | "queue"
  | "mobile"
  | "webapp"
  | "browser";

export interface C4Node {
  id: string;
  kind: C4NodeKind;
  title: string;
  technology?: string;
  description?: string;
  parentId?: string;
  level: C4Level;
  width: number;
  height: number;
  collapsed?: boolean;
  /** Link to parent system/container for navigation (upstream ID) */
  navigateOnClick?: string;
  metadata?: Record<string, any>;
  [key: string]: unknown; // Index signature for React Flow compatibility
}

export type EdgeType = "straight" | "smoothstep" | "step" | "bezier" | "default";

export interface C4Edge {
  id: string;
  source: string;
  target: string;
  label?: string;
  technology?: string;
  description?: string;
  direction?: "uni" | "bi";
  /** Preferred edge type for rendering */
  edgeType?: EdgeType;
}

export interface ViewState {
  level: C4Level;
  /** The ID of the System (for L2) or Container (for L3) being focused */
  focusNodeId?: string;
  /** Set of node IDs that are manually collapsed by the user (if we support explicit collapse) */
  collapsedNodeIds: Set<string>;
}

export interface LayoutOptions {
  rankdir: "TB" | "LR";
  nodesep: number;
  ranksep: number;
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
}

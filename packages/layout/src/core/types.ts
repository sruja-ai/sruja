/**
 * Core types and interfaces for the new modular layout engine
 * Provides the foundation for all layout operations with immutability and type safety
 */

// =============================================================================
// Basic Geometry Types
// =============================================================================

export interface Point {
  readonly x: number;
  readonly y: number;
}

export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface Size {
  readonly width: number;
  readonly height: number;
}

export interface Bounds {
  readonly minX: number;
  readonly minY: number;
  readonly maxX: number;
  readonly maxY: number;
}

// =============================================================================
// C4 Model Types
// =============================================================================

export type C4Level = "L0" | "L1" | "L2" | "L3";

export interface C4Node {
  readonly id: string;
  readonly type: "Person" | "System" | "Container" | "Component";
  readonly level: C4Level;
  readonly label: string;
  readonly description?: string;
  readonly technology?: string;
  readonly parentId?: string;
  readonly children?: readonly string[];
  readonly metadata?: readonly Record<string, unknown>[];
  readonly collapsed?: boolean;
  readonly hidden?: boolean;
  readonly position?: Point;
  readonly size?: Size;
  readonly layoutPriority?: number;
  readonly tags?: readonly string[];
}

export interface C4Relationship {
  readonly id: string;
  readonly from: string;
  readonly to: string;
  readonly label?: string;
  readonly technology?: string;
  readonly type?: "Dependency" | "Communication" | "DataFlow" | "ControlFlow";
  readonly preferredRoute?: "orthogonal" | "curved" | "splines";
  readonly bidirectional?: boolean;
  readonly metadata?: readonly Record<string, unknown>[];
}

export interface C4Graph {
  readonly nodes: ReadonlyMap<string, C4Node>;
  readonly relationships: readonly C4Relationship[];
}

export interface C4ViewState {
  readonly level: C4Level;
  readonly expandedNodes: ReadonlySet<string>;
  readonly hiddenNodes: ReadonlySet<string>;
  readonly gridSize: number;
  readonly snapToGrid: boolean;
  readonly viewport?: Size;
}

// =============================================================================
// Layout Engine Core Types
// =============================================================================

export interface LayoutNode {
  id: string;
  original: C4Node;
  bbox: Rect;
  contentBox: Rect;
  labelBox: Rect;
  parent?: LayoutNode;
  children: LayoutNode[];
  depth: number;
  level: C4Level;
  collapsed: boolean;
  visible: boolean;
  zIndex: number;
  ports: LayoutPort[];
  constraints: LayoutConstraints;
  metadata: LayoutNodeMetadata;
}

export interface LayoutPort {
  readonly id: string;
  readonly side: "north" | "south" | "east" | "west";
  readonly position: Point;
  readonly angle: number;
  readonly usage: number;
  readonly capacity: number;
}

export interface LayoutEdge {
  readonly id: string;
  readonly original: C4Relationship;
  readonly source: LayoutNode;
  readonly target: LayoutNode;
  readonly points: readonly Point[];
  readonly controlPoints?: readonly Point[];
  readonly segmentTypes: readonly ("line" | "arc" | "orthogonal")[];
  readonly sourcePort?: LayoutPort;
  readonly targetPort?: LayoutPort;
  readonly labelPosition: Point;
  readonly labelAngle: number;
  readonly labelBounds: Rect;
  readonly arrowEnd: Point;
  readonly arrowAngle: number;
  readonly length: number;
  readonly bendCount: number;
  readonly crossesBoundaries: boolean;
  readonly constraints?: EdgeConstraints;
}

export interface LayoutConstraints {
  readonly position?: Point; // Fixed position
  readonly size?: Size; // Fixed size
  readonly minWidth?: number;
  readonly minHeight?: number;
  readonly maxWidth?: number;
  readonly maxHeight?: number;
  readonly sameRank?: readonly string[]; // Nodes that should be on same level
  readonly rankOf?: Map<string, number>; // Explicit rank assignment
  readonly orderHint?: Map<string, number>; // Ordering within rank
  readonly padding?: number; // Minimum padding around node
  readonly aspectRatio?: number; // Preferred aspect ratio
}

export interface EdgeConstraints {
  readonly avoidNodes?: readonly string[]; // Nodes to avoid
  readonly preferredSides?: {
    readonly source?: "north" | "south" | "east" | "west";
    readonly target?: "north" | "south" | "east" | "west";
  };
  readonly minBends?: number;
  readonly maxBends?: number;
  readonly preferredLength?: number;
  readonly bundleWith?: readonly string[]; // Edge IDs to bundle with
}

export interface LayoutNodeMetadata {
  readonly processingOrder: number;
  readonly clusterId?: string;
  readonly weight: number;
  readonly importance: number;
  readonly special: boolean;
  readonly tags: readonly string[];
}

// =============================================================================
// Layout Options and Configuration
// =============================================================================

export interface LayoutOptions {
  readonly strategy: "L0-landscape" | "L1-context" | "L2-container" | "L3-component";
  readonly optimization?: OptimizationOptions;
  readonly edgeRouting: EdgeRoutingOptions;
  readonly spacing: SpacingOptions;
  readonly alignment: AlignmentOptions;
  readonly quality: QualityOptions;
  readonly performance: PerformanceOptions;
  readonly debug: DebugOptions;
}

export interface OptimizationOptions {
  readonly enabled: boolean;
  readonly maxIterations: number;
  readonly tolerance: number;
  readonly aggressiveness: number; // 0-1, how aggressively to optimize
  readonly phases: OptimizationPhase[];
}

export type OptimizationPhase =
  | "overlap-removal"
  | "crossing-minimization"
  | "spacing-optimization"
  | "edge-routing"
  | "label-placement"
  | "containment-enforcement";

export interface EdgeRoutingOptions {
  readonly algorithm: "orthogonal" | "curved" | "splines" | "auto";
  readonly padding: number;
  readonly bendPenalty: number;
  readonly crossingPenalty: number;
  readonly portAssignment: "smart" | "balanced" | "minimal";
  readonly bundling: BundlingOptions;
}

export interface BundlingOptions {
  readonly enabled: boolean;
  readonly angleTolerance: number;
  readonly positionTolerance: number;
  readonly fanOutSpacing: number;
  readonly minBundleSize: number;
}

export interface SpacingOptions {
  readonly nodePadding: number;
  readonly edgePadding: number;
  readonly labelPadding: number;
  readonly parentChildPadding: number;
  readonly siblingSpacing: number;
  readonly levelSpacing: number;
}

export interface AlignmentOptions {
  readonly snapToGrid: boolean;
  readonly gridSize: number;
  readonly alignNodes: boolean;
  readonly alignLabels: boolean;
  readonly horizontalAlignment: "left" | "center" | "right" | "justified";
  readonly verticalAlignment: "top" | "middle" | "bottom" | "justified";
}

export interface QualityOptions {
  readonly targetGrade: "A" | "B" | "C";
  readonly strictMode: boolean;
  readonly validateConstraints: boolean;
  readonly enforceMetrics: boolean;
  readonly earlyExit: boolean; // Stop optimizing when target grade reached
}

export interface PerformanceOptions {
  readonly maxNodes: number;
  readonly maxEdges: number;
  readonly parallelProcessing: boolean;
  readonly spatialIndexing: boolean;
  readonly cacheResults: boolean;
  readonly timeLimit: number; // milliseconds
}

export interface DebugOptions {
  readonly enabled: boolean;
  readonly saveIntermediates: boolean;
  readonly showMetrics: boolean;
  readonly showHeatmap: boolean;
  readonly showPortUsage: boolean;
  readonly verboseLogging: boolean;
}

// =============================================================================
// Layout Pipeline Types
// =============================================================================

export interface LayoutPipeline {
  readonly phases: readonly LayoutPhase[];
  readonly context: LayoutContext;
}

export interface LayoutPhase {
  readonly name: string;
  readonly description: string;
  readonly dependencies: readonly string[];
  readonly execute: (context: LayoutContext) => LayoutContext | Promise<LayoutContext>;
  readonly validate?: (context: LayoutContext) => boolean;
  readonly rollback?: (context: LayoutContext) => LayoutContext;
}

export interface LayoutContext {
  graph: C4Graph;
  view: C4ViewState;
  options: LayoutOptions;
  nodes: Map<string, LayoutNode>;
  edges: Map<string, LayoutEdge>;
  spatialIndex: SpatialIndex;
  qualityScore: QualityScore;
  metrics: LayoutMetrics;
  debug: DebugInfo;
  timestamp: number;
  qualityEvaluator?: (context: LayoutContext) => QualityScore;
  metricsCalculator?: (context: LayoutContext) => LayoutMetrics;
}

// =============================================================================
// Quality and Metrics Types
// =============================================================================

export interface QualityScore {
  readonly grade: "A" | "B" | "C" | "D" | "F";
  readonly score: number; // 0-100
  readonly metrics: Map<string, QualityMetric>;
  readonly violations: QualityViolation[];
  readonly recommendations: readonly QualityRecommendation[];
}

export interface QualityMetric {
  readonly name: string;
  readonly value: number;
  readonly weight: number;
  readonly target: number;
  readonly achieved: boolean;
  readonly impact: "critical" | "important" | "aesthetic";
}

export interface QualityViolation {
  readonly type: QualityViolationType;
  readonly severity: "critical" | "major" | "minor";
  readonly description: string;
  readonly affectedNodes: readonly string[];
  readonly affectedEdges: readonly string[];
  readonly autoFixable: boolean;
  readonly penalty: number;
}

export type QualityViolationType =
  | "node-overlap"
  | "containment-violation"
  | "edge-crossing"
  | "edge-over-node"
  | "clipped-label"
  | "insufficient-spacing"
  | "poor-aspect-ratio"
  | "low-viewport-utilization"
  | "excessive-edge-length"
  | "too-many-bends"
  | "port-congestion";

export interface QualityRecommendation {
  readonly action: string;
  readonly description: string;
  readonly expectedImprovement: number;
  readonly effort: "low" | "medium" | "high";
  readonly applicablePhases: readonly OptimizationPhase[];
}

export interface LayoutMetrics {
  readonly totalNodes: number;
  readonly visibleNodes: number;
  readonly totalEdges: number;
  readonly edgeCrossings: number;
  readonly totalEdgeLength: number;
  readonly averageEdgeLength: number;
  readonly edgeBends: number;
  readonly averageBends: number;
  readonly overlaps: number;
  readonly containmentViolations: number;
  readonly aspectRatio: number;
  readonly coverage: number; // viewport utilization
  readonly uniformity: number;
  readonly balance: number;
  readonly compactness: number;
  readonly processingTime: number;
  readonly memoryUsage: number;
}

// =============================================================================
// Spatial Indexing
// =============================================================================

export interface SpatialIndex {
  readonly type: "quadtree" | "rtree" | "grid";
  readonly bounds: Bounds;
  readonly insert: (node: LayoutNode) => void;
  readonly remove: (node: LayoutNode) => void;
  readonly query: (bounds: Bounds) => readonly LayoutNode[];
  readonly queryPoint: (point: Point) => readonly LayoutNode[];
  readonly nearest: (point: Point, maxDistance?: number) => readonly LayoutNode[];
  readonly clear: () => void;
  readonly size: number;
}

// =============================================================================
// Debug and Diagnostics
// =============================================================================

export interface DebugInfo {
  phases: PhaseDebugInfo[];
  warnings: string[];
  errors: string[];
  metrics: Map<string, unknown>;
  heatMap?: Map<string, number>; // node ID -> badness score
  intermediateResults?: Map<string, unknown>;
}

export interface PhaseDebugInfo {
  readonly name: string;
  readonly duration: number;
  readonly nodesProcessed: number;
  readonly improvements: Map<string, number>; // metric -> improvement amount
  readonly issues: readonly string[];
}

// =============================================================================
// Layout Result Types
// =============================================================================

export interface LayoutResult {
  readonly nodes: ReadonlyMap<string, LayoutNode>;
  readonly edges: readonly LayoutEdge[];
  readonly bbox: Rect;
  readonly center: Point;
  readonly metrics: LayoutMetrics;
  readonly quality: QualityScore;
  readonly debug: DebugInfo;
  readonly timestamp: number;
}

export interface LayoutValidationError {
  readonly type: "error" | "warning";
  readonly code: string;
  readonly message: string;
  readonly nodeIds?: readonly string[];
  readonly edgeIds?: readonly string[];
  readonly fixable: boolean;
  readonly suggestion?: string;
}

// =============================================================================
// Utility Types
// =============================================================================

export type Maybe<T> = T | null | undefined;
export type NonNullable<T> = T extends null | undefined ? never : T;

export interface ProgressCallback {
  (phase: string, progress: number, message?: string): void;
}

export interface LayoutComparator {
  (a: LayoutResult, b: LayoutResult): number;
}

export type LayoutPlugin = {
  readonly name: string;
  readonly version: string;
  readonly phases: readonly LayoutPhase[];
  readonly initialize?: (options: LayoutOptions) => void;
  readonly cleanup?: () => void;
};

// =============================================================================
// Default Presets
// =============================================================================

export const DefaultLayoutOptions: LayoutOptions = {
  strategy: "L1-context",
  optimization: {
    enabled: true,
    maxIterations: 100,
    tolerance: 0.001,
    aggressiveness: 0.7,
    phases: [
      "overlap-removal",
      "crossing-minimization",
      "spacing-optimization",
      "edge-routing",
      "label-placement",
      "containment-enforcement",
    ],
  },
  edgeRouting: {
    algorithm: "auto",
    padding: 20,
    bendPenalty: 1.0,
    crossingPenalty: 2.0,
    portAssignment: "smart",
    bundling: {
      enabled: true,
      angleTolerance: 5,
      positionTolerance: 20,
      fanOutSpacing: 15,
      minBundleSize: 2,
    },
  },
  spacing: {
    nodePadding: 30,
    edgePadding: 15,
    labelPadding: 25,
    parentChildPadding: 50,
    siblingSpacing: 40,
    levelSpacing: 80,
  },
  alignment: {
    snapToGrid: false,
    gridSize: 10,
    alignNodes: true,
    alignLabels: true,
    horizontalAlignment: "center",
    verticalAlignment: "middle",
  },
  quality: {
    targetGrade: "B",
    strictMode: false,
    validateConstraints: true,
    enforceMetrics: true,
    earlyExit: true,
  },
  performance: {
    maxNodes: 1000,
    maxEdges: 2000,
    parallelProcessing: false,
    spatialIndexing: true,
    cacheResults: true,
    timeLimit: 5000,
  },
  debug: {
    enabled: false,
    saveIntermediates: false,
    showMetrics: false,
    showHeatmap: false,
    showPortUsage: false,
    verboseLogging: false,
  },
};

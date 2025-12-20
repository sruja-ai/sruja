export * from "./brand";
export * from "./types";
export * from "./c4-model";
export * from "./c4-view";
// Legacy c4-layout removed - use modular engine via applySrujaLayout
export * from "./c4-options";
export * from "./theme";
export * from "./plugin";
export * from "./text";
export * from "./tree/buildTree";
export * from "./algorithms/sizing";
export * from "./algorithms/flowLayout";
export * from "./algorithms/hierarchy";
export * from "./algorithms/coordinates";
export * from "./algorithms/edge-router";
export * from "./algorithms/overlap";
export * from "./utils/immutability";
export { MockTextMeasurer2 } from "./utils/text-measurer";
export * from "./utils/cached-text-measurer";
export * from "./utils/canvas-text-measurer";
export * from "./presets/default";
export * from "./constants";
export * from "./algorithms/c4-level-layouts";
export * from "./algorithms/clutter-detection";
export * from "./algorithms/self-optimizer";
export * from "./algorithms/spatial-index";
export * from "./algorithms/unified-router";
export * from "./algorithms/transitions";
export * from "./algorithms/grid-layout";
export * from "./algorithms/l0-layout";
export type { ArchitectureJSON } from "./bridge";
export * from "./algorithms/label-placer";
export * from "./algorithms/edge-bundler";
export * from "./algorithms/optimizer";
export * from "./algorithms/viewport-expander";

// Layout Helpers (extracted utilities)
export * from "./layout-helpers";

// Edge Routing Helpers
export * from "./edge-routing-helpers";

// Edge Routing Pipeline
export * from "./edge-routing-pipeline";

// Bridge Adapter (for backward compatibility and feature flag control)
export {
  applySrujaLayout,
  isNewLayoutEngineEnabled,
  enableNewLayoutEngine,
  disableNewLayoutEngine,
  testNewLayoutEngine,
} from "./bridge";




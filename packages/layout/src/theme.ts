import type { C4Kind, C4Level } from "./c4-model";

// ============================================================================
// CORE THEME TYPES
// ============================================================================

export interface NodeColorScheme {
  fill: string;
  stroke: string;
  text: string;
  /** Optional gradient for premium look */
  gradient?: { from: string; to: string };
}

export interface EdgeColorScheme {
  default: string;
  sync: string;
  async: string;
  database: string;
  /** External system connections */
  external: string;
}

export interface VisualEffects {
  shadowEnabled: boolean;
  shadowColor: string;
  shadowBlur: number;
  shadowOffset: { x: number; y: number };
  /** Border radius for containers */
  borderRadius: number;
  /** Opacity for external/grayed out elements */
  fadedOpacity: number;
}

export interface C4Theme {
  /** Theme identifier */
  name: string;

  /** Base typography */
  fontFamily: string;
  fontSize: Partial<Record<C4Kind | C4Level, number>>;
  fontWeight: Partial<Record<C4Kind | C4Level, string>>;

  /** Spacing */
  padding: Partial<Record<C4Kind | C4Level, number>>;
  margin: Partial<Record<C4Kind | C4Level, number>>;

  /** Base colors */
  textColor: string;
  backgroundColor: string;

  /** Semantic node colors by C4 kind */
  nodeColors: Partial<Record<C4Kind, NodeColorScheme>>;

  /** Edge colors by interaction type */
  edgeColors: EdgeColorScheme;

  /** Visual effects (shadows, borders) */
  effects: VisualEffects;
}

// ============================================================================
// DEFAULT THEME (Legacy compatibility)
// ============================================================================

export const DefaultTheme: C4Theme = {
  name: "default",
  fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  fontSize: {
    Person: 14,
    SoftwareSystem: 16,
    Container: 14,
    Component: 12,
    Database: 14,
    landscape: 18,
    context: 16,
    container: 14,
    component: 12,
    deployment: 14,
  },
  fontWeight: {
    Person: "bold",
    SoftwareSystem: "bold",
    Container: "600",
    Component: "normal",
    Database: "600",
  },
  padding: { Person: 16, SoftwareSystem: 24, Container: 20, Component: 12, Database: 16 },
  margin: { Person: 20, SoftwareSystem: 30, Container: 20, Component: 15, Database: 20 },
  textColor: "#1a1a1a",
  backgroundColor: "#ffffff",
  nodeColors: {},
  edgeColors: {
    default: "#666666",
    sync: "#1168bd",
    async: "#438dd5",
    database: "#2d882d",
    external: "#999999",
  },
  effects: {
    shadowEnabled: false,
    shadowColor: "rgba(0,0,0,0.15)",
    shadowBlur: 8,
    shadowOffset: { x: 2, y: 4 },
    borderRadius: 8,
    fadedOpacity: 0.6,
  },
};

// ============================================================================
// C4 LIGHT THEME - Classic C4 Model colors
// ============================================================================

export const C4LightTheme: C4Theme = {
  name: "c4-light",
  fontFamily: "Inter, system-ui, -apple-system, sans-serif",
  fontSize: {
    Person: 14,
    SoftwareSystem: 16,
    Container: 14,
    Component: 13,
    Database: 14,
    landscape: 18,
    context: 16,
    container: 14,
    component: 12,
    deployment: 14,
  },
  fontWeight: {
    Person: "600",
    SoftwareSystem: "700",
    Container: "600",
    Component: "500",
    Database: "600",
  },
  padding: { Person: 16, SoftwareSystem: 24, Container: 20, Component: 14, Database: 16 },
  margin: { Person: 24, SoftwareSystem: 32, Container: 24, Component: 18, Database: 24 },
  textColor: "#1e293b",
  backgroundColor: "#f8fafc",
  nodeColors: {
    Person: {
      fill: "#08427b",
      stroke: "#052e56",
      text: "#ffffff",
      gradient: { from: "#1168bd", to: "#08427b" },
    },
    SoftwareSystem: {
      fill: "#1168bd",
      stroke: "#0d5199",
      text: "#ffffff",
      gradient: { from: "#438dd5", to: "#1168bd" },
    },
    Container: {
      fill: "#438dd5",
      stroke: "#3478b8",
      text: "#ffffff",
    },
    Component: {
      fill: "#85bbf0",
      stroke: "#6ba8e5",
      text: "#1e293b",
    },
    Database: {
      fill: "#2d882d",
      stroke: "#236b23",
      text: "#ffffff",
    },
    Queue: {
      fill: "#f97316",
      stroke: "#ea580c",
      text: "#ffffff",
    },
    Cache: {
      fill: "#8b5cf6",
      stroke: "#7c3aed",
      text: "#ffffff",
    },
    ExternalSystem: {
      fill: "#999999",
      stroke: "#777777",
      text: "#ffffff",
    },
    ExternalContainer: {
      fill: "#aaaaaa",
      stroke: "#888888",
      text: "#1e293b",
    },
    DeploymentNode: {
      fill: "#ffffff",
      stroke: "#1168bd",
      text: "#1e293b",
    },
  },
  edgeColors: {
    default: "#475569",
    sync: "#1168bd",
    async: "#f97316",
    database: "#2d882d",
    external: "#94a3b8",
  },
  effects: {
    shadowEnabled: true,
    shadowColor: "rgba(15, 23, 42, 0.12)",
    shadowBlur: 12,
    shadowOffset: { x: 0, y: 4 },
    borderRadius: 10,
    fadedOpacity: 0.6,
  },
};

// ============================================================================
// C4 DARK THEME - High contrast dark mode
// ============================================================================

export const C4DarkTheme: C4Theme = {
  name: "c4-dark",
  fontFamily: "Inter, system-ui, -apple-system, sans-serif",
  fontSize: {
    Person: 14,
    SoftwareSystem: 16,
    Container: 14,
    Component: 13,
    Database: 14,
    landscape: 18,
    context: 16,
    container: 14,
    component: 12,
    deployment: 14,
  },
  fontWeight: {
    Person: "600",
    SoftwareSystem: "700",
    Container: "600",
    Component: "500",
    Database: "600",
  },
  padding: { Person: 16, SoftwareSystem: 24, Container: 20, Component: 14, Database: 16 },
  margin: { Person: 24, SoftwareSystem: 32, Container: 24, Component: 18, Database: 24 },
  textColor: "#e2e8f0",
  backgroundColor: "#0f172a",
  nodeColors: {
    Person: {
      fill: "#1e40af",
      stroke: "#3b82f6",
      text: "#ffffff",
      gradient: { from: "#3b82f6", to: "#1e40af" },
    },
    SoftwareSystem: {
      fill: "#1d4ed8",
      stroke: "#60a5fa",
      text: "#ffffff",
      gradient: { from: "#60a5fa", to: "#1d4ed8" },
    },
    Container: {
      fill: "#2563eb",
      stroke: "#93c5fd",
      text: "#ffffff",
    },
    Component: {
      fill: "#3b82f6",
      stroke: "#bfdbfe",
      text: "#ffffff",
    },
    Database: {
      fill: "#15803d",
      stroke: "#4ade80",
      text: "#ffffff",
    },
    Queue: {
      fill: "#c2410c",
      stroke: "#fb923c",
      text: "#ffffff",
    },
    Cache: {
      fill: "#7c3aed",
      stroke: "#a78bfa",
      text: "#ffffff",
    },
    ExternalSystem: {
      fill: "#475569",
      stroke: "#94a3b8",
      text: "#e2e8f0",
    },
    DeploymentNode: {
      fill: "#1e293b",
      stroke: "#60a5fa",
      text: "#e2e8f0",
    },
  },
  edgeColors: {
    default: "#94a3b8",
    sync: "#60a5fa",
    async: "#fb923c",
    database: "#4ade80",
    external: "#64748b",
  },
  effects: {
    shadowEnabled: true,
    shadowColor: "rgba(0, 0, 0, 0.4)",
    shadowBlur: 16,
    shadowOffset: { x: 0, y: 6 },
    borderRadius: 10,
    fadedOpacity: 0.5,
  },
};

// ============================================================================
// STRUCTURIZR THEME - Matches Structurizr's styling
// ============================================================================

export const StructurizrTheme: C4Theme = {
  name: "structurizr",
  fontFamily: '"Open Sans", system-ui, sans-serif',
  fontSize: {
    Person: 14,
    SoftwareSystem: 15,
    Container: 14,
    Component: 12,
    Database: 14,
    landscape: 18,
    context: 16,
    container: 14,
    component: 12,
    deployment: 14,
  },
  fontWeight: {
    Person: "600",
    SoftwareSystem: "600",
    Container: "600",
    Component: "normal",
    Database: "600",
  },
  padding: { Person: 15, SoftwareSystem: 20, Container: 18, Component: 12, Database: 15 },
  margin: { Person: 20, SoftwareSystem: 28, Container: 22, Component: 16, Database: 20 },
  textColor: "#000000",
  backgroundColor: "#ffffff",
  nodeColors: {
    Person: {
      fill: "#08427b",
      stroke: "#073b6e",
      text: "#ffffff",
    },
    SoftwareSystem: {
      fill: "#1168bd",
      stroke: "#0e5aa7",
      text: "#ffffff",
    },
    Container: {
      fill: "#438dd5",
      stroke: "#3a7bbc",
      text: "#ffffff",
    },
    Component: {
      fill: "#85bbf0",
      stroke: "#74a9dc",
      text: "#000000",
    },
    Database: {
      fill: "#438dd5",
      stroke: "#3a7bbc",
      text: "#ffffff",
    },
    ExternalSystem: {
      fill: "#999999",
      stroke: "#8a8a8a",
      text: "#ffffff",
    },
    DeploymentNode: {
      fill: "#ffffff",
      stroke: "#888888",
      text: "#000000",
    },
  },
  edgeColors: {
    default: "#707070",
    sync: "#707070",
    async: "#707070",
    database: "#707070",
    external: "#aaaaaa",
  },
  effects: {
    shadowEnabled: false,
    shadowColor: "rgba(0,0,0,0.1)",
    shadowBlur: 4,
    shadowOffset: { x: 1, y: 2 },
    borderRadius: 5,
    fadedOpacity: 0.7,
  },
};

// ============================================================================
// THEME UTILITIES
// ============================================================================

/**
 * Get node colors for a specific C4 kind, with fallback to defaults
 */
export function getNodeColors(theme: C4Theme, kind: C4Kind): NodeColorScheme {
  const custom = theme.nodeColors[kind];
  if (custom) return custom;

  // Fallback defaults
  return {
    fill: "#cccccc",
    stroke: "#999999",
    text: theme.textColor,
  };
}

/**
 * Get edge color for a specific interaction type
 */
export function getEdgeColor(theme: C4Theme, interaction?: "sync" | "async" | "event"): string {
  if (!interaction) return theme.edgeColors.default;
  if (interaction === "event") return theme.edgeColors.async;
  return theme.edgeColors[interaction] ?? theme.edgeColors.default;
}

/**
 * Available theme presets
 */
export const ThemePresets = {
  default: DefaultTheme,
  "c4-light": C4LightTheme,
  "c4-dark": C4DarkTheme,
  structurizr: StructurizrTheme,
} as const;

export type ThemePresetName = keyof typeof ThemePresets;

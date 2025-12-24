// C4 Model Color Constants
// Following Simon Brown's C4 model conventions
// Using BORDER-ONLY style (no fills) like official C4 diagrams

export const C4_COLORS = {
  // Person - Blue
  person: {
    bg: "transparent", // No fill
    border: "#08427B", // Dark blue border
    text: "#08427B", // Match border
  },
  // Internal System - Blue
  systemInternal: {
    bg: "transparent", // No fill
    border: "#1168BD", // Blue border
    text: "#1168BD", // Match border
  },
  // External System - Gray
  systemExternal: {
    bg: "transparent", // No fill
    border: "#999999", // Gray border
    text: "#666666", // Darker gray text
  },
  // Container - Lighter blue
  container: {
    bg: "transparent", // No fill
    border: "#438DD5", // Light blue border
    text: "#438DD5", // Match border
  },
  // Component - Lightest blue
  component: {
    bg: "transparent", // No fill
    border: "#85BBF0", // Very light blue border
    text: "#5A8FC0", // Slightly darker for readability
  },
  // Database - Container blue
  datastore: {
    bg: "transparent", // No fill
    border: "#438DD5", // Light blue border
    text: "#438DD5", // Match border
  },
  // Queue - Container blue
  queue: {
    bg: "transparent", // No fill
    border: "#438DD5", // Light blue border
    text: "#438DD5", // Match border
  },
} as const;

// FAANG-Quality Premium Colors - Semantic colors per node type
export const PREMIUM_COLORS = {
  person: {
    light: {
      bg: "#EFF6FF", // Blue-50
      border: "#1E40AF", // Blue-800
      text: "#1E40AF",
      gradient: "linear-gradient(135deg, #EFF6FF 0%, #DBEAFE 100%)",
      shadow: "0 4px 12px rgba(30, 64, 175, 0.15)",
      glow: "0 0 0 3px rgba(59, 130, 246, 0.3)",
    },
    dark: {
      bg: "#1E3A5F",
      border: "#60A5FA",
      text: "#F0F9FF",
      gradient: "linear-gradient(135deg, #1E3A5F 0%, #172554 100%)",
      shadow: "0 4px 12px rgba(96, 165, 250, 0.2)",
      glow: "0 0 0 3px rgba(96, 165, 250, 0.4)",
    },
  },
  system: {
    light: {
      bg: "#F0FDF4", // Green-50
      border: "#166534", // Green-800
      text: "#166534",
      gradient: "linear-gradient(135deg, #F0FDF4 0%, #DCFCE7 100%)",
      shadow: "0 4px 12px rgba(22, 101, 52, 0.15)",
      glow: "0 0 0 3px rgba(34, 197, 94, 0.3)",
    },
    dark: {
      bg: "#14532D",
      border: "#4ADE80",
      text: "#F0FDF4",
      gradient: "linear-gradient(135deg, #14532D 0%, #052E16 100%)",
      shadow: "0 4px 12px rgba(74, 222, 128, 0.2)",
      glow: "0 0 0 3px rgba(74, 222, 128, 0.4)",
    },
  },
  systemExternal: {
    light: {
      bg: "#FEF3C7", // Amber-100
      border: "#B45309", // Amber-700
      text: "#92400E",
      gradient: "linear-gradient(135deg, #FEF3C7 0%, #FDE68A 100%)",
      shadow: "0 4px 12px rgba(180, 83, 9, 0.15)",
      glow: "0 0 0 3px rgba(245, 158, 11, 0.3)",
    },
    dark: {
      bg: "#451A03",
      border: "#FCD34D",
      text: "#FFFBEB",
      gradient: "linear-gradient(135deg, #451A03 0%, #78350F 100%)",
      shadow: "0 4px 12px rgba(252, 211, 77, 0.2)",
      glow: "0 0 0 3px rgba(252, 211, 77, 0.4)",
    },
  },
  container: {
    light: {
      bg: "#EDE9FE", // Violet-100
      border: "#6D28D9", // Violet-700
      text: "#5B21B6",
      gradient: "linear-gradient(135deg, #EDE9FE 0%, #DDD6FE 100%)",
      shadow: "0 4px 12px rgba(109, 40, 217, 0.15)",
      glow: "0 0 0 3px rgba(139, 92, 246, 0.3)",
    },
    dark: {
      bg: "#4C1D95",
      border: "#A78BFA",
      text: "#F5F3FF",
      gradient: "linear-gradient(135deg, #4C1D95 0%, #3B0764 100%)",
      shadow: "0 4px 12px rgba(167, 139, 250, 0.2)",
      glow: "0 0 0 3px rgba(167, 139, 250, 0.4)",
    },
  },
  component: {
    light: {
      bg: "#FDF2F8", // Pink-50
      border: "#BE185D", // Pink-700
      text: "#9D174D",
      gradient: "linear-gradient(135deg, #FDF2F8 0%, #FCE7F3 100%)",
      shadow: "0 4px 12px rgba(190, 24, 93, 0.15)",
      glow: "0 0 0 3px rgba(236, 72, 153, 0.3)",
    },
    dark: {
      bg: "#831843",
      border: "#F472B6",
      text: "#FDF2F8",
      gradient: "linear-gradient(135deg, #831843 0%, #500724 100%)",
      shadow: "0 4px 12px rgba(244, 114, 182, 0.2)",
      glow: "0 0 0 3px rgba(244, 114, 182, 0.4)",
    },
  },
  datastore: {
    light: {
      bg: "#ECFEFF", // Cyan-50
      border: "#0E7490", // Cyan-700
      text: "#155E75",
      gradient: "linear-gradient(135deg, #ECFEFF 0%, #CFFAFE 100%)",
      shadow: "0 4px 12px rgba(14, 116, 144, 0.15)",
      glow: "0 0 0 3px rgba(6, 182, 212, 0.3)",
    },
    dark: {
      bg: "#155E75",
      border: "#22D3EE",
      text: "#ECFEFF",
      gradient: "linear-gradient(135deg, #155E75 0%, #164E63 100%)",
      shadow: "0 4px 12px rgba(34, 211, 238, 0.2)",
      glow: "0 0 0 3px rgba(34, 211, 238, 0.4)",
    },
  },
  queue: {
    light: {
      bg: "#FEF2F2", // Red-50
      border: "#B91C1C", // Red-700
      text: "#991B1B",
      gradient: "linear-gradient(135deg, #FEF2F2 0%, #FEE2E2 100%)",
      shadow: "0 4px 12px rgba(185, 28, 28, 0.15)",
      glow: "0 0 0 3px rgba(239, 68, 68, 0.3)",
    },
    dark: {
      bg: "#7F1D1D",
      border: "#F87171",
      text: "#FEF2F2",
      gradient: "linear-gradient(135deg, #7F1D1D 0%, #450A0A 100%)",
      shadow: "0 4px 12px rgba(248, 113, 113, 0.2)",
      glow: "0 0 0 3px rgba(248, 113, 113, 0.4)",
    },
  },
} as const;

// Alternative FILLED style (legacy)
export const C4_COLORS_FILLED = {
  person: {
    bg: "#08427B",
    border: "#052E56",
    text: "#FFFFFF",
  },
  systemInternal: {
    bg: "#1168BD",
    border: "#0B4884",
    text: "#FFFFFF",
  },
  systemExternal: {
    bg: "#999999",
    border: "#6B6B6B",
    text: "#FFFFFF",
  },
  container: {
    bg: "#438DD5",
    border: "#2E6295",
    text: "#FFFFFF",
  },
  component: {
    bg: "#85BBF0",
    border: "#5D99C9",
    text: "#000000",
  },
  datastore: {
    bg: "#438DD5",
    border: "#2E6295",
    text: "#FFFFFF",
  },
  queue: {
    bg: "#438DD5",
    border: "#2E6295",
    text: "#FFFFFF",
  },
} as const;

export const EDGE_STYLES = {
  default: {
    stroke: "#707070",
    strokeWidth: 1.5,
  },
  selected: {
    stroke: "#333333",
    strokeWidth: 2,
  },
  animated: {
    stroke: "#1168BD",
    strokeWidth: 2,
    strokeDasharray: "5,5",
  },
  dimmed: {
    stroke: "#CCCCCC",
    strokeWidth: 1,
    opacity: 0.5,
  },
} as const;

// Style mode switch
export type ColorStyle = "outline" | "filled";
let currentStyle: ColorStyle = "filled";

export function setColorStyle(style: ColorStyle) {
  currentStyle = style;
}

export function getColorStyle(): ColorStyle {
  return currentStyle;
}

// Detect current theme (light or dark)
function getCurrentTheme(): "light" | "dark" {
  if (typeof document === "undefined") return "light";
  const themeAttr = document.documentElement.getAttribute("data-theme");
  return themeAttr === "dark" ? "dark" : "light";
}

// Get color scheme based on node type
export function getNodeColors(type: string, isExternal?: boolean) {
  const isDark = getCurrentTheme() === "dark";
  const colors = currentStyle === "outline" ? C4_COLORS : C4_COLORS_FILLED;

  let nodeColors = (() => {
    switch (type) {
      case "person":
        return colors.person;
      case "system":
        return isExternal ? colors.systemExternal : colors.systemInternal;
      case "container":
        return colors.container;
      case "component":
        return colors.component;
      case "datastore":
        return colors.datastore;
      case "queue":
        return colors.queue;
      default:
        return colors.systemInternal;
    }
  })();

  // Special handling for person nodes: icon is filled with navy blue, so text needs contrast
  if (type === "person") {
    if (currentStyle === "outline") {
      // In outline mode, icon is navy blue. Use dark text with white shadow for visibility
      // against both light background and navy blue icon
      return {
        ...nodeColors,
        text: "#08427B", // Keep navy blue text, but we'll add text shadow in component
      };
    } else if (currentStyle === "filled" && !isDark) {
      // In filled mode + light theme, background is navy blue, use white text
      return {
        ...nodeColors,
        text: "#FFFFFF", // White text for visibility against navy blue background
      };
    }
    // In filled mode + dark theme, white text is already correct
    return nodeColors;
  }

  // In filled mode, adjust text color for light theme to ensure visibility
  if (currentStyle === "filled" && !isDark) {
    // For light theme, use darker text colors for better contrast
    // TypeScript: nodeColors always has text property from the color definitions
    const textColor = ((nodeColors as any).border || (nodeColors as any).text) as string;
    return {
      ...nodeColors,
      text: textColor, // Use border color (darker) for text in light theme
    };
  }

  // In outline mode, ensure text is visible in dark theme
  if (currentStyle === "outline" && isDark) {
    // Lighten the text color for dark theme
    if (isDark && type !== "person") {
      // For non-person nodes, use a lighter version of the border color
      return {
        ...nodeColors,
        text: "#cbd5e1", // slate-300 for better visibility in dark theme
      };
    }
  }

  return nodeColors;
}

// Get FAANG-quality premium colors with gradients and shadows
export function getPremiumNodeColors(type: string, isExternal?: boolean) {
  const isDark = getCurrentTheme() === "dark";
  const theme = isDark ? "dark" : "light";

  // Map type to premium colors
  const getPremiumStyle = () => {
    switch (type) {
      case "person":
      case "actor":
        return PREMIUM_COLORS.person[theme];
      case "system":
        return isExternal ? PREMIUM_COLORS.systemExternal[theme] : PREMIUM_COLORS.system[theme];
      case "container":
        return PREMIUM_COLORS.container[theme];
      case "component":
        return PREMIUM_COLORS.component[theme];
      case "datastore":
        return PREMIUM_COLORS.datastore[theme];
      case "queue":
        return PREMIUM_COLORS.queue[theme];
      default:
        return PREMIUM_COLORS.system[theme];
    }
  };

  return getPremiumStyle();
}

export function getTagStyles(tags?: string[]) {
  const style: Record<string, any> = {};
  if (!tags || tags.length === 0) return style;
  const t = new Set(tags.map((x) => x.toLowerCase()));
  if (t.has("deprecated")) style.borderStyle = "dashed";
  if (t.has("critical")) style.boxShadow = "0 0 0 3px #ef4444";
  if (t.has("experimental")) style.boxShadow = "0 0 0 2px #f59e0b";
  return style;
}

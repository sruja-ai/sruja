// packages/ui/src/utils/mermaidTheme.ts
// Shared Mermaid theme aligned with Sruja design system (theme.ts / styles.css).
// Uses theme "base" + themeVariables for readable diagrams in light and dark mode.

/** Config passed to mermaid.initialize(); theme "base" + themeVariables. */
export type MermaidThemeConfig = Parameters<typeof import("mermaid").default.initialize>[0];

/** Detect if the app is in dark mode (matches ThemeProvider / design system). */
export function getIsDark(): boolean {
  if (typeof document === "undefined") return false;
  const root = document.documentElement;
  return root.getAttribute("data-theme") === "dark" || root.classList.contains("dark");
}

/**
 * Mermaid config aligned with Sruja design system.
 * Light: surface #f8fafc, text #0f172a, primary violet #7c3aed, high-contrast nodes.
 * Dark: surface #1e293b, text #f1f5f9, primary #a78bfa, visible edges and labels.
 */
export function getMermaidConfig(isDark: boolean): MermaidThemeConfig {
  if (isDark) {
    return {
      startOnLoad: false,
      theme: "base",
      securityLevel: "loose",
      fontFamily: "inherit",
      flowchart: {
        htmlLabels: true,
        useMaxWidth: true,
        wrappingWidth: 320,
        nodeSpacing: 50,
        rankSpacing: 50,
        padding: 15,
      },
      themeVariables: {
        darkMode: true,
        background: "#1e293b",
        primaryColor: "#334155",
        primaryTextColor: "#f1f5f9",
        primaryBorderColor: "#64748b",
        secondaryColor: "#475569",
        secondaryBorderColor: "#64748b",
        secondaryTextColor: "#f1f5f9",
        tertiaryColor: "#475569",
        tertiaryBorderColor: "#94a3b8",
        tertiaryTextColor: "#cbd5e1",
        lineColor: "#94a3b8",
        mainBkg: "#334155",
        noteBkgColor: "#475569",
        noteTextColor: "#f1f5f9",
        noteBorderColor: "#64748b",
        nodeTextColor: "#f1f5f9",
        titleColor: "#f1f5f9",
        actorBkg: "#334155",
        actorBorder: "#64748b",
        actorTextColor: "#f1f5f9",
        signalColor: "#94a3b8",
        signalTextColor: "#f1f5f9",
        activationBorderColor: "#64748b",
        activationBkgColor: "#475569",
        sequenceNumberColor: "#94a3b8",
        edgeLabelBackground: "#e2e8f0",
        textColor: "#0f172a",
      },
    };
  }
  return {
    startOnLoad: false,
    theme: "base",
    securityLevel: "loose",
    fontFamily: "inherit",
    flowchart: {
      htmlLabels: true,
      useMaxWidth: true,
      wrappingWidth: 320,
      nodeSpacing: 50,
      rankSpacing: 50,
      padding: 15,
    },
    themeVariables: {
      darkMode: false,
      background: "#f8fafc",
      primaryColor: "#e9e5ff",
      primaryTextColor: "#0f172a",
      primaryBorderColor: "#7c3aed",
      secondaryColor: "#f1f5f9",
      secondaryBorderColor: "#cbd5e1",
      secondaryTextColor: "#334155",
      tertiaryColor: "#e2e8f0",
      tertiaryBorderColor: "#cbd5e1",
      tertiaryTextColor: "#475569",
      lineColor: "#475569",
      textColor: "#0f172a",
      mainBkg: "#e9e5ff",
      noteBkgColor: "#fef3c7",
      noteTextColor: "#0f172a",
      noteBorderColor: "#e2e8f0",
      actorBkg: "#e9e5ff",
      actorBorder: "#7c3aed",
      actorTextColor: "#0f172a",
      signalColor: "#475569",
      signalTextColor: "#0f172a",
      activationBorderColor: "#a78bfa",
      activationBkgColor: "#e9e5ff",
      sequenceNumberColor: "#475569",
    },
  };
}

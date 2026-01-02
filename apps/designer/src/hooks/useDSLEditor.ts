import { useMemo, useCallback } from "react";
import { useTheme } from "@sruja/ui";
import type * as monacoTypes from "monaco-editor";

/**
 * Hook to configure Monaco editor for DSL editing.
 *
 * Provides theme configuration and editor mount handler.
 */
export function useDSLEditor(_dslSource: string | null) {
  const { mode } = useTheme();

  // Determine Monaco theme based on UI theme
  const monacoTheme = useMemo((): "vs" | "vs-dark" | "hc-black" => {
    return mode === "dark" ||
      (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)
      ? "vs-dark"
      : "vs";
  }, [mode]);

  // Handle editor mount - can be used for additional configuration
  const handleEditorDidMount = useCallback(
    (
      _monaco: typeof import("monaco-editor"),
      _editor: monacoTypes.editor.IStandaloneCodeEditor
    ) => {
      // Additional editor configuration can be added here
      // For example: custom keybindings, language features, etc.
    },
    []
  );

  return {
    monacoTheme,
    handleEditorDidMount,
  };
}

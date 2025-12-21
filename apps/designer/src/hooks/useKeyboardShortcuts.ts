// apps/playground/src/hooks/useKeyboardShortcuts.ts
import { useEffect, useCallback } from "react";

export interface KeyboardShortcut {
  key: string;
  ctrlKey?: boolean;
  metaKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  action: (() => void) | (() => Promise<void>);
  description: string;
  preventDefault?: boolean;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[]) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
        const ctrlOrCmd = isMac ? event.metaKey : event.ctrlKey;

        // Check if this shortcut matches
        const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();
        const ctrlMatches = shortcut.ctrlKey !== undefined ? ctrlOrCmd === shortcut.ctrlKey : true;
        const metaMatches =
          shortcut.metaKey !== undefined ? event.metaKey === shortcut.metaKey : true;
        const shiftMatches =
          shortcut.shiftKey !== undefined ? event.shiftKey === shortcut.shiftKey : true;
        const altMatches = shortcut.altKey !== undefined ? event.altKey === shortcut.altKey : true;

        if (keyMatches && ctrlMatches && metaMatches && shiftMatches && altMatches) {
          // Don't trigger if user is typing in an input/textarea
          const target = event.target as HTMLElement;
          if (
            target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable
          ) {
            // Allow some shortcuts even in inputs (like Cmd+S to save)
            if (shortcut.key !== "s" && shortcut.key !== "o") {
              continue;
            }
          }

          // For copy/paste shortcuts, check if text is selected - if so, allow default behavior
          if ((shortcut.key === "c" || shortcut.key === "v" || shortcut.key === "x") && ctrlOrCmd) {
            const selection = window.getSelection();
            if (selection && selection.toString().length > 0 && shortcut.key === "c") {
              // User has text selected and wants to copy - allow default behavior
              continue;
            }
            // For paste, also check if we're in an input/textarea
            if (shortcut.key === "v" && (
              target.tagName === "INPUT" ||
              target.tagName === "TEXTAREA" ||
              target.isContentEditable
            )) {
              // Allow default paste in inputs
              continue;
            }
          }

          if (shortcut.preventDefault !== false) {
            event.preventDefault();
          }
          shortcut.action();
          break;
        }
      }
    },
    [shortcuts]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}

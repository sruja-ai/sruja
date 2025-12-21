// apps/playground/src/components/shared/ShortcutsModal.tsx
import { useEffect } from "react";
import { X } from "lucide-react";
import { Button } from "@sruja/ui";

export interface Shortcut {
  keys: string[];
  description: string;
  category?: string;
}

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
  shortcuts: Shortcut[];
}

export function ShortcutsModal({ isOpen, onClose, shortcuts }: ShortcutsModalProps) {
  // Close on Escape key
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  // Group shortcuts by category
  const groupedShortcuts = shortcuts.reduce(
    (acc, shortcut) => {
      const category = shortcut.category || "general";
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(shortcut);
      return acc;
    },
    {} as Record<string, Shortcut[]>
  );

  const categoryOrder = ["general", "navigation", "actions", "export", "editing"];

  const formatKey = (key: string): string => {
    // Format common keys
    const keyMap: Record<string, string> = {
      cmd: "⌘",
      ctrl: "Ctrl",
      shift: "⇧",
      alt: "⌥",
      meta: "⌘",
      escape: "Esc",
      enter: "↵",
      backspace: "⌫",
      delete: "⌦",
      arrowup: "↑",
      arrowdown: "↓",
      arrowleft: "←",
      arrowright: "→",
    };

    const lowerKey = key.toLowerCase();
    return keyMap[lowerKey] || key.toUpperCase();
  };

  const formatKeys = (keys: string[]): string => {
    const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
    return keys
      .map((key) => {
        if (isMac && key.toLowerCase() === "ctrl") {
          return "⌘";
        }
        return formatKey(key);
      })
      .join(" + ");
  };

  return (
    <div
      className="shortcuts-modal-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
      style={{
        position: "fixed",
        inset: 0,
        backgroundColor: "rgba(0, 0, 0, 0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10000,
        padding: "20px",
      }}
    >
      <div
        className="shortcuts-modal"
        onClick={(e) => e.stopPropagation()}
        style={{
          width: "600px",
          maxWidth: "90vw",
          maxHeight: "80vh",
          backgroundColor: "var(--bg-primary)",
          border: "1px solid var(--border-color)",
          borderRadius: "12px",
          boxShadow: "0 20px 60px rgba(0, 0, 0, 0.3)",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "20px",
            borderBottom: "1px solid var(--border-color)",
          }}
        >
          <h2
            style={{
              margin: 0,
              fontSize: "20px",
              fontWeight: 600,
              color: "var(--text-primary)",
            }}
          >
            Keyboard Shortcuts
          </h2>
          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            style={{
              padding: "4px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "var(--text-secondary)",
            }}
            aria-label="Close shortcuts"
          >
            <X size={20} />
          </Button>
        </div>

        {/* Content */}
        <div
          style={{
            flex: 1,
            overflowY: "auto",
            padding: "20px",
          }}
        >
          {categoryOrder.map((category) => {
            const categoryShortcuts = groupedShortcuts[category];
            if (!categoryShortcuts || categoryShortcuts.length === 0) return null;

            return (
              <div key={category} style={{ marginBottom: "32px" }}>
                <h3
                  style={{
                    margin: "0 0 12px 0",
                    fontSize: "14px",
                    fontWeight: 600,
                    textTransform: "uppercase",
                    letterSpacing: "0.5px",
                    color: "var(--text-secondary)",
                  }}
                >
                  {category}
                </h3>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                  {categoryShortcuts.map((shortcut, idx) => (
                    <div
                      key={idx}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        padding: "12px",
                        backgroundColor: "var(--bg-secondary)",
                        borderRadius: "8px",
                      }}
                    >
                      <span
                        style={{
                          fontSize: "14px",
                          color: "var(--text-primary)",
                        }}
                      >
                        {shortcut.description}
                      </span>
                      <kbd
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: "4px",
                          padding: "6px 12px",
                          backgroundColor: "var(--bg-tertiary)",
                          border: "1px solid var(--border-color)",
                          borderRadius: "6px",
                          fontSize: "12px",
                          fontFamily: "monospace",
                          color: "var(--text-primary)",
                          fontWeight: 500,
                        }}
                      >
                        {formatKeys(shortcut.keys)}
                      </kbd>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

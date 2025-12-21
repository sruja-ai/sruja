// apps/playground/src/components/shared/CommandPalette.tsx
import { useState, useEffect, useRef, useMemo } from "react";
import { Search } from "lucide-react";
import { Input, Button } from "@sruja/ui";

export interface Command {
  id: string;
  label: string;
  description?: string;
  icon?: React.ReactNode;
  category: "navigation" | "actions" | "export" | "settings";
  action: () => void | Promise<void>;
  keywords?: string[]; // For fuzzy search
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: Command[];
}

export function CommandPalette({ isOpen, onClose, commands }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Filter commands based on query
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;

    const q = query.toLowerCase().trim();
    return commands.filter((cmd) => {
      const labelMatch = cmd.label.toLowerCase().includes(q);
      const descMatch = cmd.description?.toLowerCase().includes(q);
      const keywordMatch = cmd.keywords?.some((k) => k.toLowerCase().includes(q));
      return labelMatch || descMatch || keywordMatch;
    });
  }, [commands, query]);

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setQuery("");
      setSelectedIndex(0);
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current && filteredCommands.length > 0) {
      const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    }
  }, [selectedIndex, filteredCommands.length]);

  // Handle keyboard navigation
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, filteredCommands.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          filteredCommands[selectedIndex].action();
          onClose();
        }
      } else if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, filteredCommands, selectedIndex, onClose]);

  // Group commands by category (must be declared before any conditional returns)
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {};
    filteredCommands.forEach((cmd) => {
      if (!groups[cmd.category]) {
        groups[cmd.category] = [];
      }
      groups[cmd.category].push(cmd);
    });
    return groups;
  }, [filteredCommands]);

  const categoryOrder: Array<keyof typeof groupedCommands> = [
    "navigation",
    "actions",
    "export",
    "settings",
  ];

  if (!isOpen) return null;

  return (
    <div
      className="command-palette-overlay"
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
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "20vh",
        zIndex: 10000,
      }}
    >
      <div
        className="command-palette"
        onClick={(e) => e.stopPropagation()}
        style={{
          width: "600px",
          maxWidth: "90vw",
          backgroundColor: "var(--bg-primary)",
          border: "1px solid var(--border-color)",
          borderRadius: "12px",
          boxShadow: "0 20px 60px rgba(0, 0, 0, 0.3)",
          overflow: "hidden",
        }}
      >
        {/* Search input */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "12px",
            padding: "16px",
            borderBottom: "1px solid var(--border-color)",
          }}
        >
          <Search size={20} style={{ color: "var(--text-secondary)" }} />
          <Input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type a command or search..."
            style={{
              flex: 1,
              border: "none",
              outline: "none",
              background: "transparent",
              color: "var(--text-primary)",
              fontSize: "16px",
            }}
          />
          <kbd
            style={{
              padding: "4px 8px",
              backgroundColor: "var(--bg-tertiary)",
              border: "1px solid var(--border-color)",
              borderRadius: "4px",
              fontSize: "12px",
              color: "var(--text-secondary)",
            }}
          >
            Esc
          </kbd>
        </div>

        {/* Commands list */}
        <div
          ref={listRef}
          style={{
            maxHeight: "400px",
            overflowY: "auto",
            padding: "8px",
          }}
        >
          {filteredCommands.length === 0 ? (
            <div
              style={{
                padding: "32px",
                textAlign: "center",
                color: "var(--text-secondary)",
                fontSize: "14px",
              }}
            >
              No commands found
            </div>
          ) : (
            categoryOrder.map((category) => {
              const categoryCommands = groupedCommands[category];
              if (!categoryCommands || categoryCommands.length === 0) return null;

              return (
                <div key={category}>
                  <div
                    style={{
                      padding: "8px 12px",
                      fontSize: "11px",
                      fontWeight: 600,
                      textTransform: "uppercase",
                      letterSpacing: "0.5px",
                      color: "var(--text-secondary)",
                    }}
                  >
                    {category}
                  </div>
                  {categoryCommands.map((cmd) => {
                    const globalIndex = filteredCommands.indexOf(cmd);
                    const isSelected = globalIndex === selectedIndex;

                    return (
                      <Button
                        key={cmd.id}
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          cmd.action();
                          onClose();
                        }}
                        style={{
                          width: "100%",
                          padding: "12px 16px",
                          display: "flex",
                          alignItems: "center",
                          gap: "12px",
                          background: isSelected ? "var(--bg-tertiary)" : "transparent",
                          textAlign: "left",
                          color: "var(--text-primary)",
                          fontSize: "14px",
                        }}
                        onMouseEnter={() => setSelectedIndex(globalIndex)}
                      >
                        {cmd.icon && <span style={{ flexShrink: 0 }}>{cmd.icon}</span>}
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div style={{ fontWeight: 500 }}>{cmd.label}</div>
                          {cmd.description && (
                            <div
                              style={{
                                fontSize: "12px",
                                color: "var(--text-secondary)",
                                marginTop: "2px",
                              }}
                            >
                              {cmd.description}
                            </div>
                          )}
                        </div>
                      </Button>
                    );
                  })}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}

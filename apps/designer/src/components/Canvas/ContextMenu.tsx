// apps/designer/src/components/Canvas/ContextMenu.tsx
import { useEffect, useRef } from "react";
import { Button } from "@sruja/ui";
import "./ContextMenu.css";

export interface ContextMenuAction {
  id: string;
  label: string;
  icon?: React.ReactNode;
  action: () => void;
  disabled?: boolean;
  separator?: boolean;
}

interface ContextMenuProps {
  x: number;
  y: number;
  actions: ContextMenuAction[];
  onClose: () => void;
}

export function ContextMenu({ x, y, actions, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    // Use capture phase to handle clicks before they bubble
    document.addEventListener("mousedown", handleClickOutside, true);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside, true);
    };
  }, [onClose]);

  // Close menu on Escape key
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  // Adjust position if menu would go off screen
  const [adjustedX, adjustedY] = (() => {
    if (!menuRef.current) return [x, y];

    const rect = menuRef.current.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let adjX = x;
    let adjY = y;

    if (x + rect.width > viewportWidth) {
      adjX = viewportWidth - rect.width - 10;
    }
    if (y + rect.height > viewportHeight) {
      adjY = viewportHeight - rect.height - 10;
    }

    return [adjX, adjY];
  })();

  return (
    <div
      ref={menuRef}
      className="context-menu"
      style={{
        position: "fixed",
        left: `${adjustedX}px`,
        top: `${adjustedY}px`,
        zIndex: 10000,
      }}
    >
      {actions.map((action, index) => {
        if (action.separator) {
          return <div key={`separator-${index}`} className="context-menu-separator" />;
        }

        return (
          <Button
            key={action.id}
            variant="ghost"
            size="sm"
            className={`context-menu-item ${action.disabled ? "disabled" : ""}`}
            onClick={(e) => {
              e.stopPropagation();
              if (!action.disabled) {
                action.action();
                onClose();
              }
            }}
            disabled={action.disabled}
          >
            {action.icon && <span className="context-menu-icon">{action.icon}</span>}
            <span className="context-menu-label">{action.label}</span>
          </Button>
        );
      })}
    </div>
  );
}

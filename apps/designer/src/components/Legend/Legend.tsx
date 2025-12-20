/**
 * C4 Legend Panel
 *
 * Shows the notation key for the C4 diagram.
 * Can be toggled on/off via a button.
 */

import { useState } from "react";
import { HelpCircle, X, User, Box, Database, MessageSquare } from "lucide-react";
import { Button } from "@sruja/ui";
import "./Legend.css";

interface LegendItem {
  label: string;
  type: string;
  color: string;
  shape: string;
  icon: React.ReactNode;
}

const LEGEND_ITEMS: LegendItem[] = [
  {
    label: "Person",
    type: "person",
    color: "#08427B",
    shape: "Rounded rectangle",
    icon: <User size={16} />,
  },
  {
    label: "Software System",
    type: "system",
    color: "#1168BD",
    shape: "Rectangle",
    icon: <Box size={16} />,
  },
  {
    label: "External System",
    type: "external",
    color: "#999999",
    shape: "Dashed rectangle",
    icon: <Box size={16} />,
  },
  {
    label: "Container",
    type: "container",
    color: "#438DD5",
    shape: "Rectangle",
    icon: <Box size={16} />,
  },
  {
    label: "Component",
    type: "component",
    color: "#85BBF0",
    shape: "Rectangle",
    icon: <Box size={16} />,
  },
  {
    label: "Database",
    type: "database",
    color: "#438DD5",
    shape: "Cylinder",
    icon: <Database size={16} />,
  },
  {
    label: "Queue",
    type: "queue",
    color: "#438DD5",
    shape: "Rectangle",
    icon: <MessageSquare size={16} />,
  },
];

export function Legend() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      {/* Toggle Button */}
      <Button
        variant="ghost"
        size="sm"
        className="legend-toggle"
        onClick={() => setIsOpen(!isOpen)}
        title={isOpen ? "Hide Legend" : "Show Legend"}
      >
        <HelpCircle size={18} />
        <span>Legend</span>
      </Button>

      {/* Legend Panel */}
      {isOpen && (
        <div className="legend-panel">
          <div className="legend-header">
            <h3>C4 Notation</h3>
            <Button variant="ghost" size="sm" className="legend-close" onClick={() => setIsOpen(false)}>
              <X size={16} />
            </Button>
          </div>

          <div className="legend-section">
            <h4>Elements</h4>
            <div className="legend-items">
              {LEGEND_ITEMS.map((item) => (
                <div key={item.type} className="legend-item">
                  <div
                    className={`legend-swatch ${item.type === "external" ? "dashed" : ""}`}
                    style={{ borderColor: item.color }}
                  >
                    {item.icon}
                  </div>
                  <div className="legend-text">
                    <span className="legend-label">{item.label}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="legend-section">
            <h4>Relationships</h4>
            <div className="legend-items">
              <div className="legend-item">
                <div className="legend-arrow">
                  <svg width="40" height="16" viewBox="0 0 40 16">
                    <line x1="0" y1="8" x2="32" y2="8" stroke="#707070" strokeWidth="1.5" />
                    <polygon points="32,4 40,8 32,12" fill="#707070" />
                  </svg>
                </div>
                <div className="legend-text">
                  <span className="legend-label">Relationship / Dependency</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

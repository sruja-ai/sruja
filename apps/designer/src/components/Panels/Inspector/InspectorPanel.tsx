import { useState, useEffect } from "react";
import { ChevronRight, ChevronLeft, Info, Layers } from "lucide-react";
import { Button } from "@sruja/ui";
import { useSelectionStore } from "../../../stores";
import { ProjectInspector } from "./ProjectInspector";
import { ElementInspector } from "./ElementInspector";
import "./InspectorPanel.css";

export function InspectorPanel() {
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);

  // Visibility is now controlled globally by App.tsx via useUIStore.isInspectorVisible
  // The panel is completely unmounted when hidden, VS Code style.

  return (
    <div className="inspector-panel">
      {/* Header */}
      <div className="inspector-header">
        <div className="inspector-title">
          {selectedNodeId ? (
            <>
              <Layers size={16} />
              <span>Inspector</span>
            </>
          ) : (
            <>
              <Info size={16} />
              <span>Project Overview</span>
            </>
          )}
        </div>
      </div>

      {/* Content Area */}
      <div className="inspector-content-container">
        {selectedNodeId ? <ElementInspector /> : <ProjectInspector />}
      </div>
    </div>
  );
}

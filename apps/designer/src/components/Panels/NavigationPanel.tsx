import { useState, useMemo } from "react";
import { X, Plus, Database, Box } from "lucide-react";
import { Input, Button } from "@sruja/ui";
import { useArchitectureStore, useViewStore, useSelectionStore, useUIStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useNavigationData } from "../../hooks/useNavigationData";
import { NavTreeItem } from "./NavTreeItem";
import { QualityScoreCard } from "../non-core/navigation/QualityScoreCard";
import "./NavigationPanel.css";
import type { SrujaModelDump } from "@sruja/shared";
import { studioScope } from "../../config/studioScope";

interface NavigationPanelProps {
  onClose?: () => void;
}

export function NavigationPanel({ onClose }: NavigationPanelProps) {
  const model = useArchitectureStore((s) => s.model) as SrujaModelDump | null;
  const currentLevel = useViewStore((s) => s.currentLevel);
  const focusedSystemId = useViewStore((s) => s.focusedSystemId);
  const focusedContainerId = useViewStore((s) => s.focusedContainerId);
  const drillDown = useViewStore((s) => s.drillDown);
  const goToRoot = useViewStore((s) => s.goToRoot);
  const setLevel = useViewStore((s) => s.setLevel);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectNode = useSelectionStore((s) => s.selectNode);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Derive selection context
  const getSelectedElement = () => {
    if (!selectedNodeId || !model) return null;
    return model.elements[selectedNodeId];
  };
  const selectedElement = getSelectedElement();

  const [filterQuery, setFilterQuery] = useState("");

  const { filteredPersons, filteredSystems, getChildren } = useNavigationData({
    model,
    filterQuery,
  });

  // Track expanded nodes locally
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());

  const toggleExpand = (id: string) => {
    setExpandedNodes((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  // --------------------------------------------------------------------------
  // Context Actions Logic
  // --------------------------------------------------------------------------
  const contextAction = useMemo(() => {
    if (!isEditMode()) return null;

    if (!selectedElement) {
      // Root Context -> Add System
      return {
        label: "Add System",
        icon: <Plus size={16} />,
        action: () => {
          useUIStore.getState().setBuilderStep("context");
          useUIStore.getState().setActiveTab("builder");
        },
        secondary: {
          label: "Add Actor",
          action: () => {
            /* TODO: Add actor specific step or handle in context step */
          },
        },
      };
    }

    if (selectedElement.kind === "system") {
      // System Context -> Add Container
      return {
        label: "Add Container",
        icon: <Database size={16} />,
        action: () => {
          useUIStore.getState().setBuilderStep("containers");
          useUIStore.getState().setActiveTab("builder");
          // Optionally auto-focus this system in wizard if supported
        },
      };
    }

    if (["container", "webapp", "mobile", "api", "database"].includes(selectedElement.kind)) {
      // Container Context -> Add Component
      return {
        label: "Add Component",
        icon: <Box size={16} />,
        action: () => {
          useUIStore.getState().setBuilderStep("components");
          useUIStore.getState().setActiveTab("builder");
        },
      };
    }

    return null;
  }, [selectedElement, isEditMode]);

  // --------------------------------------------------------------------------
  // Editing State Stubs
  // --------------------------------------------------------------------------
  // ... (keeping stubs or removing if not used, reusing generic pattern)

  return (
    <div className="navigation-panel glass-panel">
      {/* 
         Context Action Header (Sticky)
         Shows the most relevant action based on selection
      */}
      {contextAction && (
        <div className="nav-context-bar">
          <Button className="context-action-btn primary" onClick={contextAction.action}>
            {contextAction.icon}
            <span>{contextAction.label}</span>
          </Button>
        </div>
      )}

      {/* Main Content */}
      <div className="nav-content-scroll">
        {/* Mobile close button */}
        {onClose && (
          <div className="panel-mobile-header">
            <span>Explorer</span>
            <Button
              variant="ghost"
              size="sm"
              className="panel-close-btn"
              onClick={onClose}
              aria-label="Close navigation"
            >
              <X size={18} />
            </Button>
          </div>
        )}

        {/* Search */}
        <div className="nav-search-row">
          <Input
            placeholder="Search..."
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            className="nav-search-input"
          />
        </div>

        {/* Level Selector - Compact */}
        <div className="nav-section compact-section">
          <div className="segmented-level-control">
            <button
              className={`segment-btn ${currentLevel === "L1" ? "active" : ""}`}
              onClick={goToRoot}
              title="System Context - Actors and Systems"
            >
              <span className="level-label">Context</span>
            </button>
            <button
              className={`segment-btn ${currentLevel === "L2" ? "active" : ""}`}
              disabled={!focusedSystemId && selectedElement?.kind !== "system"}
              onClick={() => {
                if (focusedSystemId) setLevel("L2");
                else if (selectedElement?.kind === "system")
                  drillDown(selectedElement.id, "system");
              }}
              title="Containers - Apps, Databases, Queues"
            >
              <span className="level-label">Containers</span>
            </button>
            <button
              className={`segment-btn ${currentLevel === "L3" ? "active" : ""}`}
              disabled={!focusedContainerId && selectedElement?.kind !== "container"}
              onClick={() => {
                if (focusedContainerId) setLevel("L3");
                else if (selectedElement?.kind === "container")
                  drillDown(selectedElement.id, "container", selectedElement.parent || undefined);
              }}
              title="Components - Internal Building Blocks"
            >
              <span className="level-label">Components</span>
            </button>
          </div>
        </div>

        {/* Quality Score Card */}
        {studioScope.qualityScoreCard && <QualityScoreCard isCollapsed={false} />}

        {/* Systems Tree */}
        <div className="nav-section tree-section">
          <div className="tree-header">
            <span>Systems & Containers</span>
            <span className="tree-header-hint" title="Your main applications and their parts">
              ℹ️
            </span>
          </div>
          <ul className="nav-tree">
            {filteredSystems.length === 0 && (
              <li className="nav-empty">
                <div className="nav-empty-content">
                  <p>No systems found.</p>
                  <p className="nav-empty-hint">💡 Start by adding systems in the Builder tab</p>
                </div>
              </li>
            )}
            {filteredSystems.map((system) => {
              const containers = getChildren(system.id, "container");
              const isExpanded = expandedNodes.has(system.id);
              return (
                <NavTreeItem
                  key={system.id}
                  element={system}
                  isExpanded={isExpanded}
                  isSelected={selectedNodeId === system.id}
                  hasChildren={containers.length > 0}
                  onExpand={toggleExpand}
                  onDrillDown={(id) => {
                    selectNode(id);
                    drillDown(id, "system");
                  }}
                  isEditMode={!!isEditMode()}
                  onEdit={() => {}} // TODO: Connect to properties panel
                >
                  {containers.map((container) => {
                    const components = getChildren(container.id, "component");
                    const isContExpanded = expandedNodes.has(container.id);
                    return (
                      <NavTreeItem
                        key={container.id}
                        element={container}
                        isExpanded={isContExpanded}
                        isSelected={selectedNodeId === container.id}
                        hasChildren={components.length > 0}
                        onExpand={toggleExpand}
                        onDrillDown={(id, _kind, pid) => {
                          selectNode(id);
                          drillDown(id, "container", pid!);
                        }}
                        isEditMode={!!isEditMode()}
                        onEdit={() => {}}
                      >
                        {/* Components (Leaf nodes) */}
                        {components.map((component) => (
                          <NavTreeItem
                            key={component.id}
                            element={component}
                            isExpanded={false}
                            isSelected={selectedNodeId === component.id}
                            hasChildren={false}
                            onExpand={() => {}}
                            onDrillDown={(id, _kind, pid) => {
                              selectNode(id);
                              if (pid) {
                                drillDown(pid, "container", undefined);
                              }
                            }}
                            isEditMode={!!isEditMode()}
                            onEdit={() => {}}
                          />
                        ))}
                      </NavTreeItem>
                    );
                  })}
                </NavTreeItem>
              );
            })}
          </ul>
        </div>

        {/* Persons */}
        {filteredPersons.length > 0 && (
          <div className="nav-section tree-section">
            <div className="tree-header">
              <span>Actors</span>
              <span
                className="tree-header-hint"
                title="People or external systems that use your system"
              >
                ℹ️
              </span>
            </div>
            <ul className="nav-tree">
              {filteredPersons.map((person) => (
                <NavTreeItem
                  key={person.id}
                  element={person}
                  isExpanded={false}
                  isSelected={selectedNodeId === person.id}
                  hasChildren={false}
                  onExpand={() => {}}
                  onDrillDown={(id) => {
                    selectNode(id);
                    goToRoot();
                  }}
                  isEditMode={!!isEditMode()}
                  onEdit={() => {}}
                />
              ))}
            </ul>
          </div>
        )}

        {/* Empty state */}
        {!model && (
          <div className="panel-empty">
            <div className="panel-empty-content">
              <p>No architecture loaded</p>
              <p className="panel-empty-hint">
                💡 Go to the <strong>Builder</strong> tab to start creating your architecture
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

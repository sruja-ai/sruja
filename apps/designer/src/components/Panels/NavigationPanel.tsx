import { useState, useEffect } from "react";
import { ChevronRight, ChevronLeft, Building2, User, X, Link2 } from "lucide-react";
import { Input, Button } from "@sruja/ui";
import { logger } from "@sruja/shared";
import { useArchitectureStore, useViewStore, useSelectionStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useNavigationData } from "../../hooks/useNavigationData";
import { NavTreeItem } from "./NavTreeItem";
import { QualityScoreCard } from "./QualityScoreCard";
import "./NavigationPanel.css";
import type { Element, SrujaModelDump } from "@sruja/shared";

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
  const selectNode = useSelectionStore((s) => s.selectNode); // Add selectNode action
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

  // Collapsed state with localStorage persistence
  const [isCollapsed, setIsCollapsed] = useState(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("navigation-panel-collapsed");
      return saved === "true";
    }
    return false;
  });

  useEffect(() => {
    if (typeof window !== "undefined") {
      localStorage.setItem("navigation-panel-collapsed", String(isCollapsed));
    }
  }, [isCollapsed]);

  const toggleCollapse = () => {
    setIsCollapsed((prev) => !prev);
  };

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
  // Editing State Stubs
  // --------------------------------------------------------------------------
  const onEditSystem = (_sys: Element) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "edit_system",
      elementType: "system",
    });
  };
  const onAddSystem = () => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "add_system",
      elementType: "system",
    });
  };
  const onEditContainer = (_cont: Element) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "edit_container",
      elementType: "container",
    });
  };
  const onAddContainer = (_sysId: string) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "add_container",
      elementType: "container",
      systemId: _sysId,
    });
  };
  const onEditComponent = (_comp: Element) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "edit_component",
      elementType: "component",
    });
  };
  const onAddComponent = (_contId: string) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "add_component",
      elementType: "component",
      containerId: _contId,
    });
  };
  const onEditPerson = (_p: Element) => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "edit_person",
      elementType: "person",
    });
  };
  const onAddPerson = () => {
    logger.warn("Action not implemented", {
      component: "NavigationPanel",
      action: "add_person",
      elementType: "person",
    });
  };

  return (
    <div className={`navigation-panel ${isCollapsed ? "collapsed" : ""}`}>
      {!isCollapsed && (
        <div className="nav-header">
          <h3 className="nav-title">Navigation</h3>
        </div>
      )}
      {!model && !isCollapsed && (
        <div className="panel-empty">Load an architecture to see navigation</div>
      )}
      {!isCollapsed && (
        <div className="nav-search-row">
          <Input
            placeholder="Search systems, actors..."
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
          />
        </div>
      )}

      {/* Collapse Toggle */}
      <Button
        variant="ghost"
        size="sm"
        className="nav-collapse-btn"
        onClick={toggleCollapse}
        aria-label={isCollapsed ? "Expand navigation" : "Collapse navigation"}
        title={isCollapsed ? "Expand navigation" : "Collapse navigation"}
      >
        {isCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
      </Button>

      {onClose && (
        <div className="panel-mobile-header">
          <span>Navigation</span>
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

      {/* Level Selector - Segmented Control */}
      <div className="nav-section">
        {!isCollapsed && <div className="nav-section-title">View Level</div>}
        <div className={`segmented-level-control ${isCollapsed ? "collapsed" : ""}`}>
          <button
            className={`segment-btn ${currentLevel === "L1" ? "active" : ""}`}
            onClick={goToRoot}
            title="System Context - Shows systems and actors"
          >
            L1
          </button>
          <button
            className={`segment-btn ${currentLevel === "L2" ? "active" : ""}`}
            title={
              !focusedSystemId && selectedElement?.kind !== "system"
                ? "Select a system to view its containers"
                : "Container View - Detailed system internals"
            }
            disabled={!focusedSystemId && selectedElement?.kind !== "system"}
            onClick={() => {
              if (focusedSystemId) setLevel("L2");
              else if (selectedElement?.kind === "system") drillDown(selectedElement.id, "system");
            }}
          >
            L2
          </button>
          <button
            className={`segment-btn ${currentLevel === "L3" ? "active" : ""}`}
            title={
              !focusedContainerId && selectedElement?.kind !== "container"
                ? "Select a container to view its components"
                : "Component View - Fine-grained components"
            }
            disabled={!focusedContainerId && selectedElement?.kind !== "container"}
            onClick={() => {
              if (focusedContainerId) setLevel("L3");
              else if (selectedElement?.kind === "container")
                drillDown(selectedElement.id, "container", selectedElement.parent || undefined);
            }}
          >
            L3
          </button>
        </div>
      </div>

      {/* Quality Score Card */}
      <QualityScoreCard isCollapsed={isCollapsed} />

      {/* Systems Tree */}
      <div className="nav-section">
        <div className="nav-section-title">
          <Building2 size={14} />
          {!isCollapsed && <span>Systems ({filteredSystems.length})</span>}
        </div>
        <ul className="nav-tree">
          {filteredSystems.length === 0 && (
            <li className="nav-empty">
              No systems.{" "}
              {isEditMode() && (
                <Button variant="ghost" size="sm" className="link-btn" onClick={onAddSystem}>
                  Add a system
                </Button>
              )}
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
                onEdit={onEditSystem}
                onAddChild={onAddContainer}
              >
                {containers.map((container: any) => {
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
                      onEdit={onEditContainer}
                      onAddChild={onAddComponent}
                    >
                      {/* Components (Leaf nodes) */}
                      {components.map((component: any) => (
                        <NavTreeItem
                          key={component.id}
                          element={component}
                          isExpanded={false}
                          isSelected={selectedNodeId === component.id}
                          hasChildren={false}
                          onExpand={() => {}}
                          onDrillDown={(id, _kind, pid) => {
                            selectNode(id);
                            // Navigate to parent container view
                            if (pid) {
                              drillDown(pid, "container", undefined); // We might need to look up parent's parent for full context? navigateOnClick usually handles ID.
                              // Wait, drillDown(targetId, kind, parentId? context).
                              // For L3, we drill down to the CONTAINER (pid).
                              // drillDown(pid, "container") should set Level 3, Focus pid.
                            }
                          }}
                          isEditMode={!!isEditMode()}
                          onEdit={onEditComponent}
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
      <div className="nav-section">
        <div className="nav-section-title">
          <User size={14} />
          {!isCollapsed && <span>Actors ({filteredPersons.length})</span>}
        </div>
        <ul className="nav-tree">
          {filteredPersons.length === 0 && (
            <li className="nav-empty">
              No actors.{" "}
              {isEditMode() && (
                <Button variant="ghost" size="sm" className="link-btn" onClick={onAddPerson}>
                  Add a person
                </Button>
              )}
            </li>
          )}
          {filteredPersons.map((person: any) => (
            <NavTreeItem
              key={person.id}
              element={person}
              isExpanded={false}
              isSelected={selectedNodeId === person.id}
              hasChildren={false}
              onExpand={() => {}}
              onDrillDown={(id) => {
                selectNode(id);
                goToRoot(); // Persons are in L1
              }}
              isEditMode={!!isEditMode()}
              onEdit={onEditPerson}
            />
          ))}
        </ul>
      </div>

      {/* Quick Links to Details Tab */}
      {!isCollapsed && (
        <div className="nav-section">
          <div className="nav-section-title">
            <Link2 size={14} />
            <span>Governance</span>
          </div>
          <div className="nav-governance-hint">Select an item to view governance details.</div>
        </div>
      )}
    </div>
  );
}

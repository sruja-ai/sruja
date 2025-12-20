import { useState, useEffect } from "react";
import {
  ChevronRight,
  ChevronLeft,
  Building2,
  User,
  X,
  Link2,
} from "lucide-react";
import { Input, Button } from "@sruja/ui";
import { useArchitectureStore, useViewStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useNavigationData } from "../../hooks/useNavigationData";
import { useLayoutMetrics } from "../../hooks/useLayoutMetrics";
import { NavTreeItem } from "./NavTreeItem";
import "./NavigationPanel.css";
import type { Element, SrujaModelDump } from "@sruja/shared";

interface NavigationPanelProps {
  onClose?: () => void;
}

export function NavigationPanel({ onClose }: NavigationPanelProps) {
  const likec4Model = useArchitectureStore((s) => s.likec4Model) as SrujaModelDump | null;
  const currentLevel = useViewStore((s) => s.currentLevel);
  const focusedSystemId = useViewStore((s) => s.focusedSystemId);
  const focusedContainerId = useViewStore((s) => s.focusedContainerId);
  const drillDown = useViewStore((s) => s.drillDown);
  const goToRoot = useViewStore((s) => s.goToRoot);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  const [filterQuery, setFilterQuery] = useState("");

  const { filteredPersons, filteredSystems, getChildren } = useNavigationData({
    likec4Model,
    filterQuery
  });

  // Keep metrics polling active (though typically not displayed in NavPanel anymore/hidden feature)
  useLayoutMetrics();

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
  const onEditSystem = (_sys: Element) => { console.warn("Edit System not implemented"); };
  const onAddSystem = () => { console.warn("Add System not implemented"); };
  const onEditContainer = (_cont: Element) => { console.warn("Edit Container not implemented"); };
  const onAddContainer = (_sysId: string) => { console.warn("Add Container not implemented"); };
  const onEditComponent = (_comp: Element) => { console.warn("Edit Component not implemented"); };
  const onAddComponent = (_contId: string) => { console.warn("Add Component not implemented"); };
  const onEditPerson = (_p: Element) => { console.warn("Edit Person not implemented"); };
  const onAddPerson = () => { console.warn("Add Person not implemented"); };


  return (
    <div className={`navigation-panel ${isCollapsed ? "collapsed" : ""}`}>
      {!isCollapsed && (
        <div className="nav-header">
          <h3 className="nav-title">Navigation</h3>
        </div>
      )}
      {!likec4Model && !isCollapsed && (
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
          <Button variant="ghost" size="sm" className="panel-close-btn" onClick={onClose} aria-label="Close navigation">
            <X size={18} />
          </Button>
        </div>
      )}

      {/* Level Selector */}
      <div className="nav-section">
        {!isCollapsed && <div className="nav-section-title">View Level</div>}
        <div className={`level-buttons ${isCollapsed ? "collapsed" : ""}`}>
          <Button
            variant={currentLevel === "L1" ? "primary" : "ghost"}
            size="sm"
            className={`level-btn ${currentLevel === "L1" ? "active" : ""}`}
            onClick={goToRoot}
            title="System Context - Shows systems and actors"
          >
            {isCollapsed ? "L1" : "L1 Context"}
          </Button>
          <Button
            variant={currentLevel === "L2" ? "primary" : "ghost"}
            size="sm"
            className={`level-btn ${currentLevel === "L2" ? "active" : ""}`}
            disabled={!focusedSystemId}
            title="Container View - Detailed system internals"
          >
            {isCollapsed ? "L2" : "L2 Container"}
          </Button>
          <Button
            variant={currentLevel === "L3" ? "primary" : "ghost"}
            size="sm"
            className={`level-btn ${currentLevel === "L3" ? "active" : ""}`}
            disabled={currentLevel !== "L3"}
            title="Component View - Fine-grained components"
          >
            {isCollapsed ? "L3" : "L3 Component"}
          </Button>
        </div>
      </div>

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
          {filteredSystems.map((system: any) => {
            const containers = getChildren(system.id, 'container');
            const isExpanded = expandedNodes.has(system.id);
            return (
              <NavTreeItem
                key={system.id}
                element={system}
                isExpanded={isExpanded}
                isSelected={focusedSystemId === system.id}
                hasChildren={containers.length > 0}
                onExpand={toggleExpand}
                onDrillDown={(id) => drillDown(id, "system")}
                isEditMode={!!isEditMode()}
                onEdit={onEditSystem}
                onAddChild={onAddContainer}
              >
                {containers.map((container: any) => {
                  const components = getChildren(container.id, 'component');
                  const isContExpanded = expandedNodes.has(container.id);
                  return (
                    <NavTreeItem
                      key={container.id}
                      element={container}
                      isExpanded={isContExpanded}
                      isSelected={focusedContainerId === container.id}
                      hasChildren={components.length > 0}
                      onExpand={toggleExpand}
                      onDrillDown={(id, _kind, pid) => drillDown(id, "container", pid!)}
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
                          isSelected={false} // Components rarely selected as view root
                          hasChildren={false}
                          onExpand={() => { }}
                          onDrillDown={() => { }} // Usually L3 is leaf
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
                <Button variant="ghost" size="sm" className="link-btn" onClick={onAddPerson}>Add a person</Button>
              )}
            </li>
          )}
          {filteredPersons.map((person: any) => (
            <NavTreeItem
              key={person.id}
              element={person}
              isExpanded={false}
              isSelected={false}
              hasChildren={false}
              onExpand={() => { }}
              onDrillDown={() => { }} // Persons usually don't have internal views
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
          <div style={{ padding: '8px 12px', fontSize: '13px', color: '#666' }}>
            Select an item to view governance details.
          </div>
        </div>
      )}
    </div>
  );
}

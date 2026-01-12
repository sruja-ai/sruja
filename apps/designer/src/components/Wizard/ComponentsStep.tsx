import { useState, useMemo } from "react";
import { Puzzle, Plus, Trash2, Box, ChevronRight, Edit } from "lucide-react";
import { Button } from "@sruja/ui"; // Removed Input
import { useArchitectureStore } from "../../stores/architectureStore";
import { useViewStore } from "../../stores/viewStore";
import { useSelectionStore } from "../../stores";
import { useEffect } from "react";
import { BestPracticeTip } from "@sruja/ui";
import "@sruja/ui/components/BestPracticeTip.css";
import { EditComponentForm } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { ElementDump, RelationDump } from "@sruja/shared";
import "./WizardSteps.css";

interface ComponentsStepProps {
  onBack: () => void;
  onFinish: () => void;
  readOnly?: boolean;
}

export function ComponentsStep({
  onBack,
  onFinish,
  readOnly: _readOnly = false,
}: ComponentsStepProps) {
  const data = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const drillDown = useViewStore((s) => s.drillDown);

  const allElements = useMemo(() => Object.values(data?.elements || {}), [data?.elements]);

  // Build container list from all systems
  const containerList = useMemo(() => {
    // Find all containers (kind "container") and map to structure for selector
    // ID hierarchy: systemId.containerId ... but kind is reliable
    // We need systemId and containerId.
    // Assuming hierarchy is reflected in ID: system.container
    const containers = allElements.filter((e: ElementDump) => e.kind === "container");

    return containers
      .map((c: ElementDump) => {
        const parts = c.id.split(".");
        // If strict 2 parts: system.container
        const systemId = parts[0];
        const containerId = parts.length > 1 ? parts[1] : c.id; // Fallback
        return {
          systemId,
          containerId,
          label: c.id, // Using full ID as label/value for selector
          title: c.title,
        };
      })
      .sort((a, b) => a.label.localeCompare(b.label));
  }, [allElements]);

  // Infer initial container selection
  const globalSelectedId = useSelectionStore((s) => s.selectedNodeId);

  const initialContainerId = useMemo(() => {
    if (globalSelectedId && data?.elements?.[globalSelectedId]?.kind === "container") {
      return globalSelectedId;
    }
    if (containerList.length > 0) return containerList[0].label;
    return "";
  }, [globalSelectedId, containerList, data]);

  // Form state
  // selectedPath is actually the container ID (full path)
  const [selectedPath, setSelectedPath] = useState(initialContainerId);

  // Sync if initial changes
  useEffect(() => {
    if (initialContainerId && !selectedPath) {
      setSelectedPath(initialContainerId);
    }
  }, [initialContainerId]);

  // Auto-Zoom: Focus diagram on selected container
  useEffect(() => {
    if (selectedPath) {
      drillDown(selectedPath, "container");
    }
  }, [selectedPath, drillDown]);

  // Modal State
  const [isComponentFormOpen, setIsComponentFormOpen] = useState(false);
  const [editComponent, setEditComponent] = useState<ElementDump | undefined>(undefined);

  const components = useMemo(() => {
    if (!selectedPath) return [];
    // Components are children of selectedPath (container ID)
    // ID hierarchy: containerId.componentId (where containerId is system.container)
    return allElements.filter((e: ElementDump) => {
      if (e.kind !== "component") return false;
      // Start with selectedPath + "." ?
      // selectedPath is e.g. "sys1.cont1"
      // component ID is "sys1.cont1.comp1"
      return (
        e.id.startsWith(selectedPath + ".") &&
        e.id.split(".").length === selectedPath.split(".").length + 1
      );
    });
  }, [allElements, selectedPath]);

  const totalComponents = useMemo(() => {
    return allElements.filter((e: ElementDump) => e.kind === "component").length;
  }, [allElements]);

  // Build L3 elements for relations (all components across all containers)
  const l3Elements = useMemo(() => {
    return allElements
      .filter((e: ElementDump) => e.kind === "component")
      .map((e: ElementDump) => ({
        id: e.id,
        label: e.title,
        type: e.kind,
      }));
  }, [allElements]);

  const removeComponent = (componentId: string) => {
    updateArchitecture((model) => {
      const newElements = { ...model.elements };
      delete newElements[componentId];
      return {
        ...model,
        elements: newElements,
      };
    });
  };

  if (containerList.length === 0) {
    return (
      <div className="wizard-step-content">
        <div className="step-placeholder">
          <h2>No Containers Defined Yet</h2>
          <p>
            You need to add at least one container first. Go back to the previous step to add
            containers.
          </p>
          <p className="placeholder-hint">
            💡 <strong>Tip:</strong> Components are optional. You can skip this step if you don't
            need detailed internal architecture.
          </p>
          <div className="step-navigation">
            <Button variant="secondary" onClick={onBack}>
              ← Back to Containers
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <Puzzle size={24} />
        </div>
        <div className="step-header-content">
          <h2>Add Components</h2>
          <p>Define the internal building blocks within each container (optional detail level).</p>
        </div>
      </div>

      <EditComponentForm
        isOpen={isComponentFormOpen}
        onClose={() => setIsComponentFormOpen(false)}
        component={editComponent}
        parentSystemId={selectedPath ? selectedPath.split(".")[0] : null}
        parentContainerId={selectedPath ? selectedPath.split(".")[1] : null}
        initialName=""
      />

      <BestPracticeTip variant="tip" show={totalComponents === 0} stepId="components">
        <strong>What are Components?</strong> — Components are the internal building blocks inside
        containers.
        <br />
        <strong>Examples:</strong> "AuthService" (inside API Server), "PaymentProcessor" (inside API
        Server), "OrderValidator" (inside API Server)
        <br />
        <strong>Note:</strong> This is optional - only add components if you need detailed internal
        architecture.
      </BestPracticeTip>

      {/* Container Selector */}
      <div className="step-section">
        <h3>Select Container</h3>
        <div className="container-tabs">
          {containerList.map((item) => {
            const count = allElements.filter((e: ElementDump) => {
              if (e.kind !== "component") return false;
              // starts with item.label (id) + "."
              return (
                e.id.startsWith(item.label + ".") &&
                e.id.split(".").length === item.label.split(".").length + 1
              );
            }).length;

            const [_sId, _cId] = item.label.split("."); // simple split for display if needed, but item.systemId/containerId exist

            return (
              <Button
                key={item.label}
                variant={selectedPath === item.label ? "primary" : "ghost"}
                size="sm"
                className={`container-tab ${selectedPath === item.label ? "active" : ""}`}
                onClick={() => setSelectedPath(item.label)}
              >
                <Box size={14} />
                <span className="container-path">
                  {item.systemId}
                  <ChevronRight size={12} />
                  {item.title || item.containerId}
                </span>
                <span className="count-badge">{count}</span>
              </Button>
            );
          })}
        </div>
      </div>

      {/* Components List */}
      <div className="step-section">
        <h3>
          Components in {selectedPath}
          <span className="count-badge">{components.length}</span>
        </h3>

        {components.length === 0 && (
          <div className="empty-state-hint">
            <p>
              💡 <strong>Tip:</strong> Components are optional. Only add them if you need to show
              internal details of this container.
            </p>
            <ul className="example-list">
              <li>
                • <strong>AuthService</strong> - Handles authentication
              </li>
              <li>
                • <strong>PaymentProcessor</strong> - Processes payments
              </li>
              <li>
                • <strong>OrderValidator</strong> - Validates orders
              </li>
            </ul>
            <p
              style={{ marginTop: "0.75rem", fontSize: "0.8125rem", color: "var(--text-tertiary)" }}
            >
              You can skip this step and continue if you don't need component-level detail.
            </p>
          </div>
        )}

        <div className="items-list">
          {components.map((c: ElementDump) => (
            <div key={c.id} className="item-card">
              <Puzzle size={16} className="item-icon component" />
              <div className="item-info">
                <span className="item-id">{c.id}</span>
                {c.title && <span className="item-label">{c.title}</span>}
                {c.technology && <span className="item-tech">{c.technology}</span>}
              </div>
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditComponent(c);
                    setIsComponentFormOpen(true);
                  }}
                  title="Edit component"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeComponent(c.id)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
          {components.length === 0 && (
            <p className="empty-message">No components in this container yet</p>
          )}
        </div>

        {/* Add Actions */}
        <div className="add-form">
          <Button
            variant="secondary"
            onClick={() => {
              setEditComponent(undefined);
              setIsComponentFormOpen(true);
            }}
          >
            <Plus size={16} /> Add Component
          </Button>
        </div>
      </div>

      <BestPracticeTip variant="info" show={totalComponents >= 3} stepId="components">
        You have {totalComponents} components defined. Add relations below to show how they
        interact!
      </BestPracticeTip>

      {/* L3 Relations */}
      {l3Elements.length >= 2 && (
        <RelationsSection
          fromElements={l3Elements}
          toElements={l3Elements}
          filterFn={(rel: RelationDump) => {
            // Show L3 relations (two dots in path = System.Container.Component)
            const srcId =
              typeof rel.source === "object" && rel.source?.model
                ? rel.source.model
                : String(rel.source || "");
            const tgtId =
              typeof rel.target === "object" && rel.target?.model
                ? rel.target.model
                : String(rel.target || "");

            const fromParts = srcId.split(".");
            const toParts = tgtId.split(".");
            return fromParts.length === 3 && toParts.length === 3;
          }}
          title="L3 Relations"
          description="Connect components (e.g., Controller → Service → Repository)"
        />
      )}

      {/* L3 Governance */}
      {l3Elements.length > 0 && (
        <GovernanceSection
          elements={l3Elements}
          levelLabel="component-level"
          filterFn={(tags) => {
            if (!tags || tags.length === 0) return false;
            // Show items tagged with L3 elements (two dots)
            return tags.some((t) => t.split(".").length === 3);
          }}
        />
      )}

      {/* Navigation */}
      <div className="step-navigation">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <div className="step-nav-right">
          <span className="step-nav-hint">
            Components are optional. You can finish now or add relations.
          </span>
          <Button variant="primary" onClick={onFinish}>
            Finish & View Diagram →
          </Button>
        </div>
      </div>
    </div>
  );
}

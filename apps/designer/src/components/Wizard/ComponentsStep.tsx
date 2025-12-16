import { useState, useMemo } from "react";
import { Puzzle, Plus, Trash2, Box, ChevronRight } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { ComponentJSON } from "../../types";
import "./WizardSteps.css";

interface ComponentsStepProps {
  onBack: () => void;
  onFinish: () => void;
}

export function ComponentsStep({ onBack, onFinish }: ComponentsStepProps) {
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Build container list from all systems
  const containerList = useMemo(() => {
    const systems = data?.architecture?.systems ?? [];
    const result: { systemId: string; containerId: string; label: string }[] = [];

    systems.forEach((system) => {
      (system.containers ?? []).forEach((container) => {
        result.push({
          systemId: system.id,
          containerId: container.id,
          label: `${system.id}.${container.id}`,
        });
      });
    });
    return result;
  }, [data]);

  // Form state
  const [selectedPath, setSelectedPath] = useState(containerList[0]?.label ?? "");
  const [newId, setNewId] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newTechnology, setNewTechnology] = useState("");

  const selectedContainer = useMemo(() => {
    const [systemId, containerId] = selectedPath.split(".");
    const system = data?.architecture?.systems?.find((s) => s.id === systemId);
    return system?.containers?.find((c) => c.id === containerId);
  }, [selectedPath, data]);

  const components = selectedContainer?.components ?? [];

  const totalComponents = useMemo(() => {
    const systems = data?.architecture?.systems ?? [];
    return systems.reduce(
      (acc, s) =>
        acc + (s.containers ?? []).reduce((cAcc, c) => cAcc + (c.components?.length ?? 0), 0),
      0
    );
  }, [data]);

  // Build L3 elements for relations (all components across all containers)
  const l3Elements = useMemo(() => {
    const systems = data?.architecture?.systems ?? [];
    const result: { id: string; label: string; type: string }[] = [];
    systems.forEach((system) => {
      (system.containers ?? []).forEach((container) => {
        (container.components ?? []).forEach((component) => {
          result.push({
            id: `${system.id}.${container.id}.${component.id}`,
            label: component.label || component.id,
            type: "component",
          });
        });
      });
    });
    return result;
  }, [data]);

  const addComponent = () => {
    if (!newId.trim() || !selectedPath) return;
    const [systemId, containerId] = selectedPath.split(".");

    const newComponent: ComponentJSON = {
      id: newId.trim(),
      label: newLabel.trim() || undefined,
      technology: newTechnology.trim() || undefined,
    };

    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: (arch.architecture.systems ?? []).map((system) => {
          if (system.id !== systemId) return system;
          return {
            ...system,
            containers: (system.containers ?? []).map((container) => {
              if (container.id !== containerId) return container;
              return {
                ...container,
                components: [...(container.components ?? []), newComponent],
              };
            }),
          };
        }),
      },
    }));
    setNewId("");
    setNewLabel("");
    setNewTechnology("");
  };

  const removeComponent = (componentId: string) => {
    const [systemId, containerId] = selectedPath.split(".");

    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: (arch.architecture.systems ?? []).map((system) => {
          if (system.id !== systemId) return system;
          return {
            ...system,
            containers: (system.containers ?? []).map((container) => {
              if (container.id !== containerId) return container;
              return {
                ...container,
                components: (container.components ?? []).filter((c) => c.id !== componentId),
              };
            }),
          };
        }),
      },
    }));
  };

  if (containerList.length === 0) {
    return (
      <div className="wizard-step-content">
        <div className="step-placeholder">
          <h2>No Containers Defined</h2>
          <p>Go back and add at least one container to add components to.</p>
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
          <p>Define the internal components within each container. This is C4 Level 3.</p>
        </div>
      </div>

      <BestPracticeTip variant="tip" show={totalComponents === 0}>
        <strong>Components are optional</strong> — Only add L3 detail for containers that need it.
        Examples: "AuthService", "PaymentProcessor", "OrderValidator"
      </BestPracticeTip>

      {/* Container Selector */}
      <div className="step-section">
        <h3>Select Container</h3>
        <div className="container-tabs">
          {containerList.map((item) => {
            const [sId, cId] = item.label.split(".");
            const system = data?.architecture?.systems?.find((s) => s.id === sId);
            const container = system?.containers?.find((c) => c.id === cId);
            const count = container?.components?.length ?? 0;

            return (
              <button
                key={item.label}
                className={`container-tab ${selectedPath === item.label ? "active" : ""}`}
                onClick={() => setSelectedPath(item.label)}
              >
                <Box size={14} />
                <span className="container-path">
                  {sId}
                  <ChevronRight size={12} />
                  {cId}
                </span>
                <span className="count-badge">{count}</span>
              </button>
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

        <div className="items-list">
          {components.map((c) => (
            <div key={c.id} className="item-card">
              <Puzzle size={16} className="item-icon component" />
              <div className="item-info">
                <span className="item-id">{c.id}</span>
                {c.label && <span className="item-label">{c.label}</span>}
                {c.technology && <span className="item-tech">{c.technology}</span>}
              </div>
              <button className="item-remove" onClick={() => removeComponent(c.id)}>
                <Trash2 size={14} />
              </button>
            </div>
          ))}
          {components.length === 0 && (
            <p className="empty-message">No components in this container yet</p>
          )}
        </div>

        {/* Add Form */}
        <div className="add-form">
          <Input
            label="ID"
            value={newId}
            onChange={(e) => setNewId(e.target.value.replace(/\s/g, ""))}
            placeholder="e.g., AuthService"
            onKeyDown={(e) => e.key === "Enter" && addComponent()}
          />
          <Input
            label="Label"
            value={newLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            placeholder="e.g., Authentication Service"
            onKeyDown={(e) => e.key === "Enter" && addComponent()}
          />
          <Input
            label="Technology"
            value={newTechnology}
            onChange={(e) => setNewTechnology(e.target.value)}
            placeholder="e.g., JWT"
            onKeyDown={(e) => e.key === "Enter" && addComponent()}
          />
          <Button variant="secondary" onClick={addComponent} disabled={!newId.trim()}>
            <Plus size={16} />
            Add
          </Button>
        </div>
      </div>

      <BestPracticeTip variant="info" show={totalComponents >= 3}>
        You have {totalComponents} components defined. Add relations below to show how they
        interact!
      </BestPracticeTip>

      {/* L3 Relations */}
      {l3Elements.length >= 2 && (
        <RelationsSection
          fromElements={l3Elements}
          toElements={l3Elements}
          filterFn={(rel) => {
            // Show L3 relations (two dots in path = System.Container.Component)
            const fromParts = rel.from.split(".");
            const toParts = rel.to.split(".");
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

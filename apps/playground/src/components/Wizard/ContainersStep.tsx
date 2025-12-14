import { useState, useMemo } from "react";
import { Box, Database, MessageSquare, Plus, Trash2, Building2 } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { ContainerJSON, DataStoreJSON, QueueJSON } from "../../types";
import "./WizardSteps.css";

interface ContainersStepProps {
  onNext: () => void;
  onBack: () => void;
}

type ElementType = "container" | "datastore" | "queue";

export function ContainersStep({ onNext, onBack }: ContainersStepProps) {
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const systems = data?.architecture?.systems ?? [];

  // Form state
  const [selectedSystemId, setSelectedSystemId] = useState(systems[0]?.id ?? "");
  const [elementType, setElementType] = useState<ElementType>("container");
  const [newId, setNewId] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newTechnology, setNewTechnology] = useState("");

  const selectedSystem = systems.find((s) => s.id === selectedSystemId);
  const containers = selectedSystem?.containers ?? [];
  const datastores = selectedSystem?.datastores ?? [];
  const queues = selectedSystem?.queues ?? [];

  const totalElements = systems.reduce(
    (acc, s) =>
      acc + (s.containers?.length ?? 0) + (s.datastores?.length ?? 0) + (s.queues?.length ?? 0),
    0
  );
  const isComplete = totalElements > 0;

  // Build L2 elements for relations (all containers, datastores, queues across all systems)
  const l2Elements = useMemo(() => {
    const result: { id: string; label: string; type: string }[] = [];
    systems.forEach((system) => {
      (system.containers ?? []).forEach((c) => {
        result.push({ id: `${system.id}.${c.id}`, label: c.label || c.id, type: "container" });
      });
      (system.datastores ?? []).forEach((d) => {
        result.push({ id: `${system.id}.${d.id}`, label: d.label || d.id, type: "datastore" });
      });
      (system.queues ?? []).forEach((q) => {
        result.push({ id: `${system.id}.${q.id}`, label: q.label || q.id, type: "queue" });
      });
    });
    return result;
  }, [systems]);

  const addElement = () => {
    if (!newId.trim() || !selectedSystemId || !data?.architecture) return;

    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: (arch.architecture.systems ?? []).map((system) => {
          if (system.id !== selectedSystemId) return system;

          if (elementType === "container") {
            const newContainer: ContainerJSON = {
              id: newId.trim(),
              label: newLabel.trim() || undefined,
              technology: newTechnology.trim() || undefined,
            };
            return {
              ...system,
              containers: [...(system.containers ?? []), newContainer],
            };
          } else if (elementType === "datastore") {
            const newDatastore: DataStoreJSON = {
              id: newId.trim(),
              label: newLabel.trim() || undefined,
            };
            return {
              ...system,
              datastores: [...(system.datastores ?? []), newDatastore],
            };
          } else {
            const newQueue: QueueJSON = {
              id: newId.trim(),
              label: newLabel.trim() || undefined,
            };
            return {
              ...system,
              queues: [...(system.queues ?? []), newQueue],
            };
          }
        }),
      },
    }));
    setNewId("");
    setNewLabel("");
    setNewTechnology("");
  };

  const removeElement = (type: ElementType, id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: (arch.architecture.systems ?? []).map((system) => {
          if (system.id !== selectedSystemId) return system;

          if (type === "container") {
            return { ...system, containers: (system.containers ?? []).filter((c) => c.id !== id) };
          } else if (type === "datastore") {
            return { ...system, datastores: (system.datastores ?? []).filter((d) => d.id !== id) };
          } else {
            return { ...system, queues: (system.queues ?? []).filter((q) => q.id !== id) };
          }
        }),
      },
    }));
  };

  if (systems.length === 0) {
    return (
      <div className="wizard-step-content">
        <div className="step-placeholder">
          <h2>No Systems Defined</h2>
          <p>Go back and add at least one system to add containers to.</p>
          <div className="step-navigation">
            <Button variant="secondary" onClick={onBack}>
              ← Back to Systems
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
          <Box size={24} />
        </div>
        <div className="step-header-content">
          <h2>Add Containers</h2>
          <p>Break down each system into containers, databases, and message queues.</p>
        </div>
      </div>

      <BestPracticeTip variant="tip" show={totalElements === 0}>
        <strong>C4 Level 2</strong> — Containers are deployable units: web apps, APIs, databases,
        queues. Example: "React Frontend", "Node.js API", "PostgreSQL Database"
      </BestPracticeTip>

      {/* System Selector */}
      <div className="step-section">
        <h3>Select System</h3>
        <div className="system-tabs">
          {systems.map((system) => {
            const count =
              (system.containers?.length ?? 0) +
              (system.datastores?.length ?? 0) +
              (system.queues?.length ?? 0);
            return (
              <button
                key={system.id}
                className={`system-tab ${selectedSystemId === system.id ? "active" : ""}`}
                onClick={() => setSelectedSystemId(system.id)}
              >
                <Building2 size={14} />
                {system.label || system.id}
                <span className="count-badge">{count}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Elements List */}
      <div className="step-section">
        <h3>
          Elements in {selectedSystem?.label || selectedSystemId}
          <span className="count-badge">
            {containers.length + datastores.length + queues.length}
          </span>
        </h3>

        <div className="items-list">
          {containers.map((c) => (
            <div key={c.id} className="item-card">
              <Box size={16} className="item-icon container" />
              <div className="item-info">
                <span className="item-id">{c.id}</span>
                {c.label && <span className="item-label">{c.label}</span>}
                {c.technology && <span className="item-tech">{c.technology}</span>}
              </div>
              <span className="item-badge container">Container</span>
              <button className="item-remove" onClick={() => removeElement("container", c.id)}>
                <Trash2 size={14} />
              </button>
            </div>
          ))}
          {datastores.map((d) => (
            <div key={d.id} className="item-card">
              <Database size={16} className="item-icon datastore" />
              <div className="item-info">
                <span className="item-id">{d.id}</span>
                {d.label && <span className="item-label">{d.label}</span>}
              </div>
              <span className="item-badge datastore">Datastore</span>
              <button className="item-remove" onClick={() => removeElement("datastore", d.id)}>
                <Trash2 size={14} />
              </button>
            </div>
          ))}
          {queues.map((q) => (
            <div key={q.id} className="item-card">
              <MessageSquare size={16} className="item-icon queue" />
              <div className="item-info">
                <span className="item-id">{q.id}</span>
                {q.label && <span className="item-label">{q.label}</span>}
              </div>
              <span className="item-badge queue">Queue</span>
              <button className="item-remove" onClick={() => removeElement("queue", q.id)}>
                <Trash2 size={14} />
              </button>
            </div>
          ))}
          {containers.length === 0 && datastores.length === 0 && queues.length === 0 && (
            <p className="empty-message">No elements in this system yet</p>
          )}
        </div>

        {/* Add Form */}
        <div className="add-form">
          <div className="form-group type-select">
            <label>Type</label>
            <select
              value={elementType}
              onChange={(e) => setElementType(e.target.value as ElementType)}
            >
              <option value="container">Container</option>
              <option value="datastore">Datastore</option>
              <option value="queue">Queue</option>
            </select>
          </div>
          <Input
            label="ID"
            value={newId}
            onChange={(e) => setNewId(e.target.value.replace(/\s/g, ""))}
            placeholder={
              elementType === "container"
                ? "API"
                : elementType === "datastore"
                  ? "DB"
                  : "OrderQueue"
            }
            onKeyDown={(e) => e.key === "Enter" && addElement()}
          />
          <Input
            label="Label"
            value={newLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            placeholder={elementType === "container" ? "API Service" : "PostgreSQL"}
            onKeyDown={(e) => e.key === "Enter" && addElement()}
          />
          {elementType === "container" && (
            <Input
              label="Technology"
              value={newTechnology}
              onChange={(e) => setNewTechnology(e.target.value)}
              placeholder="e.g., Node.js"
              onKeyDown={(e) => e.key === "Enter" && addElement()}
            />
          )}
          <Button variant="secondary" onClick={addElement} disabled={!newId.trim()}>
            <Plus size={16} />
            Add
          </Button>
        </div>
      </div>

      <BestPracticeTip variant="warning" show={containers.length > 0 && datastores.length === 0}>
        Don't forget to add a database! Most systems need persistent storage.
      </BestPracticeTip>

      {/* L2 Relations */}
      {l2Elements.length >= 2 && (
        <RelationsSection
          fromElements={l2Elements}
          toElements={l2Elements}
          filterFn={(rel) => {
            // Show L2 relations (one dot in path = System.Container)
            const fromParts = rel.from.split(".");
            const toParts = rel.to.split(".");
            return fromParts.length === 2 && toParts.length === 2;
          }}
          title="L2 Relations"
          description="Connect containers within or across systems (e.g., API → Database)"
        />
      )}

      {/* L2 Governance */}
      {l2Elements.length > 0 && (
        <GovernanceSection
          elements={l2Elements}
          levelLabel="container-level"
          filterFn={(tags) => {
            if (!tags || tags.length === 0) return false;
            // Show items tagged with L2 elements (one dot)
            return tags.some((t) => t.split(".").length === 2);
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
            {isComplete ? "Ready to add components!" : "Add at least one container"}
          </span>
          <Button variant="primary" onClick={onNext} disabled={!isComplete}>
            Continue to Components →
          </Button>
        </div>
      </div>
    </div>
  );
}

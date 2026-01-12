import { useState, useMemo } from "react";
import { Box, Database, MessageSquare, Plus, Trash2, Building2, Edit } from "lucide-react";
import { Button } from "@sruja/ui"; // Removed Input
import { useArchitectureStore } from "../../stores/architectureStore";
import { useViewStore } from "../../stores/viewStore";
import { useSelectionStore } from "../../stores";
import { useEffect } from "react";
import { BestPracticeTip } from "@sruja/ui";
import "@sruja/ui/components/BestPracticeTip.css";
import { EditContainerForm, EditDataStoreForm, EditQueueForm } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { ElementDump, RelationDump } from "@sruja/shared";
import "./WizardSteps.css";

interface ContainersStepProps {
  onNext: () => void;
  onBack: () => void;
  readOnly?: boolean;
}

type ElementType = "container" | "datastore" | "queue";

export function ContainersStep({
  onNext,
  onBack,
  readOnly: _readOnly = false,
}: ContainersStepProps) {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const drillDown = useViewStore((s) => s.drillDown);

  // Derive systems from flat elements
  const allElements = useMemo(
    () => Object.values(data?.elements || {}) as ElementDump[],
    [data?.elements]
  );
  const systems = useMemo(
    () => allElements.filter((e: ElementDump) => e.kind === "system"),
    [allElements]
  );

  // Form state
  // We need to handle case where selectedSystemId might not maintain validity if systems change, but for now init is fine.
  // Infer initial system selection:
  // 1. Check if a specific system is selected in global state (e.g. user right-clicked "MySystem" -> "Add Container")
  // 2. Fallback to first system
  const globalSelectedId = useSelectionStore((s) => s.selectedNodeId);

  const initialSystemId = useMemo(() => {
    if (globalSelectedId && data?.elements?.[globalSelectedId]?.kind === "system") {
      return globalSelectedId;
    }
    if (systems.length > 0) return systems[0].id; // Fallback
    return "";
  }, [globalSelectedId, systems, data]);

  const [selectedSystemId, setSelectedSystemId] = useState(initialSystemId);

  // Sync if initial changes (e.g. user navigates then opens wizard)
  useEffect(() => {
    if (initialSystemId && !selectedSystemId) {
      setSelectedSystemId(initialSystemId);
    }
  }, [initialSystemId]);

  // Update selected if empty and systems exist
  if (!selectedSystemId && systems.length > 0) {
    setSelectedSystemId((systems[0] as ElementDump).id);
  }

  // Auto-Zoom: Focus diagram on selected system
  useEffect(() => {
    if (selectedSystemId) {
      drillDown(selectedSystemId, "system");
    }
  }, [selectedSystemId, drillDown]);

  // Modal State
  const [isContainerFormOpen, setIsContainerFormOpen] = useState(false);
  const [editContainer, setEditContainer] = useState<ElementDump | undefined>(undefined);

  const [isDataStoreFormOpen, setIsDataStoreFormOpen] = useState(false);
  const [editDataStore, setEditDataStore] = useState<ElementDump | undefined>(undefined);

  const [isQueueFormOpen, setIsQueueFormOpen] = useState(false);
  const [editQueue, setEditQueue] = useState<ElementDump | undefined>(undefined);

  const selectedSystem = systems.find((s: ElementDump) => s.id === selectedSystemId);

  // Filter children by ID hierarchy (systemId.childId)
  const systemChildren = useMemo(() => {
    if (!selectedSystemId) return [];
    return allElements.filter((e: ElementDump) => {
      if (e.kind === "system" || e.kind === "person" || e.kind === "actor") return false;
      // Check if parent is selectedSystemId. Assuming dot notation or exact parent checking if we had it.
      // Using dot notation: id starts with "systemId." and has no other dots?
      // Actually L2 elements might just check if they start with systemId.
      // But strict hierarchy: systemId.childId.
      const parts = e.id.split(".");
      return parts.length === 2 && parts[0] === selectedSystemId;
    });
  }, [allElements, selectedSystemId]);

  const containers = systemChildren.filter((e) => e.kind === "container");
  const datastores = systemChildren.filter((e) => e.kind === "datastore" || e.kind === "database"); // Handle both?
  const queues = systemChildren.filter((e) => e.kind === "queue");

  // Calculate total L2 elements across all systems for validation
  const totalElements = useMemo(() => {
    // Count all elements that are children of ANY system
    return allElements.filter((e: ElementDump) => {
      const parts = e.id.split(".");
      return parts.length === 2 && systems.some((s: ElementDump) => s.id === parts[0]);
    }).length;
  }, [allElements, systems]);

  const isComplete = totalElements > 0;

  // Build L2 elements for relations (all containers, datastores, queues across all systems)
  const l2Elements = useMemo(() => {
    return allElements
      .filter((e: ElementDump) => {
        const parts = e.id.split(".");
        return parts.length === 2 && systems.some((s: ElementDump) => s.id === parts[0]);
      })
      .map((e: ElementDump) => ({
        id: e.id,
        label: e.title,
        type: e.kind,
      }));
  }, [allElements, systems]);

  const removeElement = (_type: ElementType, id: string) => {
    updateArchitecture((model) => {
      const newElements = { ...model.elements };
      delete newElements[id];
      return {
        ...model,
        elements: newElements,
      };
    });
  };

  if (systems.length === 0) {
    return (
      <div className="wizard-step-content">
        <div className="step-placeholder">
          <h2>No Systems Defined Yet</h2>
          <p>
            You need to add at least one system first. Go back to the previous step to add systems.
          </p>
          <p className="placeholder-hint">
            💡 <strong>Tip:</strong> A system is your main application or service (e.g., "E-commerce
            Platform", "Mobile App")
          </p>
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

      <BestPracticeTip variant="tip" show={totalElements === 0} stepId="containers">
        <strong>What are Containers?</strong> — Containers are the deployable parts of your system.
        Think of them as the building blocks: web apps, APIs, databases, and message queues.
        <br />
        <strong>Examples:</strong> "React Frontend", "Node.js API", "PostgreSQL Database", "Redis
        Queue"
      </BestPracticeTip>

      <EditContainerForm
        isOpen={isContainerFormOpen}
        onClose={() => setIsContainerFormOpen(false)}
        container={editContainer}
        parentSystemId={selectedSystemId}
        initialName=""
      />
      <EditDataStoreForm
        isOpen={isDataStoreFormOpen}
        onClose={() => setIsDataStoreFormOpen(false)}
        dataStore={editDataStore}
        parentSystemId={selectedSystemId}
        initialName=""
      />
      <EditQueueForm
        isOpen={isQueueFormOpen}
        onClose={() => setIsQueueFormOpen(false)}
        queue={editQueue}
        parentSystemId={selectedSystemId}
        initialName=""
      />

      {/* System Selector */}
      <div className="step-section">
        <h3>Select System</h3>
        <div className="system-tabs">
          {systems.map((system) => {
            const count = allElements.filter((e: ElementDump) => {
              const parts = e.id.split(".");
              return parts.length === 2 && parts[0] === system.id;
            }).length;

            return (
              <Button
                key={system.id}
                variant={selectedSystemId === system.id ? "primary" : "ghost"}
                size="sm"
                className={`system-tab ${selectedSystemId === system.id ? "active" : ""}`}
                onClick={() => setSelectedSystemId(system.id)}
              >
                <Building2 size={14} />
                {system.title || system.id}
                <span className="count-badge">{count}</span>
              </Button>
            );
          })}
        </div>
      </div>

      {/* Elements List */}
      <div className="step-section">
        <h3>
          Elements in {selectedSystem?.title || selectedSystemId}
          <span className="count-badge">
            {containers.length + datastores.length + queues.length}
          </span>
        </h3>

        {containers.length === 0 && datastores.length === 0 && queues.length === 0 && (
          <div className="empty-state-hint">
            <p>
              💡 <strong>Getting Started:</strong> Add containers, databases, or queues to this
              system. For example:
            </p>
            <ul className="example-list">
              <li>
                • <strong>Web App</strong> - User-facing application (React, Vue, etc.)
              </li>
              <li>
                • <strong>API Server</strong> - Backend API (Node.js, Python, etc.)
              </li>
              <li>
                • <strong>Database</strong> - Data storage (PostgreSQL, MongoDB, etc.)
              </li>
              <li>
                • <strong>Queue</strong> - Message queue (RabbitMQ, Redis, etc.)
              </li>
            </ul>
          </div>
        )}

        <div className="items-list">
          {containers.map((c: ElementDump) => (
            <div key={c.id} className="item-card">
              <Box size={16} className="item-icon container" />
              <div className="item-info">
                <span className="item-id">{c.id}</span>
                {c.title && <span className="item-label">{c.title}</span>}
                {c.technology && <span className="item-tech">{c.technology}</span>}
              </div>
              <span className="item-badge container">Container</span>
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditContainer(c);
                    setIsContainerFormOpen(true);
                  }}
                  title="Edit container"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeElement("container", c.id)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
          {datastores.map((d: ElementDump) => (
            <div key={d.id} className="item-card">
              <Database size={16} className="item-icon datastore" />
              <div className="item-info">
                <span className="item-id">{d.id}</span>
                {d.title && <span className="item-label">{d.title}</span>}
              </div>
              <span className="item-badge datastore">Datastore</span>
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditDataStore(d);
                    setIsDataStoreFormOpen(true);
                  }}
                  title="Edit datastore"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeElement("datastore", d.id)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
          {queues.map((q: ElementDump) => (
            <div key={q.id} className="item-card">
              <MessageSquare size={16} className="item-icon queue" />
              <div className="item-info">
                <span className="item-id">{q.id}</span>
                {q.title && <span className="item-label">{q.title}</span>}
              </div>
              <span className="item-badge queue">Queue</span>
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditQueue(q);
                    setIsQueueFormOpen(true);
                  }}
                  title="Edit queue"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeElement("queue", q.id)}
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
          {containers.length === 0 && datastores.length === 0 && queues.length === 0 && (
            <p className="empty-message">No elements in this system yet</p>
          )}
        </div>

        {/* Add Actions */}
        <div className="add-actions-row" style={{ display: "flex", gap: "8px", marginTop: "16px" }}>
          <Button
            variant="secondary"
            onClick={() => {
              setEditContainer(undefined);
              setIsContainerFormOpen(true);
            }}
          >
            <Plus size={16} /> Add Container
          </Button>
          <Button
            variant="secondary"
            onClick={() => {
              setEditDataStore(undefined);
              setIsDataStoreFormOpen(true);
            }}
          >
            <Plus size={16} /> Add DB
          </Button>
          <Button
            variant="secondary"
            onClick={() => {
              setEditQueue(undefined);
              setIsQueueFormOpen(true);
            }}
          >
            <Plus size={16} /> Add Queue
          </Button>
        </div>
      </div>

      <BestPracticeTip
        variant="warning"
        show={containers.length > 0 && datastores.length === 0}
        stepId="containers"
      >
        Don't forget to add a database! Most systems need persistent storage.
      </BestPracticeTip>

      {/* L2 Relations */}
      {l2Elements.length >= 2 && (
        <RelationsSection
          fromElements={l2Elements}
          toElements={l2Elements}
          filterFn={(rel: RelationDump) => {
            // Show L2 relations (one dot in path = System.Container)
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
            return fromParts.length === 2 && toParts.length === 2;
          }}
          title="L2 Relations"
          description="Connect containers within or across systems (e.g., API → Database)"
        />
      )}

      <BestPracticeTip variant="info" show={l2Elements.length > 0} stepId="containers-progress">
        Great progress! You have {l2Elements.length} container-level element(s). Continue building
        your architecture, then add goals & requirements in the final step to formalize and tag them
        to your elements.
      </BestPracticeTip>

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

import { useState, useMemo } from "react";
import { Users, Building2, Plus, Trash2, ExternalLink, Edit } from "lucide-react";
import { Button } from "@sruja/ui"; // Removed Input
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "@sruja/ui";
import "@sruja/ui/components/BestPracticeTip.css";
import { EditPersonForm, EditSystemForm } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { ElementDump } from "@sruja/shared";
import "./WizardSteps.css";

interface SystemContextStepProps {
  onNext: () => void;
  onBack: () => void;
  readOnly?: boolean;
}

export function SystemContextStep({
  onNext,
  onBack,
  readOnly: _readOnly = false,
}: SystemContextStepProps) {
  const data = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // SrujaModelDump uses flat elements map
  const elements = useMemo(
    () => (data?.elements ? (Object.values(data.elements) as ElementDump[]) : []),
    [data?.elements]
  );

  const persons = useMemo(
    () => elements.filter((e) => e.kind === "person" || e.kind === "actor" || e.kind === "user"),
    [elements]
  );
  const systems = useMemo(() => elements.filter((e) => e.kind === "system"), [elements]);

  // Form state
  const [isPersonFormOpen, setIsPersonFormOpen] = useState(false);
  const [editPerson, setEditPerson] = useState<ElementDump | undefined>(undefined);

  const [isSystemFormOpen, setIsSystemFormOpen] = useState(false);
  const [editSystem, setEditSystem] = useState<ElementDump | undefined>(undefined);

  const isComplete = persons.length > 0 || systems.length > 0;

  const removePerson = (id: string) => {
    // Requires updating elements map - this logic is complex with flat map if updateArchitecture expects mutation.
    // Assuming updateArchitecture can handle basic updates or we just warn.
    // Actually updateArchitecture expects a callback on the FULL model.
    updateArchitecture((model) => {
      const newElements = { ...(model.elements || {}) };
      delete newElements[id];
      return { ...model, elements: newElements };
    });
  };

  const removeSystem = (id: string) => {
    updateArchitecture((model) => {
      const newElements = { ...(model.elements || {}) };
      delete newElements[id];
      return { ...model, elements: newElements };
    });
  };

  const isExternal = (system: ElementDump) => {
    return (
      (system.tags && system.tags.includes("external")) ||
      (system.links && system.links.some((m) => m.title === "external"))
    );
    // Legacy metadata is gone. Check tags mostly.
  };

  // Elements for relations (L1: persons and systems)
  const personElements = useMemo(
    () => persons.map((p) => ({ id: p.id, label: p.title || p.id, type: "person" })),
    [persons]
  );

  const systemElements = useMemo(
    () => systems.map((s) => ({ id: s.id, label: s.title || s.id, type: "system" })),
    [systems]
  );

  const allL1Elements = useMemo(
    () => [...personElements, ...systemElements],
    [personElements, systemElements]
  );

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <Users size={24} />
        </div>
        <div className="step-header-content">
          <h2>Define System Context</h2>
          <p>Who uses your system? What external systems do you interact with?</p>
        </div>
      </div>

      <BestPracticeTip
        variant="tip"
        show={persons.length === 0 && systems.length === 0}
        stepId="context"
      >
        <strong>Getting Started:</strong> Think about who uses your system and what your system
        does.
        <br />
        <strong>Examples:</strong> Actors like "Customer", "Admin", or "Mobile App". Systems like
        "E-commerce Platform" or "Payment Service".
      </BestPracticeTip>

      {/* Modals */}
      <EditPersonForm
        isOpen={isPersonFormOpen}
        onClose={() => setIsPersonFormOpen(false)}
        person={editPerson}
      />
      <EditSystemForm
        isOpen={isSystemFormOpen}
        onClose={() => setIsSystemFormOpen(false)}
        system={editSystem}
      />

      {/* Persons Section */}
      <div className="step-section">
        <h3>
          <Users size={18} />
          Actors (Persons)
          <span className="count-badge">{persons.length}</span>
        </h3>
        <p className="section-description">
          People or roles that interact with your system. Examples: Customer, Admin, Mobile App User
        </p>

        {persons.length === 0 && (
          <div className="empty-state-hint">
            <p>
              💡 <strong>Tip:</strong> Start by adding who uses your system. For example:
            </p>
            <ul className="example-list">
              <li>
                • <strong>Customer</strong> - People who buy products
              </li>
              <li>
                • <strong>Admin</strong> - People who manage the system
              </li>
              <li>
                • <strong>Mobile App</strong> - External application
              </li>
            </ul>
          </div>
        )}

        <div className="items-list">
          {persons.map((person: ElementDump) => (
            <div key={person.id} className="item-card">
              <Users size={16} className="item-icon" />
              <div className="item-info">
                <span className="item-id">{person.id}</span>
                {person.title && <span className="item-label">{person.title}</span>}
              </div>
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditPerson(person);
                    setIsPersonFormOpen(true);
                  }}
                  title="Edit actor"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removePerson(person.id)}
                  title="Remove actor"
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
        </div>

        <div className="add-form">
          <Button
            variant="secondary"
            onClick={() => {
              setEditPerson(undefined);
              setIsPersonFormOpen(true);
            }}
          >
            <Plus size={16} />
            Add Actor
          </Button>
        </div>
      </div>

      {/* Systems Section */}
      <div className="step-section">
        <h3>
          <Building2 size={18} />
          Systems
          <span className="count-badge">{systems.length}</span>
        </h3>
        <p className="section-description">
          Your main system and any external systems it interacts with. Examples: "E-commerce
          Platform", "Payment Service", "Email Service"
        </p>

        {systems.length === 0 && (
          <div className="empty-state-hint">
            <p>
              💡 <strong>Tip:</strong> Add your main system and any external services it uses. For
              example:
            </p>
            <ul className="example-list">
              <li>
                • <strong>E-commerce Platform</strong> - Your main application
              </li>
              <li>
                • <strong>Payment Service</strong> - External payment provider (mark as external)
              </li>
              <li>
                • <strong>Email Service</strong> - External email provider (mark as external)
              </li>
            </ul>
          </div>
        )}

        <div className="items-list">
          {systems.map((system: ElementDump) => (
            <div key={system.id} className={`item-card ${isExternal(system) ? "external" : ""}`}>
              <Building2 size={16} className="item-icon" />
              <div className="item-info">
                <span className="item-id">{system.id}</span>
                {system.title && <span className="item-label">{system.title}</span>}
              </div>
              {isExternal(system) && (
                <span className="item-badge external">
                  <ExternalLink size={12} />
                  External
                </span>
              )}
              <div className="item-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-edit"
                  onClick={() => {
                    setEditSystem(system);
                    setIsSystemFormOpen(true);
                  }}
                  title="Edit system"
                >
                  <Edit size={14} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeSystem(system.id)}
                  title="Remove system"
                >
                  <Trash2 size={14} />
                </Button>
              </div>
            </div>
          ))}
        </div>

        <div className="add-form">
          <Button
            variant="secondary"
            onClick={() => {
              setEditSystem(undefined);
              setIsSystemFormOpen(true);
            }}
          >
            <Plus size={16} />
            Add System
          </Button>
        </div>
      </div>

      <BestPracticeTip variant="info" show={systems.length >= 1 && persons.length >= 1}>
        Great! You have {persons.length} actor(s) and {systems.length} system(s). Next step: add
        containers to break down your systems.
      </BestPracticeTip>

      <BestPracticeTip variant="warning" show={systems.length > 5}>
        Too many systems at L1 can be overwhelming. Consider grouping related systems or moving
        details to L2 (containers).
      </BestPracticeTip>

      {/* L1 Relations */}
      {allL1Elements.length >= 2 && (
        <RelationsSection
          fromElements={allL1Elements}
          toElements={allL1Elements}
          filterFn={(rel) => {
            // Show only L1 relations (no dots in source/target)
            // Handle FqnRef format: source/target are { model: string } objects
            const srcFqn =
              typeof rel.source === "object" && rel.source?.model
                ? rel.source.model
                : String(rel.source || "");
            const tgtFqn =
              typeof rel.target === "object" && rel.target?.model
                ? rel.target.model
                : String(rel.target || "");
            return !srcFqn.includes(".") && !tgtFqn.includes(".");
          }}
          title="Connect Actors to Systems"
          description="Show how actors interact with your systems (e.g., User → WebApp, Admin → API)"
        />
      )}

      {/* L1 Governance */}
      {allL1Elements.length > 0 && (
        <GovernanceSection
          elements={allL1Elements}
          levelLabel="system-level"
          filterFn={(tags) => {
            if (!tags || tags.length === 0) return false;
            // Show items tagged with L1 elements (no dots)
            return tags.some((t) => !t.includes("."));
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
            {isComplete ? "Ready to add containers!" : "Add at least one actor or system"}
          </span>
          <Button variant="primary" onClick={onNext} disabled={!isComplete}>
            Continue to Containers →
          </Button>
        </div>
      </div>
    </div>
  );
}

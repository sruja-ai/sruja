import { useState, useMemo } from "react";
import { Users, Building2, Plus, Trash2, ExternalLink } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "../shared";
import { RelationsSection } from "./RelationsSection";
import { GovernanceSection } from "./GovernanceSection";
import type { PersonJSON, SystemJSON } from "../../types";
import "./WizardSteps.css";

interface SystemContextStepProps {
  onNext: () => void;
  onBack: () => void;
}

export function SystemContextStep({ onNext, onBack }: SystemContextStepProps) {
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const persons = data?.architecture?.persons ?? [];
  const systems = data?.architecture?.systems ?? [];

  // Form state
  const [newPersonId, setNewPersonId] = useState("");
  const [newPersonLabel, setNewPersonLabel] = useState("");
  const [newSystemId, setNewSystemId] = useState("");
  const [newSystemLabel, setNewSystemLabel] = useState("");
  const [newSystemExternal, setNewSystemExternal] = useState(false);

  const isComplete = persons.length > 0 || systems.length > 0;

  // Add person
  const addPerson = () => {
    if (!newPersonId.trim() || !data?.architecture) return;
    const newPerson: PersonJSON = {
      id: newPersonId.trim(),
      label: newPersonLabel.trim() || undefined,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        persons: [...(arch.architecture.persons ?? []), newPerson],
      },
    }));
    setNewPersonId("");
    setNewPersonLabel("");
  };

  const removePerson = (id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        persons: (arch.architecture.persons ?? []).filter((p) => p.id !== id),
      },
    }));
  };

  // Add system
  const addSystem = () => {
    if (!newSystemId.trim() || !data?.architecture) return;
    const newSystem: SystemJSON = {
      id: newSystemId.trim(),
      label: newSystemLabel.trim() || undefined,
      metadata: newSystemExternal ? [{ key: "external", value: "true" }] : undefined,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: [...(arch.architecture.systems ?? []), newSystem],
      },
    }));
    setNewSystemId("");
    setNewSystemLabel("");
    setNewSystemExternal(false);
  };

  const removeSystem = (id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        systems: (arch.architecture.systems ?? []).filter((s) => s.id !== id),
      },
    }));
  };

  const isExternal = (system: SystemJSON) => {
    return system.metadata?.some((m) => m.key === "external" && m.value === "true");
  };

  // Elements for relations (L1: persons and systems)
  const personElements = useMemo(
    () => persons.map((p) => ({ id: p.id, label: p.label || p.id, type: "person" })),
    [persons]
  );

  const systemElements = useMemo(
    () => systems.map((s) => ({ id: s.id, label: s.label || s.id, type: "system" })),
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

      <BestPracticeTip variant="tip" show={persons.length === 0 && systems.length === 0}>
        <strong>Start with actors</strong> — Identify who uses your system: Users, Admins, External
        Services. This is C4 Level 1 (System Context).
      </BestPracticeTip>

      {/* Persons Section */}
      <div className="step-section">
        <h3>
          <Users size={18} />
          Actors (Persons)
          <span className="count-badge">{persons.length}</span>
        </h3>
        <p className="section-description">People or roles that interact with your system</p>

        <div className="items-list">
          {persons.map((person) => (
            <div key={person.id} className="item-card">
              <Users size={16} className="item-icon" />
              <div className="item-info">
                <span className="item-id">{person.id}</span>
                {person.label && <span className="item-label">{person.label}</span>}
              </div>
              <button
                className="item-remove"
                onClick={() => removePerson(person.id)}
                title="Remove actor"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>

        <div className="add-form">
          <Input
            label="ID"
            value={newPersonId}
            onChange={(e) => setNewPersonId(e.target.value.replace(/\s/g, ""))}
            placeholder="e.g., User"
            onKeyDown={(e) => e.key === "Enter" && addPerson()}
          />
          <Input
            label="Label (optional)"
            value={newPersonLabel}
            onChange={(e) => setNewPersonLabel(e.target.value)}
            placeholder="e.g., End User"
            onKeyDown={(e) => e.key === "Enter" && addPerson()}
          />
          <Button variant="secondary" onClick={addPerson} disabled={!newPersonId.trim()}>
            <Plus size={16} />
            Add
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
          Your main system and any external systems it interacts with
        </p>

        <div className="items-list">
          {systems.map((system) => (
            <div key={system.id} className={`item-card ${isExternal(system) ? "external" : ""}`}>
              <Building2 size={16} className="item-icon" />
              <div className="item-info">
                <span className="item-id">{system.id}</span>
                {system.label && <span className="item-label">{system.label}</span>}
              </div>
              {isExternal(system) && (
                <span className="item-badge external">
                  <ExternalLink size={12} />
                  External
                </span>
              )}
              <button
                className="item-remove"
                onClick={() => removeSystem(system.id)}
                title="Remove system"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>

        <div className="add-form">
          <Input
            label="ID"
            value={newSystemId}
            onChange={(e) => setNewSystemId(e.target.value.replace(/\s/g, ""))}
            placeholder="e.g., WebApp"
            onKeyDown={(e) => e.key === "Enter" && addSystem()}
          />
          <Input
            label="Label (optional)"
            value={newSystemLabel}
            onChange={(e) => setNewSystemLabel(e.target.value)}
            placeholder="e.g., Web Application"
            onKeyDown={(e) => e.key === "Enter" && addSystem()}
          />
          <div className="form-group checkbox-row">
            <input
              type="checkbox"
              id="systemExternal"
              checked={newSystemExternal}
              onChange={(e) => setNewSystemExternal(e.target.checked)}
            />
            <label htmlFor="systemExternal">External system</label>
          </div>
          <Button variant="secondary" onClick={addSystem} disabled={!newSystemId.trim()}>
            <Plus size={16} />
            Add
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
            // Show only L1 relations (no dots in from/to)
            return !rel.from.includes(".") && !rel.to.includes(".");
          }}
          title="L1 Relations"
          description="Connect actors to systems (e.g., User → WebApp)"
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

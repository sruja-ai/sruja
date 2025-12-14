import { useState, useMemo } from "react";
import { GitBranch, Play, Plus, Trash2, ArrowRight } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useUIStore } from "../../stores/uiStore";
import { BestPracticeTip } from "../shared";
import type { ScenarioJSON, ScenarioStepJSON } from "../../types";
import "./WizardSteps.css";

interface FlowsStepProps {
  onBack: () => void;
}

export function FlowsStep({ onBack }: FlowsStepProps) {
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  const scenarios = data?.architecture?.scenarios ?? [];

  // Build all elements for step dropdowns
  const allElements = useMemo(() => {
    const systems = data?.architecture?.systems ?? [];
    const persons = data?.architecture?.persons ?? [];
    const result: { id: string; label: string }[] = [];

    persons.forEach((p) => {
      result.push({ id: p.id, label: p.label || p.id });
    });

    systems.forEach((system) => {
      result.push({ id: system.id, label: system.label || system.id });
      (system.containers ?? []).forEach((c) => {
        result.push({ id: `${system.id}.${c.id}`, label: c.label || c.id });
        (c.components ?? []).forEach((comp) => {
          result.push({ id: `${system.id}.${c.id}.${comp.id}`, label: comp.label || comp.id });
        });
      });
      (system.datastores ?? []).forEach((d) => {
        result.push({ id: `${system.id}.${d.id}`, label: d.label || d.id });
      });
      (system.queues ?? []).forEach((q) => {
        result.push({ id: `${system.id}.${q.id}`, label: q.label || q.id });
      });
    });

    return result;
  }, [data]);

  // Form state
  const [selectedScenarioId, setSelectedScenarioId] = useState<string | null>(null);
  const [newScenarioId, setNewScenarioId] = useState("");
  const [newScenarioTitle, setNewScenarioTitle] = useState("");
  const [stepFrom, setStepFrom] = useState("");
  const [stepTo, setStepTo] = useState("");
  const [stepDescription, setStepDescription] = useState("");

  const selectedScenario = scenarios.find((s) => s.id === selectedScenarioId);

  const addScenario = () => {
    if (!newScenarioId.trim() || !newScenarioTitle.trim() || !data?.architecture) return;
    const newScenario: ScenarioJSON = {
      id: newScenarioId.trim(),
      title: newScenarioTitle.trim(),
      steps: [],
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        scenarios: [...(arch.architecture.scenarios ?? []), newScenario],
      },
    }));
    setSelectedScenarioId(newScenarioId.trim());
    setNewScenarioId("");
    setNewScenarioTitle("");
  };

  const removeScenario = (id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        scenarios: (arch.architecture.scenarios ?? []).filter((s) => s.id !== id),
      },
    }));
    if (selectedScenarioId === id) {
      setSelectedScenarioId(null);
    }
  };

  const addStep = () => {
    if (!stepFrom || !stepTo || !selectedScenarioId) return;
    const newStep: ScenarioStepJSON = {
      from: stepFrom,
      to: stepTo,
      description: stepDescription.trim() || undefined,
      order: (selectedScenario?.steps?.length ?? 0) + 1,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        scenarios: (arch.architecture.scenarios ?? []).map((s) => {
          if (s.id !== selectedScenarioId) return s;
          return { ...s, steps: [...(s.steps ?? []), newStep] };
        }),
      },
    }));
    setStepFrom("");
    setStepTo("");
    setStepDescription("");
  };

  const removeStep = (index: number) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        scenarios: (arch.architecture.scenarios ?? []).map((s) => {
          if (s.id !== selectedScenarioId) return s;
          return { ...s, steps: (s.steps ?? []).filter((_, i) => i !== index) };
        }),
      },
    }));
  };

  const getElementLabel = (id: string) => {
    const el = allElements.find((e) => e.id === id);
    return el?.label || id;
  };

  const finish = () => {
    setActiveTab("diagram");
  };

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon flows">
          <GitBranch size={24} />
        </div>
        <div className="step-header-content">
          <h2>Flows & Scenarios</h2>
          <p>Define user journeys and data flows through your system.</p>
        </div>
      </div>

      <BestPracticeTip variant="tip" show={scenarios.length === 0}>
        <strong>Scenarios show behavior</strong> — Define step-by-step flows like "User Login" or
        "Order Checkout". These can be visualized as sequence diagrams.
      </BestPracticeTip>

      {/* Scenarios List */}
      <div className="step-section">
        <h3>
          <Play size={18} />
          Scenarios
          <span className="count-badge">{scenarios.length}</span>
        </h3>

        <div className="scenario-tabs">
          {scenarios.map((scenario) => (
            <button
              key={scenario.id}
              className={`scenario-tab ${selectedScenarioId === scenario.id ? "active" : ""}`}
              onClick={() => setSelectedScenarioId(scenario.id)}
            >
              <Play size={14} />
              {scenario.title || scenario.id}
              <span className="count-badge">{scenario.steps?.length ?? 0}</span>
              <button
                className="scenario-remove"
                onClick={(e) => {
                  e.stopPropagation();
                  removeScenario(scenario.id);
                }}
              >
                <Trash2 size={12} />
              </button>
            </button>
          ))}
          <div className="add-scenario-inline">
            <Input
              label=""
              value={newScenarioId}
              onChange={(e) => setNewScenarioId(e.target.value.replace(/\s/g, ""))}
              placeholder="ID"
            />
            <Input
              label=""
              value={newScenarioTitle}
              onChange={(e) => setNewScenarioTitle(e.target.value)}
              placeholder="Title (e.g., User Login)"
              onKeyDown={(e) => e.key === "Enter" && addScenario()}
            />
            <Button
              variant="secondary"
              onClick={addScenario}
              disabled={!newScenarioId.trim() || !newScenarioTitle.trim()}
            >
              <Plus size={16} />
            </Button>
          </div>
        </div>
      </div>

      {/* Selected Scenario Steps */}
      {selectedScenario && (
        <div className="step-section">
          <h3>
            Steps in "{selectedScenario.title}"
            <span className="count-badge">{selectedScenario.steps?.length ?? 0}</span>
          </h3>

          <div className="items-list steps-list">
            {(selectedScenario.steps ?? []).map((step, index) => (
              <div key={index} className="item-card step-card">
                <span className="step-number">{index + 1}</span>
                <span className="relation-element from">{getElementLabel(step.from)}</span>
                <ArrowRight size={14} className="relation-arrow-small" />
                <span className="relation-element to">{getElementLabel(step.to)}</span>
                {step.description && <span className="step-desc">"{step.description}"</span>}
                <button className="item-remove" onClick={() => removeStep(index)}>
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
            {(selectedScenario.steps ?? []).length === 0 && (
              <p className="empty-message">No steps yet. Add the first step below.</p>
            )}
          </div>

          {/* Add Step Form */}
          <div className="add-form step-form">
            <div className="form-group">
              <select value={stepFrom} onChange={(e) => setStepFrom(e.target.value)}>
                <option value="">From...</option>
                {allElements.map((el) => (
                  <option key={el.id} value={el.id}>
                    {el.id}
                  </option>
                ))}
              </select>
            </div>
            <ArrowRight size={16} className="form-arrow" />
            <div className="form-group">
              <select value={stepTo} onChange={(e) => setStepTo(e.target.value)}>
                <option value="">To...</option>
                {allElements
                  .filter((e) => e.id !== stepFrom)
                  .map((el) => (
                    <option key={el.id} value={el.id}>
                      {el.id}
                    </option>
                  ))}
              </select>
            </div>
            <Input
              label=""
              value={stepDescription}
              onChange={(e) => setStepDescription(e.target.value)}
              placeholder="Description (e.g., submits credentials)"
              onKeyDown={(e) => e.key === "Enter" && addStep()}
            />
            <Button variant="secondary" onClick={addStep} disabled={!stepFrom || !stepTo}>
              <Plus size={16} />
              Add Step
            </Button>
          </div>
        </div>
      )}

      {!selectedScenario && scenarios.length > 0 && (
        <div className="step-section">
          <p className="empty-message">Select a scenario above to add steps.</p>
        </div>
      )}

      {/* Navigation */}
      <div className="step-navigation">
        <Button variant="secondary" onClick={onBack}>
          ← Back
        </Button>
        <div className="step-nav-right">
          <span className="step-nav-hint">Flows are optional. You can view your diagram now!</span>
          <Button variant="primary" onClick={finish}>
            Finish & View Diagram →
          </Button>
        </div>
      </div>
    </div>
  );
}

import { useState } from "react";
import { Target, Lightbulb, Plus, Trash2, LayoutTemplate } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "../shared";
import { TemplateGallery } from "./TemplateGallery";
import type { RequirementJSON } from "../../types";
import "./WizardSteps.css";

interface GoalsStepProps {
  onNext: () => void;
}

export function GoalsStep({ onNext }: GoalsStepProps) {
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Goals are in overview.goals
  const goals = data?.architecture?.overview?.goals ?? [];
  const requirements = data?.architecture?.requirements ?? [];

  const [newGoal, setNewGoal] = useState("");
  const [newRequirement, setNewRequirement] = useState("");
  const [reqType, setReqType] = useState<"functional" | "non-functional">("functional");
  const [showTemplates, setShowTemplates] = useState(false);

  const isComplete = goals.length > 0 || requirements.length > 0;

  const addGoal = () => {
    if (!newGoal.trim() || !data?.architecture) return;
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        overview: {
          ...arch.architecture.overview,
          goals: [...goals, newGoal.trim()],
        },
      },
    }));
    setNewGoal("");
  };

  const removeGoal = (index: number) => {
    if (!data?.architecture) return;
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        overview: {
          ...arch.architecture.overview,
          goals: goals.filter((_: string, i: number) => i !== index),
        },
      },
    }));
  };

  const addRequirement = () => {
    if (!newRequirement.trim() || !data?.architecture) return;
    const id = `req-${Date.now()}`;
    const newReq: RequirementJSON = {
      id,
      description: newRequirement.trim(),
      type: reqType,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        requirements: [...(arch.architecture.requirements ?? []), newReq],
      },
    }));
    setNewRequirement("");
  };

  const removeRequirement = (id: string) => {
    if (!data?.architecture) return;
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        requirements: (arch.architecture.requirements ?? []).filter((r) => r.id !== id),
      },
    }));
  };

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <Target size={24} />
        </div>
        <div className="step-header-content">
          <h2>Define Goals & Requirements</h2>
          <p>What are you building and why? Start with the business context.</p>
        </div>
      </div>

      {/* Quick Start with Template */}
      <div className="template-prompt">
        <button className="template-prompt-btn" onClick={() => setShowTemplates(true)}>
          <LayoutTemplate size={18} />
          <span>Start from a Template</span>
        </button>
        <span className="template-prompt-hint">or define your own below</span>
      </div>

      <TemplateGallery isOpen={showTemplates} onClose={() => setShowTemplates(false)} />

      {/* Goals Section */}
      <div className="step-section">
        <h3>
          <Lightbulb size={18} />
          Goals
          <span className="count-badge">{goals.length}</span>
        </h3>
        <p className="section-description">
          High-level objectives this architecture should achieve
        </p>

        <div className="items-list">
          {goals.map((goal: string, index: number) => (
            <div key={index} className="item-card">
              <span className="item-text">{goal}</span>
              <button className="item-remove" onClick={() => removeGoal(index)} title="Remove goal">
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>

        <div className="add-form">
          <Input
            label=""
            value={newGoal}
            onChange={(e) => setNewGoal(e.target.value)}
            placeholder="e.g., Support 1M concurrent users"
            onKeyDown={(e) => e.key === "Enter" && addGoal()}
          />
          <Button variant="secondary" onClick={addGoal} disabled={!newGoal.trim()}>
            <Plus size={16} />
            Add Goal
          </Button>
        </div>
      </div>

      {/* Requirements Section */}
      <div className="step-section">
        <h3>
          <Target size={18} />
          Requirements
          <span className="count-badge">{requirements.length}</span>
        </h3>
        <p className="section-description">Specific functional and non-functional requirements</p>

        <div className="items-list">
          {requirements.map((req) => (
            <div key={req.id} className="item-card">
              <span className={`item-type ${req.type ?? "functional"}`}>
                {req.type === "non-functional" ? "NFR" : "FR"}
              </span>
              <span className="item-text">{req.description ?? req.title ?? req.id}</span>
              <button
                className="item-remove"
                onClick={() => removeRequirement(req.id)}
                title="Remove requirement"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>

        <div className="add-form">
          <div className="form-group type-select">
            <select
              value={reqType}
              onChange={(e) => setReqType(e.target.value as "functional" | "non-functional")}
            >
              <option value="functional">Functional</option>
              <option value="non-functional">Non-functional</option>
            </select>
          </div>
          <Input
            label=""
            value={newRequirement}
            onChange={(e) => setNewRequirement(e.target.value)}
            placeholder={
              reqType === "functional"
                ? "e.g., Users can reset their password via email"
                : "e.g., API response time < 200ms"
            }
            onKeyDown={(e) => e.key === "Enter" && addRequirement()}
          />
          <Button variant="secondary" onClick={addRequirement} disabled={!newRequirement.trim()}>
            <Plus size={16} />
            Add
          </Button>
        </div>
      </div>

      <BestPracticeTip variant="info" show={requirements.length >= 3}>
        Great start! You have {requirements.length} requirements defined. You can always add more
        later.
      </BestPracticeTip>

      {/* Navigation */}
      <div className="step-navigation">
        <div className="step-nav-hint">
          {isComplete
            ? "Ready to define your system context!"
            : "Add at least one goal or requirement to continue"}
        </div>
        <Button variant="primary" onClick={onNext} disabled={!isComplete}>
          Continue to System Context →
        </Button>
      </div>
    </div>
  );
}

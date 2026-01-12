import { useState } from "react";
import { Target, Lightbulb, Plus, Trash2, Edit } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "@sruja/ui";
import "@sruja/ui/components/BestPracticeTip.css";
import { EditRequirementForm } from "../shared";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import type { RequirementDump, SrujaModelDump } from "@sruja/shared";
import "./WizardSteps.css";

interface GoalsStepProps {
  onNext: () => void;
  onBack?: () => void;
  readOnly?: boolean;
}

export function GoalsStep({ onNext, onBack, readOnly = false }: GoalsStepProps) {
  const data = useArchitectureStore((s) => s.model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const sruja = (data as SrujaModelDump)?.sruja ?? {};
  // Goals are in sruja.goals
  // Fix for type mismatch with SrujaExtensions from shared
  const goals = (sruja as unknown as { goals?: string[] }).goals ?? [];
  const allRequirements: RequirementDump[] = [...(sruja.requirements ?? [])];
  // Deduplicate requirements to prevent rendering duplicates
  const requirements = deduplicateRequirements(allRequirements);

  const [newGoal, setNewGoal] = useState("");

  // Requirement Form State
  const [isRequirementFormOpen, setIsRequirementFormOpen] = useState(false);
  const [editRequirement, setEditRequirement] = useState<RequirementDump | undefined>(undefined);

  const isComplete = goals.length > 0 || requirements.length > 0;

  const addGoal = () => {
    if (!newGoal.trim() || !data) return;

    updateArchitecture((model) => {
      const currentSruja = (model as SrujaModelDump).sruja || {};
      const currentGoals = currentSruja.goals || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          goals: [...currentGoals, newGoal.trim()],
        },
      };
    });
    setNewGoal("");
  };

  const removeGoal = (index: number) => {
    if (!data) return;
    updateArchitecture((model) => {
      const currentSruja = (model as SrujaModelDump).sruja || {};
      const currentGoals = currentSruja.goals || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          goals: currentGoals.filter((_: string, i: number) => i !== index),
        },
      };
    });
  };

  const removeRequirement = (id: string) => {
    if (!data) return;
    updateArchitecture((model) => {
      const currentSruja = (model as SrujaModelDump).sruja || {};
      const currentReqs = currentSruja.requirements || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          requirements: currentReqs.filter((r) => r.id !== id),
        },
      };
    });
  };

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <Target size={24} />
        </div>
        <div className="step-header-content">
          <h2>Define Goals & Requirements</h2>
          <p>
            Formalize your architecture by adding goals & requirements and tagging them to your
            elements.
          </p>
        </div>
      </div>

      <EditRequirementForm
        isOpen={isRequirementFormOpen}
        onClose={() => setIsRequirementFormOpen(false)}
        requirement={editRequirement}
      />

      {/* Template option removed from final step - users should have already built architecture */}

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
              {!readOnly && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-remove"
                  onClick={() => removeGoal(index)}
                  title="Remove goal"
                >
                  <Trash2 size={14} />
                </Button>
              )}
            </div>
          ))}
        </div>

        {!readOnly && (
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
        )}
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
          {requirements.map((req: RequirementDump) => (
            <div key={req.id} className="item-card">
              <span className={`item-type ${req.type ?? "functional"}`}>
                {req.type === "non-functional" ? "NFR" : "FR"}
              </span>
              <span className="item-text">{req.description ?? req.title ?? req.id}</span>
              {!readOnly && (
                <div className="item-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="item-edit"
                    onClick={() => {
                      setEditRequirement(req);
                      setIsRequirementFormOpen(true);
                    }}
                    title="Edit requirement"
                  >
                    <Edit size={14} />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="item-remove"
                    onClick={() => removeRequirement(req.id)}
                    title="Remove requirement"
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
              )}
            </div>
          ))}
        </div>

        {!readOnly && (
          <div className="add-form">
            <Button
              variant="secondary"
              onClick={() => {
                setEditRequirement(undefined);
                setIsRequirementFormOpen(true);
              }}
            >
              <Plus size={16} />
              Add Requirement
            </Button>
          </div>
        )}
      </div>

      <BestPracticeTip variant="info" show={requirements.length >= 3} stepId="goals">
        Great start! You have {requirements.length} requirements defined. You can always add more
        later.
      </BestPracticeTip>

      <BestPracticeTip variant="info" show={!isComplete && !readOnly} stepId="goals-final">
        💡 <strong>Formalize your architecture:</strong> Now that you've built your architecture,
        add goals & requirements and tag them to your elements (systems, containers, components).
        This creates traceability - you can see which requirements each element fulfills.
      </BestPracticeTip>

      {/* Navigation */}
      <div className="step-navigation">
        {onBack && (
          <Button variant="secondary" onClick={onBack}>
            ← Back
          </Button>
        )}
        <div className="step-nav-hint">
          {isComplete
            ? "Great! You've formalized your architecture with goals & requirements."
            : readOnly
              ? "No goals or requirements defined yet"
              : "Add goals & requirements to document what your architecture achieves"}
        </div>
        {!readOnly && (
          <Button variant="primary" onClick={onNext} disabled={false}>
            Finish →
          </Button>
        )}
      </div>
    </div>
  );
}

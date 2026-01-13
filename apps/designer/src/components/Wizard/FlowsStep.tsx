import { ArrowRight, Plus } from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { BestPracticeTip } from "../shared/BestPracticeTip";
import "./WizardSteps.css";

interface FlowsStepProps {
  onBack: () => void;
  readOnly?: boolean;
}

export function FlowsStep({ onBack, readOnly = false }: FlowsStepProps) {
  const data = useArchitectureStore((s) => s.model);
  // scenarios are in sruja.scenarios
  const scenarios = data?.sruja?.scenarios ?? [];

  return (
    <div className="wizard-step-content">
      <div className="step-header">
        <div className="step-icon">
          <ArrowRight size={24} />
        </div>
        <div className="step-header-content">
          <h2>Define Flows</h2>
          <p>Describe how users interact with your system through scenarios.</p>
        </div>
      </div>

      <BestPracticeTip variant="info" show={scenarios.length === 0} stepId="flows">
        <strong>What are Scenarios?</strong> — Scenarios show step-by-step how users and systems
        work together.
        <br />
        <strong>Example:</strong> "User logs in" → "API validates credentials" → "Database stores
        session" → "User sees dashboard"
      </BestPracticeTip>

      <div className="step-section">
        {scenarios.length === 0 && (
          <div className="empty-state-hint">
            <p>
              💡 <strong>Tip:</strong> Scenarios help document how your system works. You can add
              them later or skip this step.
            </p>
            <p
              style={{ marginTop: "0.5rem", fontSize: "0.8125rem", color: "var(--text-tertiary)" }}
            >
              This feature is coming soon. For now, you can continue to the next step.
            </p>
          </div>
        )}

        <div className="items-list">
          {scenarios.map((s, i) => (
            <div key={i} className="item-card">
              <span className="item-text">{s.title || "Untitled Scenario"}</span>
            </div>
          ))}
        </div>

        {!readOnly && (
          <div className="add-form">
            <Button variant="secondary" disabled>
              <Plus size={16} />
              Add Scenario (Coming Soon)
            </Button>
          </div>
        )}
      </div>
      <div className="step-navigation">
        <Button variant="ghost" onClick={onBack}>
          ← Back
        </Button>
      </div>
    </div>
  );
}

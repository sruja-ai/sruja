import { X } from "lucide-react";
import "./WizardStepper.css";

export interface WizardStep {
  id: string;
  label: string;
  description?: string;
  isComplete: boolean;
  isLocked: boolean;
}

interface WizardStepperProps {
  steps: WizardStep[];
  currentStep: number;
  onStepClick: (stepIndex: number) => void;
  onClose?: () => void;
  extraActions?: React.ReactNode;
}

export function WizardStepper({
  steps,
  currentStep,
  onStepClick,
  onClose,
  extraActions,
}: WizardStepperProps) {
  const currentLabel = steps[currentStep]?.label || "";

  return (
    <div className="wizard-progress-container">
      {/* Header with Title and Counter */}
      <div className="wizard-progress-header">
        <div className="current-step-info">
          <span className="current-step-label">{currentLabel}</span>
          {/* <span className="current-step-desc"> - {currentDesc}</span> */}
        </div>

        <div
          className="wizard-header-actions"
          style={{ display: "flex", alignItems: "center", gap: "8px" }}
        >
          <div className="step-counter">
            Step {currentStep + 1} of {steps.length}
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="wizard-close-btn"
              style={{
                background: "transparent",
                border: "none",
                cursor: "pointer",
                color: "var(--text-tertiary)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                padding: "4px",
                borderRadius: "4px",
                marginLeft: "8px",
              }}
              title="Close Wizard"
            >
              <X size={16} />
            </button>
          )}
        </div>
      </div>

      {/* The Progress Track */}
      <div className="progress-track">
        {steps.map((step, index) => {
          const isActive = index === currentStep;
          const isCompleted = step.isComplete || index < currentStep;
          const isLocked = step.isLocked;

          return (
            <div
              key={step.id}
              className={`progress-segment 
                            ${isActive ? "active" : ""} 
                            ${isCompleted ? "completed" : ""} 
                            ${isLocked ? "locked" : ""}
                        `}
              onClick={() => !isLocked && onStepClick(index)}
              title={`${step.label}${isLocked ? " (Locked)" : ""}`}
            />
          );
        })}
      </div>

      {/* Optional: Extra Actions (like Share) */}
      {extraActions && (
        <div
          className="wizard-extra-actions"
          style={{ marginTop: "8px", display: "flex", justifyContent: "flex-end" }}
        >
          {extraActions}
        </div>
      )}
    </div>
  );
}

import { X } from "lucide-react";
import "./WizardStepper.css";

export interface WizardStep {
  id: string;
  label: string;
  description?: string;
  isComplete: boolean;
  isLocked: boolean;
  isOptional?: boolean;
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
  const currentStepInfo = steps[currentStep];
  const currentLabel = currentStepInfo?.label || "";
  const currentDesc = currentStepInfo?.description || "";
  const currentOptional = currentStepInfo?.isOptional === true;

  return (
    <div className="wizard-progress-container">
      {/* Header with Title and Counter */}
      <div className="wizard-progress-header">
        <div className="current-step-info">
          <span className="current-step-label">{currentLabel}</span>
          {currentDesc && <span className="current-step-desc"> - {currentDesc}</span>}
          {currentOptional && <span className="step-optional-pill">Optional</span>}
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
              onClick={() => onStepClick(index)}
              title={`${step.label}${step.description ? ` - ${step.description}` : ""}${isLocked ? " (Prerequisites not met)" : ""}`}
            />
          );
        })}
      </div>

      {/* Step Labels Row - Clickable Steps */}
      <div className="step-labels-row">
        {steps.map((step, index) => {
          const isActive = index === currentStep;
          const isCompleted = step.isComplete || index < currentStep;
          const isLocked = step.isLocked;

          return (
            <button
              key={step.id}
              type="button"
              className={`step-label-button
                            ${isActive ? "active" : ""} 
                            ${isCompleted ? "completed" : ""} 
                            ${isLocked ? "locked" : ""}
                        `}
              onClick={() => onStepClick(index)}
              disabled={false}
              title={`${step.label}${step.description ? ` - ${step.description}` : ""}${isLocked ? " (Prerequisites not met)" : ""}`}
            >
              <span className="step-label-number">{index + 1}</span>
              <span className="step-label-text">{step.label}</span>
              {step.isOptional && <span className="step-optional-badge">Optional</span>}
              {isCompleted && !isActive && <span className="step-label-check">✓</span>}
            </button>
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

import { Check, Lock } from "lucide-react";
import { Button } from "@sruja/ui";
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
  extraActions?: React.ReactNode;
}

export function WizardStepper({
  steps,
  currentStep,
  onStepClick,
  extraActions,
}: WizardStepperProps) {
  const progress = (currentStep / (steps.length - 1)) * 100;
  const completedSteps = steps.filter((s) => s.isComplete).length;

  return (
    <div className="wizard-stepper">
      {/* Progress Bar */}
      <div className="wizard-progress">
        <div className="wizard-progress-track">
          <div className="wizard-progress-fill" style={{ width: `${progress}%` }} />
        </div>
        <span className="wizard-progress-text">
          {completedSteps}/{steps.length} complete
        </span>
        {extraActions && <div className="wizard-actions">{extraActions}</div>}
      </div>

      {/* Steps */}
      <div className="wizard-steps">
        {steps.map((step, index) => {
          const isActive = index === currentStep;
          const isPast = index < currentStep;
          const isClickable = step.isComplete || index <= currentStep;

          return (
            <Button
              key={step.id}
              variant={isActive ? "primary" : "ghost"}
              size="sm"
              className={`wizard-step ${isActive ? "active" : ""} ${isPast ? "past" : ""} ${step.isComplete ? "complete" : ""} ${step.isLocked ? "locked" : ""}`}
              onClick={() => isClickable && onStepClick(index)}
              disabled={step.isLocked && !isClickable}
              title={step.description}
            >
              <div className="wizard-step-indicator">
                {step.isComplete ? (
                  <Check size={16} />
                ) : step.isLocked ? (
                  <Lock size={14} />
                ) : (
                  <span className="wizard-step-number">{index + 1}</span>
                )}
              </div>
              <div className="wizard-step-content">
                <span className="wizard-step-label">{step.label}</span>
                {step.description && (
                  <span className="wizard-step-description">{step.description}</span>
                )}
              </div>
            </Button>
          );
        })}
      </div>
    </div>
  );
}

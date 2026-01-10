/**
 * Step Description Overlay
 *
 * Displays the description of the current animation step.
 * Shows from → to flow with elegant animations.
 */

import type { StepDump } from "@sruja/shared";
import "../../components/Nodes/nodes.css";

interface StepDescriptionOverlayProps {
  currentStep: number;
  totalSteps: number;
  stepData: StepDump | null;
  className?: string;
}

export function StepDescriptionOverlay({
  currentStep,
  totalSteps,
  stepData,
  className,
}: StepDescriptionOverlayProps) {
  // Only show if we have data
  if (!stepData) return null;

  // Extract meaningful names from FQN (e.g., "System.Container" → "Container")
  const formatName = (fqn: string): string => {
    if (!fqn) return "";
    const parts = fqn.split(".");
    return parts[parts.length - 1]; // Return last part
  };

  return (
    <div className={`step-overlay ${className || ""}`}>
      <div className="step-number">
        Step {currentStep + 1} of {totalSteps}
      </div>

      {stepData.from && stepData.to && (
        <div className="step-from-to">
          <span className="step-from">{formatName(stepData.from)}</span>
          <span className="step-arrow">→</span>
          <span className="step-to">{formatName(stepData.to)}</span>
        </div>
      )}

      {stepData.description && <div className="step-description">{stepData.description}</div>}
    </div>
  );
}

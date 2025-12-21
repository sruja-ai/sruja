/**
 * Animation Controls Component
 * 
 * Provides UI controls for flow animation: play/pause, step navigation,
 * progress indicator, and settings.
 */

import { Play, Pause, SkipBack, SkipForward, RotateCcw } from "lucide-react";
import { Button } from "@sruja/ui";
import { useSelectionStore } from "../../stores/viewStore";
import type { FlowDump } from "@sruja/shared";
import "./AnimationControls.css";

interface AnimationControlsProps {
  flow: FlowDump | null;
  className?: string;
}

export function AnimationControls({ flow, className }: AnimationControlsProps) {
  const activeFlow = useSelectionStore((s) => s.activeFlow);
  const flowStep = useSelectionStore((s) => s.flowStep);
  const isFlowPlaying = useSelectionStore((s) => s.isFlowPlaying);
  const playFlow = useSelectionStore((s) => s.playFlow);
  const pauseFlow = useSelectionStore((s) => s.pauseFlow);
  const nextStep = useSelectionStore((s) => s.nextStep);
  const prevStep = useSelectionStore((s) => s.prevStep);
  const setFlowStep = useSelectionStore((s) => s.setFlowStep);

  const currentFlow = activeFlow || flow;
  const totalSteps = currentFlow?.steps?.length ?? 0;
  const currentStep = flowStep;
  const hasSteps = totalSteps > 0;
  const isAtStart = currentStep === 0;
  const isAtEnd = currentStep >= totalSteps - 1;
  const currentStepData = currentFlow?.steps?.[currentStep];

  const progressPercentage = totalSteps > 0 ? ((currentStep + 1) / totalSteps) * 100 : 0;

  const handlePlayPause = () => {
    if (isFlowPlaying) {
      pauseFlow();
    } else {
      playFlow();
    }
  };

  const handleRestart = () => {
    setFlowStep(0);
    pauseFlow();
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      prevStep();
    }
  };

  const handleNext = () => {
    if (currentStep < totalSteps - 1) {
      nextStep();
    }
  };

  if (!currentFlow || !hasSteps) {
    return null;
  }

  return (
    <div className={`animation-controls ${className || ""}`} role="region" aria-label="Flow animation controls">
      <div className="animation-controls-main">
        {/* Play/Pause Button */}
        <Button
          variant="primary"
          size="sm"
          onClick={handlePlayPause}
          disabled={!hasSteps}
          aria-label={isFlowPlaying ? "Pause animation" : "Play animation"}
          title={isFlowPlaying ? "Pause" : "Play"}
        >
          {isFlowPlaying ? <Pause size={16} /> : <Play size={16} />}
        </Button>

        {/* Step Navigation */}
        <div className="animation-controls-steps">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRestart}
            disabled={isAtStart}
            aria-label="Restart animation"
            title="Restart"
          >
            <RotateCcw size={14} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handlePrev}
            disabled={isAtStart || isFlowPlaying}
            aria-label="Previous step"
            title="Previous step"
          >
            <SkipBack size={14} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleNext}
            disabled={isAtEnd || isFlowPlaying}
            aria-label="Next step"
            title="Next step"
          >
            <SkipForward size={14} />
          </Button>
        </div>
      </div>

      {/* Progress Indicator */}
      <div className="animation-controls-progress">
        <div className="animation-controls-progress-bar">
          <div
            className="animation-controls-progress-fill"
            style={{ width: `${progressPercentage}%` }}
            role="progressbar"
            aria-valuenow={currentStep + 1}
            aria-valuemin={1}
            aria-valuemax={totalSteps}
            aria-label={`Step ${currentStep + 1} of ${totalSteps}`}
          />
        </div>
        <div className="animation-controls-progress-text">
          <span className="animation-controls-step-counter">
            Step {currentStep + 1} of {totalSteps}
          </span>
          {currentStepData && (
            <span className="animation-controls-step-description" title={currentStepData.description}>
              {currentStepData.description}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}


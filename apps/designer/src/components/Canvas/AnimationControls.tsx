/**
 * Animation Controls Component
 *
 * Provides UI controls for flow animation: play/pause, step navigation,
 * progress indicator, and settings.
 */

import { Play, Pause, SkipBack, SkipForward, RotateCcw } from "lucide-react";
import { Button } from "@sruja/ui";
import { useSelectionStore } from "../../stores/viewStore";
import type { FlowDump, ScenarioDump } from "@sruja/shared";
import "./AnimationControls.css";

interface AnimationControlsProps {
  animation: FlowDump | ScenarioDump | null;
  className?: string;
}

export function AnimationControls({ animation, className }: AnimationControlsProps) {
  const activeAnimation = useSelectionStore((s) => s.activeAnimation);
  const animationStep = useSelectionStore((s) => s.animationStep);
  const isAnimationPlaying = useSelectionStore((s) => s.isAnimationPlaying);
  const playAnimation = useSelectionStore((s) => s.playAnimation);
  const pauseAnimation = useSelectionStore((s) => s.pauseAnimation);
  const nextStep = useSelectionStore((s) => s.nextStep);
  const prevStep = useSelectionStore((s) => s.prevStep);
  const setAnimationStep = useSelectionStore((s) => s.setAnimationStep);

  const currentAnimation = activeAnimation || animation;
  const totalSteps = currentAnimation?.steps?.length ?? 0;
  const currentStep = animationStep;
  const hasSteps = totalSteps > 0;
  const isAtStart = currentStep === 0;
  const isAtEnd = currentStep >= totalSteps - 1;
  const currentStepData = currentAnimation?.steps?.[currentStep];

  const progressPercentage = totalSteps > 0 ? ((currentStep + 1) / totalSteps) * 100 : 0;

  const handlePlayPause = () => {
    if (isAnimationPlaying) {
      pauseAnimation();
    } else {
      playAnimation();
    }
  };

  const handleRestart = () => {
    setAnimationStep(0);
    pauseAnimation();
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

  if (!currentAnimation || !hasSteps) {
    return null;
  }

  return (
    <div
      className={`animation-controls ${className || ""}`}
      role="region"
      aria-label="Animation controls"
    >
      <div className="animation-controls-main">
        {/* Play/Pause Button */}
        <Button
          variant="primary"
          size="sm"
          onClick={handlePlayPause}
          disabled={!hasSteps}
          aria-label={isAnimationPlaying ? "Pause animation" : "Play animation"}
          title={isAnimationPlaying ? "Pause" : "Play"}
        >
          {isAnimationPlaying ? <Pause size={16} /> : <Play size={16} />}
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
            disabled={isAtStart || isAnimationPlaying}
            aria-label="Previous step"
            title="Previous step"
          >
            <SkipBack size={14} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleNext}
            disabled={isAtEnd || isAnimationPlaying}
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
            <span
              className="animation-controls-step-description"
              title={currentStepData.description}
            >
              {currentStepData.description}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

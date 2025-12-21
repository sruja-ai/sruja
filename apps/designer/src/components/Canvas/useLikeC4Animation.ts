import { useEffect, useRef } from "react";
import { useSelectionStore } from "../../stores/viewStore";
import { AnimationController, VisualEffectsSystem } from "../../utils/animation";

interface UseLikeC4AnimationProps {
  containerRef: React.RefObject<HTMLDivElement | null>;
  activeFlow: any | null; // typing 'any' for now to match strictness elsewhere, ideally should be typed
  isFlowPlaying: boolean;
  flowStep: number;
}

export function useLikeC4Animation({
  containerRef,
  activeFlow,
  isFlowPlaying,
  flowStep,
}: UseLikeC4AnimationProps) {
  const animationControllerRef = useRef<AnimationController | null>(null);
  const visualEffectsRef = useRef<VisualEffectsSystem | null>(null);

  // Initialize animation engine
  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    if (!visualEffectsRef.current) {
      visualEffectsRef.current = new VisualEffectsSystem();
      visualEffectsRef.current.initialize(containerRef.current);
    }

    if (!animationControllerRef.current) {
      animationControllerRef.current = new AnimationController({
        onStepChange: (step) => {
          useSelectionStore.getState().setFlowStep(step);
        },
        onStateChange: (state) => {
          if (visualEffectsRef.current) {
            visualEffectsRef.current.updateStepVisuals(
              state.activeNodes,
              state.activeEdges,
              state.visitedNodes,
              state.visitedEdges
            );
          }
        },
      });
    }

    return () => {
      animationControllerRef.current?.destroy();
      visualEffectsRef.current?.destroy();
    };
  }, [containerRef]);

  // Sync animation controller with active flow
  useEffect(() => {
    if (!animationControllerRef.current) {
      return;
    }

    animationControllerRef.current.setFlow(activeFlow);

    if (activeFlow) {
      if (isFlowPlaying && !animationControllerRef.current.isPlaying()) {
        animationControllerRef.current.play();
      } else if (!isFlowPlaying && animationControllerRef.current.isPlaying()) {
        animationControllerRef.current.pause();
      }

      const currentStep = animationControllerRef.current.getCurrentStep();
      if (currentStep !== flowStep) {
        animationControllerRef.current.goToStep(flowStep);
      }
    }
  }, [activeFlow, isFlowPlaying, flowStep]);

  return {
    animationControllerRef,
    visualEffectsRef,
  };
}

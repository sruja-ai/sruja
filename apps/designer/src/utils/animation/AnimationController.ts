/**
 * Animation Controller
 *
 * Manages animation state, timing, and playback for flow animations.
 * Provides controls for play, pause, step navigation, and configuration.
 */

import type { FlowDump, ScenarioDump, SrujaModelDump, StepDump } from "@sruja/shared";
import { resolveFqnToNodeId } from "../fqnResolver";

export type NodeState = "idle" | "active" | "highlighted" | "visited" | "pending";
export type EdgeState = "idle" | "active" | "highlighted" | "visited";

export interface AnimationState {
  isPlaying: boolean;
  currentStep: number;
  activeNodes: Set<string>;
  activeEdges: Set<string>;
  visitedNodes: Set<string>;
  visitedEdges: Set<string>;
  animationSpeed: number; // 0.5 to 2.0
  stepDuration: number; // milliseconds
  transitionDuration: number; // milliseconds
  autoAdvance: boolean;
  loop: boolean;
}

export interface AnimationConfig {
  stepDuration?: number;
  transitionDuration?: number;
  autoAdvance?: boolean;
  loop?: boolean;
  animationSpeed?: number;
  onStepChange?: (step: number, stepData: StepDump | null) => void;
  onStateChange?: (state: AnimationState) => void;
  model?: SrujaModelDump | null; // Added model for FQN resolution
}

export class AnimationController {
  private source: FlowDump | ScenarioDump | null = null;
  private state: AnimationState;
  private config: Required<AnimationConfig> & { model: SrujaModelDump | null };
  private autoAdvanceTimer: ReturnType<typeof setTimeout> | null = null;
  private prefersReducedMotion: boolean;

  constructor(config: AnimationConfig = {}) {
    this.prefersReducedMotion =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    this.config = {
      stepDuration: config.stepDuration ?? (this.prefersReducedMotion ? 1000 : 2000),
      transitionDuration: config.transitionDuration ?? (this.prefersReducedMotion ? 200 : 500),
      autoAdvance: config.autoAdvance ?? true,
      loop: config.loop ?? false,
      animationSpeed: config.animationSpeed ?? 1.0,
      onStepChange: config.onStepChange ?? (() => {}),
      onStateChange: config.onStateChange ?? (() => {}),
      model: config.model ?? null,
    };

    this.state = {
      isPlaying: false,
      currentStep: 0,
      activeNodes: new Set(),
      activeEdges: new Set(),
      visitedNodes: new Set(),
      visitedEdges: new Set(),
      animationSpeed: this.config.animationSpeed,
      stepDuration: this.config.stepDuration,
      transitionDuration: this.config.transitionDuration,
      autoAdvance: this.config.autoAdvance,
      loop: this.config.loop,
    };

    // Listen for reduced motion preference changes
    if (typeof window !== "undefined") {
      const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
      mediaQuery.addEventListener("change", (e) => {
        this.prefersReducedMotion = e.matches;
        if (e.matches) {
          this.setStepDuration(1000);
          this.setTransitionDuration(200);
        }
      });
    }
  }

  /**
   * Set the active source (flow or scenario) for animation
   */
  setSource(source: FlowDump | ScenarioDump | null): void {
    this.source = source;
    this.stop();
    this.state.currentStep = 0;
    this.clearAll();
    this.notifyStateChange();
  }

  /**
   * Update the model reference (needed for FQN resolution if model changes)
   */
  setModel(model: SrujaModelDump | null): void {
    this.config.model = model;
  }

  /**
   * Get the current source
   */
  getSource(): FlowDump | ScenarioDump | null {
    return this.source;
  }

  /**
   * Start animation playback
   */
  play(): void {
    if (!this.source || !this.hasSteps()) {
      return;
    }

    if (this.state.isPlaying) {
      return; // Already playing
    }

    this.state.isPlaying = true;
    this.notifyStateChange();

    if (this.state.autoAdvance) {
      this.scheduleNextStep();
    }
  }

  /**
   * Pause animation playback
   */
  pause(): void {
    this.state.isPlaying = false;
    this.clearAutoAdvanceTimer();
    this.notifyStateChange();
  }

  /**
   * Stop animation and reset to beginning
   */
  stop(): void {
    this.pause();
    this.state.currentStep = 0;
    this.clearAll();
    this.notifyStateChange();
  }

  /**
   * Advance to next step
   */
  nextStep(): void {
    if (!this.source || !this.hasSteps()) {
      return;
    }

    const totalSteps = this.source.steps?.length ?? 0;
    if (this.state.currentStep >= totalSteps - 1) {
      if (this.state.loop) {
        this.state.currentStep = 0;
        this.clearAll();
      } else {
        this.pause();
        return;
      }
    } else {
      this.state.currentStep++;
    }

    this.updateStepVisuals();
    this.notifyStepChange();
    this.notifyStateChange();

    if (this.state.isPlaying && this.state.autoAdvance) {
      this.scheduleNextStep();
    }
  }

  /**
   * Go to previous step
   */
  prevStep(): void {
    if (!this.source || !this.hasSteps()) {
      return;
    }

    if (this.state.currentStep > 0) {
      this.state.currentStep--;
      this.updateStepVisuals();
      this.notifyStepChange();
      this.notifyStateChange();
    }

    if (this.state.isPlaying && this.state.autoAdvance) {
      this.clearAutoAdvanceTimer();
      this.scheduleNextStep();
    }
  }

  /**
   * Jump to specific step
   */
  goToStep(step: number): void {
    if (!this.source || !this.hasSteps()) {
      return;
    }

    const totalSteps = this.source.steps?.length ?? 0;
    const clampedStep = Math.max(0, Math.min(step, totalSteps - 1));

    if (clampedStep !== this.state.currentStep) {
      this.state.currentStep = clampedStep;
      this.updateStepVisuals();
      this.notifyStepChange();
      this.notifyStateChange();
    }

    if (this.state.isPlaying && this.state.autoAdvance) {
      this.clearAutoAdvanceTimer();
      this.scheduleNextStep();
    }
  }

  /**
   * Set animation speed (0.5 to 2.0)
   */
  setSpeed(speed: number): void {
    this.state.animationSpeed = Math.max(0.5, Math.min(2.0, speed));
    this.notifyStateChange();
  }

  /**
   * Set step duration in milliseconds
   */
  setStepDuration(duration: number): void {
    this.state.stepDuration = Math.max(100, duration);
    this.notifyStateChange();
  }

  /**
   * Set transition duration in milliseconds
   */
  setTransitionDuration(duration: number): void {
    this.state.transitionDuration = Math.max(0, duration);
    this.notifyStateChange();
  }

  /**
   * Enable/disable auto-advance
   */
  setAutoAdvance(enabled: boolean): void {
    this.state.autoAdvance = enabled;
    if (enabled && this.state.isPlaying) {
      this.scheduleNextStep();
    } else {
      this.clearAutoAdvanceTimer();
    }
    this.notifyStateChange();
  }

  /**
   * Enable/disable loop mode
   */
  setLoop(enabled: boolean): void {
    this.state.loop = enabled;
    this.notifyStateChange();
  }

  /**
   * Get current step index
   */
  getCurrentStep(): number {
    return this.state.currentStep;
  }

  /**
   * Get total number of steps
   */
  getTotalSteps(): number {
    return this.source?.steps?.length ?? 0;
  }

  /**
   * Get current step data
   */
  getCurrentStepData(): StepDump | null {
    if (!this.source?.steps) {
      return null;
    }
    return this.source.steps[this.state.currentStep] ?? null;
  }

  /**
   * Check if animation is playing
   */
  isPlaying(): boolean {
    return this.state.isPlaying;
  }

  /**
   * Get current animation state
   */
  getState(): Readonly<AnimationState> {
    return { ...this.state };
  }

  /**
   * Get active nodes for current step
   */
  getActiveNodes(): ReadonlySet<string> {
    return this.state.activeNodes;
  }

  /**
   * Get active edges for current step
   */
  getActiveEdges(): ReadonlySet<string> {
    return this.state.activeEdges;
  }

  /**
   * Get visited nodes
   */
  getVisitedNodes(): ReadonlySet<string> {
    return this.state.visitedNodes;
  }

  /**
   * Get visited edges
   */
  getVisitedEdges(): ReadonlySet<string> {
    return this.state.visitedEdges;
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.stop();
    this.clearAutoAdvanceTimer();
  }

  // Private methods

  private hasSteps(): boolean {
    return (this.source?.steps?.length ?? 0) > 0;
  }

  private scheduleNextStep(): void {
    this.clearAutoAdvanceTimer();

    const duration = this.state.stepDuration / this.state.animationSpeed;
    this.autoAdvanceTimer = setTimeout(() => {
      if (this.state.isPlaying) {
        this.nextStep();
      }
    }, duration);
  }

  private clearAutoAdvanceTimer(): void {
    if (this.autoAdvanceTimer) {
      clearTimeout(this.autoAdvanceTimer);
      this.autoAdvanceTimer = null;
    }
  }

  private updateStepVisuals(): void {
    if (!this.source?.steps) {
      return;
    }

    const steps = this.source.steps;
    const currentStep = steps[this.state.currentStep];
    if (!currentStep) {
      return;
    }

    // Helper to get node ID (handles FQN resolution)
    const getNodeId = (ref: string): string => {
      // If we have a model, try to resolve FQN
      if (this.config.model) {
        const resolved = resolveFqnToNodeId(ref, this.config.model);
        if (resolved) return resolved;
      }
      return ref; // Fallback to raw string
    };

    // Clear previous active nodes/edges
    this.state.activeNodes.clear();
    this.state.activeEdges.clear();

    const fromId = currentStep.from ? getNodeId(currentStep.from) : null;
    const toId = currentStep.to ? getNodeId(currentStep.to) : null;

    // Add current step nodes
    if (fromId) {
      this.state.activeNodes.add(fromId);
      this.state.visitedNodes.add(fromId);
    }
    if (toId) {
      this.state.activeNodes.add(toId);
      this.state.visitedNodes.add(toId);
    }

    // Create edge ID from step (format: "from->to")
    if (fromId && toId) {
      // Note: Edge IDs in ReactFlow might be different (e.g. "e-from-to-0").
      // VisualEffectsSystem usually matches on data-edge-source/target or similar?
      // Or we need to construct the ID precisely.
      // Current implementation used constructs like `${from}->${to}` which matches traffic edges?
      // Let's stick to the existing convention: `${from}->${to}`
      const edgeId = `${fromId}->${toId}`;
      this.state.activeEdges.add(edgeId);
      this.state.visitedEdges.add(edgeId);
    }

    // Mark previous steps as visited
    for (let i = 0; i < this.state.currentStep; i++) {
      const step = steps[i];
      const sFrom = step?.from ? getNodeId(step.from) : null;
      const sTo = step?.to ? getNodeId(step.to) : null;

      if (sFrom) {
        this.state.visitedNodes.add(sFrom);
      }
      if (sTo) {
        this.state.visitedNodes.add(sTo);
      }
      if (sFrom && sTo) {
        const edgeId = `${sFrom}->${sTo}`;
        this.state.visitedEdges.add(edgeId);
      }
    }
  }

  private clearAll(): void {
    this.state.activeNodes.clear();
    this.state.activeEdges.clear();
    this.state.visitedNodes.clear();
    this.state.visitedEdges.clear();
  }

  private notifyStepChange(): void {
    const stepData = this.getCurrentStepData();
    this.config.onStepChange(this.state.currentStep, stepData);
  }

  private notifyStateChange(): void {
    this.config.onStateChange(this.state);
  }
}

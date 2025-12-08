import { useEffect, useRef } from 'react';
import { Play, Pause, SkipBack, SkipForward, X } from 'lucide-react';
import { useArchitectureStore, useSelectionStore } from '../../stores';
import type { ScenarioJSON, FlowJSON } from '../../types';
import './FlowController.css';

interface FlowControllerProps {
    onHighlightEdge?: (from: string, to: string) => void;
    onClearHighlight?: () => void;
}

export function FlowController({ onHighlightEdge, onClearHighlight }: FlowControllerProps) {
    const data = useArchitectureStore((s) => s.data);
    const activeFlow = useSelectionStore((s) => s.activeFlow);
    const flowStep = useSelectionStore((s) => s.flowStep);
    const isPlaying = useSelectionStore((s) => s.isFlowPlaying);
    const setActiveFlow = useSelectionStore((s) => s.setActiveFlow);
    const playFlow = useSelectionStore((s) => s.playFlow);
    const pauseFlow = useSelectionStore((s) => s.pauseFlow);
    const nextStep = useSelectionStore((s) => s.nextStep);
    const prevStep = useSelectionStore((s) => s.prevStep);

    const timerRef = useRef<number | null>(null);

    // Auto-advance when playing
    useEffect(() => {
        if (isPlaying && activeFlow) {
            timerRef.current = window.setInterval(() => {
                nextStep();
            }, 2000); // 2 second per step
        }

        return () => {
            if (timerRef.current) {
                clearInterval(timerRef.current);
                timerRef.current = null;
            }
        };
    }, [isPlaying, activeFlow, nextStep]);

    // Stop at end
    useEffect(() => {
        if (activeFlow && flowStep >= (activeFlow.steps?.length ?? 0) - 1) {
            pauseFlow();
        }
    }, [flowStep, activeFlow, pauseFlow]);

    // Highlight current step edge
    useEffect(() => {
        if (activeFlow?.steps && flowStep < activeFlow.steps.length) {
            const step = activeFlow.steps[flowStep];
            if (onHighlightEdge) {
                onHighlightEdge(step.from, step.to);
            }
        } else if (onClearHighlight) {
            onClearHighlight();
        }
    }, [activeFlow, flowStep, onHighlightEdge, onClearHighlight]);

    const scenarios = data?.architecture?.scenarios ?? [];
    const flows = data?.architecture?.flows ?? [];
    const allFlows: (ScenarioJSON | FlowJSON)[] = [...scenarios, ...flows];

    if (allFlows.length === 0 && !activeFlow) {
        return null;
    }

    const handleSelectFlow = (flow: ScenarioJSON | FlowJSON) => {
        setActiveFlow(flow as FlowJSON);
    };

    const handleClose = () => {
        setActiveFlow(null);
        if (onClearHighlight) {
            onClearHighlight();
        }
    };

    const currentStep = activeFlow?.steps?.[flowStep];
    const maxSteps = activeFlow?.steps?.length ?? 0;

    return (
        <div className="flow-controller">
            {!activeFlow ? (
                <div className="flow-selector">
                    <span className="flow-label">Scenarios & Flows:</span>
                    <div className="flow-list">
                        {allFlows.map((flow) => (
                            <button
                                key={flow.id}
                                className="flow-item"
                                onClick={() => handleSelectFlow(flow)}
                            >
                                {flow.title || flow.label || flow.id}
                            </button>
                        ))}
                    </div>
                </div>
            ) : (
                <div className="flow-player">
                    <div className="flow-info">
                        <span className="flow-title">{activeFlow.title || activeFlow.label || activeFlow.id}</span>
                        <button className="close-flow-btn" onClick={handleClose}>
                            <X size={14} />
                        </button>
                    </div>

                    <div className="flow-controls">
                        <button
                            className="control-btn"
                            onClick={prevStep}
                            disabled={flowStep === 0}
                            title="Previous step"
                        >
                            <SkipBack size={16} />
                        </button>

                        {isPlaying ? (
                            <button className="control-btn play-btn" onClick={pauseFlow} title="Pause">
                                <Pause size={18} />
                            </button>
                        ) : (
                            <button
                                className="control-btn play-btn"
                                onClick={playFlow}
                                disabled={flowStep >= maxSteps - 1}
                                title="Play"
                            >
                                <Play size={18} />
                            </button>
                        )}

                        <button
                            className="control-btn"
                            onClick={nextStep}
                            disabled={flowStep >= maxSteps - 1}
                            title="Next step"
                        >
                            <SkipForward size={16} />
                        </button>

                        <span className="step-indicator">
                            {flowStep + 1} / {maxSteps}
                        </span>
                    </div>

                    {currentStep && (
                        <div className="step-info">
                            <div className="step-path">
                                <span className="step-from">{currentStep.from}</span>
                                <span className="step-arrow">→</span>
                                <span className="step-to">{currentStep.to}</span>
                            </div>
                            {currentStep.description && (
                                <p className="step-description">{currentStep.description}</p>
                            )}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

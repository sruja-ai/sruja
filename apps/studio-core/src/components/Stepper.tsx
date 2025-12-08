// apps/studio-core/src/components/Stepper.tsx
import React, { useMemo } from 'react';
import { useViewStore, type ViewStep } from '../stores/ViewStore';
import { CheckCircle2, Circle, CircleDot } from 'lucide-react';
import { ArchitectureJSON } from '@sruja/viewer';

interface StepperProps {
    archData: ArchitectureJSON | null;
}

const steps: { 
    id: ViewStep; 
    label: string; 
    prompt: string;
}[] = [
    { 
        id: 'context', 
        label: 'Context',
        prompt: 'Add Persons and Systems to describe who uses your system and what the system is.'
    },
    { 
        id: 'containers', 
        label: 'Containers',
        prompt: 'Break the system into deployable Containers. A typical system has 2–6 containers.'
    },
    { 
        id: 'components', 
        label: 'Components',
        prompt: 'Add Components that represent modules, capabilities, or responsibilities.'
    },
    { 
        id: 'stitch', 
        label: 'Stitch',
        prompt: 'Define relations between elements to show how they interact.'
    },
];

export const Stepper: React.FC<StepperProps> = React.memo(({ archData }) => {
    const { activeStep, setStep } = useViewStore();

    // Memoize step status calculations
    const stepStatuses = useMemo(() => {
        const activeIndex = steps.findIndex((s) => s.id === activeStep);
        
        return steps.map((step) => {
            const stepIndex = steps.findIndex((s) => s.id === step.id);
            
            if (stepIndex < activeIndex) return 'completed';
            if (stepIndex === activeIndex) return 'active';
            
            // Check if step has partial progress
            if (archData && step.id === 'context') {
                const hasPersons = (archData.architecture?.persons?.length || 0) > 0;
                const hasSystems = (archData.architecture?.systems?.length || 0) > 0;
                if (hasPersons || hasSystems) return 'partial';
            } else if (archData && step.id === 'containers') {
                const hasContainers = archData.architecture?.systems?.some(s => 
                    (s.containers?.length || 0) > 0
                );
                if (hasContainers) return 'partial';
            } else if (archData && step.id === 'components') {
                const hasComponents = archData.architecture?.systems?.some(s =>
                    s.containers?.some(c => (c.components?.length || 0) > 0)
                );
                if (hasComponents) return 'partial';
            }
            
            return 'pending';
        });
    }, [activeStep, archData]);

    const getStepStatus = (stepId: ViewStep): 'completed' | 'active' | 'partial' | 'pending' => {
        const stepIndex = steps.findIndex((s) => s.id === stepId);
        return stepStatuses[stepIndex] || 'pending';
    };

    const activeStepData = useMemo(() => steps.find(s => s.id === activeStep), [activeStep]);

    return (
        <div className="flex-1 flex flex-col overflow-hidden">
            <div className="flex-1 p-3 space-y-2 overflow-y-auto">
                {steps.map((step) => {
                    const status = getStepStatus(step.id);
                    return (
                        <button
                            key={step.id}
                            onClick={() => setStep(step.id)}
                            className={`flex items-center w-full gap-3 px-3 py-2.5 rounded-md transition-all duration-200 text-left group ${
                                status === 'active'
                                    ? 'bg-blue-900/30 text-blue-400 border border-blue-800/60 shadow-sm'
                                    : status === 'completed'
                                        ? 'text-green-400 hover:bg-gray-800/70 hover:border-gray-700 border border-transparent'
                                        : status === 'partial'
                                            ? 'text-yellow-400 hover:bg-gray-800/70 hover:border-gray-700 border border-transparent'
                                            : 'text-gray-500 hover:bg-gray-800/70 hover:text-gray-300 border border-transparent'
                            }`}
                        >
                            <div className="flex-shrink-0 transition-transform duration-200 group-hover:scale-110">
                                {status === 'completed' ? (
                                    <CheckCircle2 size={18} className="text-green-500" />
                                ) : status === 'active' ? (
                                    <CircleDot size={18} className="text-blue-500" />
                                ) : status === 'partial' ? (
                                    <Circle size={18} className="fill-current text-yellow-500" />
                                ) : (
                                    <Circle size={18} className="text-gray-600" />
                                )}
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className="text-sm font-medium">{step.label}</div>
                            </div>
                        </button>
                    );
                })}
            </div>
            
            {/* Micro-prompt for active step */}
            {activeStepData && (
                <div className="p-3 border-t border-gray-800 bg-gradient-to-b from-gray-900/80 to-gray-900/50">
                    <div className="text-xs text-gray-400 mb-2 uppercase tracking-wider font-semibold flex items-center gap-1.5">
                        <div className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
                        Current Step
                    </div>
                    <div className="text-xs text-gray-300 leading-relaxed">
                        {activeStepData.prompt}
                    </div>
                </div>
            )}

        </div>
    );
});

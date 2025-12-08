// apps/studio-core/src/components/GoalsPanel.tsx
import React, { useMemo } from 'react';
import { ArchitectureJSON } from '@sruja/viewer';
import { calculateReadiness } from '../utils/readiness';
import { CheckSquare, AlertCircle } from 'lucide-react';

interface GoalsPanelProps {
    archData: ArchitectureJSON | null;
}

export const GoalsPanel: React.FC<GoalsPanelProps> = React.memo(({ archData }) => {
    // Memoize expensive readiness calculation
    const report = useMemo(() => calculateReadiness(archData), [archData]);

    return (
        <div className="p-4 space-y-6">
            <div className="space-y-2">
                <div className="flex items-center justify-between text-sm font-medium">
                    <span className="text-[var(--color-text-secondary)]">Readiness Score</span>
                    <span className={`font-bold ${report.overallScore >= 80 ? 'text-green-400' : report.overallScore >= 50 ? 'text-yellow-400' : 'text-red-400'}`}>
                        {report.overallScore}%
                    </span>
                </div>
                <div className="w-full h-2 bg-gray-800 rounded-full overflow-hidden">
                    <div
                        className={`h-full transition-all duration-500 ease-out ${report.overallScore >= 80 ? 'bg-green-500' : report.overallScore >= 50 ? 'bg-yellow-500' : 'bg-red-500'}`}
                        style={{ width: `${report.overallScore}%` }}
                    />
                </div>
            </div>

            <div className="space-y-3">
                <h3 className="text-xs font-bold text-[var(--color-text-tertiary)] uppercase tracking-wider flex items-center gap-2">
                    <CheckSquare className="w-3 h-3" />
                    Checklist
                </h3>

                {report.items.length === 0 ? (
                    <p className="text-sm text-[var(--color-text-tertiary)] italic">No goals found.</p>
                ) : (
                    <div className="space-y-2">
                        {report.items.map((item) => (
                            <div
                                key={item.id}
                                className={`flex items-start gap-3 p-3 rounded-md border text-sm transition-colors ${item.completed
                                        ? 'bg-green-900/10 border-green-900/30 text-green-400'
                                        : 'bg-red-900/10 border-red-900/30 text-gray-300'
                                    }`}
                            >
                                <div className={`mt-0.5 ${item.completed ? 'text-green-500' : 'text-red-400'}`}>
                                    {item.completed ? <CheckSquare size={16} /> : <AlertCircle size={16} />}
                                </div>
                                <div className="flex-1">
                                    <p className="font-medium">{item.label}</p>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
});


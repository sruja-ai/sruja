import { Target, Ban, AlertTriangle, Tag, Layers } from 'lucide-react';
import type { OverviewJSON } from '../../types';

interface GoalsSectionProps {
    overview: OverviewJSON | undefined;
}

export function GoalsSection({ overview }: GoalsSectionProps) {
    if (!overview) return null;

    // Check if we have anything to render
    const hasGoals = overview.goals && overview.goals.length > 0;
    const hasNonGoals = overview.nonGoals && overview.nonGoals.length > 0;
    const hasRisks = overview.risks && overview.risks.length > 0;
    const hasAudience = !!overview.audience;
    const hasScope = !!overview.scope;

    if (!hasGoals && !hasNonGoals && !hasRisks && !hasAudience && !hasScope) {
        return null;
    }

    return (
        <>
            {/* Goals & Non-Goals */}
            {(hasGoals || hasNonGoals) && (
                <div className="overview-goals-section">
                    {hasGoals && (
                        <div className="goals-card goals">
                            <h3><Target size={16} /> Goals</h3>
                            <ul>
                                {overview.goals!.map((goal, i) => (
                                    <li key={i}>{goal}</li>
                                ))}
                            </ul>
                        </div>
                    )}
                    {hasNonGoals && (
                        <div className="goals-card non-goals">
                            <h3><Ban size={16} /> Non-Goals</h3>
                            <ul>
                                {overview.nonGoals!.map((ng, i) => (
                                    <li key={i}>{ng}</li>
                                ))}
                            </ul>
                        </div>
                    )}
                </div>
            )}

            {/* Risks */}
            {hasRisks && (
                <div className="overview-risks">
                    <h3><AlertTriangle size={16} /> Risks & Concerns</h3>
                    <div className="risks-list">
                        {overview.risks!.map((risk, i) => (
                            <div key={i} className="risk-item">
                                <AlertTriangle size={14} />
                                <span>{risk}</span>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Audience & Scope */}
            {(hasAudience || hasScope) && (
                <div className="overview-context">
                    {hasAudience && (
                        <div className="context-item">
                            <Tag size={14} />
                            <span className="context-label">Audience:</span>
                            <span>{overview.audience}</span>
                        </div>
                    )}
                    {hasScope && (
                        <div className="context-item">
                            <Layers size={14} />
                            <span className="context-label">Scope:</span>
                            <span>{overview.scope}</span>
                        </div>
                    )}
                </div>
            )}
        </>
    );
}

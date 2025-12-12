import React from 'react';
import type { DiagramQualityMetrics } from '../../utils/diagramQuality';

interface LayoutScorePanelProps {
    metrics: DiagramQualityMetrics | null;
    showHeatmap: boolean;
    onToggleHeatmap: (show: boolean) => void;
}

export const LayoutScorePanel: React.FC<LayoutScorePanelProps> = ({
    metrics,
    showHeatmap,
    onToggleHeatmap
}) => {
    if (!metrics) return null;

    const getColor = (grade: string) => {
        switch (grade) {
            case 'A': return '#4ade80'; // green-400
            case 'B': return '#a3e635'; // lime-400
            case 'C': return '#facc15'; // yellow-400
            case 'D': return '#fb923c'; // orange-400
            case 'F': return '#f87171'; // red-400
            default: return '#9ca3af';
        }
    };

    return (
        <div style={{
            position: 'absolute',
            bottom: 20,
            right: 20,
            backgroundColor: 'rgba(17, 24, 39, 0.9)', // gray-900
            padding: '12px 16px',
            borderRadius: '8px',
            color: 'white',
            border: `1px solid ${getColor(metrics.grade)}`,
            backdropFilter: 'blur(4px)',
            zIndex: 50,
            boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
            minWidth: '200px',
            fontFamily: 'system-ui, sans-serif'
        }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                <span style={{ fontWeight: 600, fontSize: '14px' }}>Layout Quality</span>
                <span style={{
                    fontWeight: 800,
                    fontSize: '16px',
                    color: getColor(metrics.grade)
                }}>
                    {metrics.weightedScore.toFixed(1)} ({metrics.grade})
                </span>
            </div>

            <div style={{ marginBottom: '12px', fontSize: '12px', color: '#d1d5db' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span>Crossings:</span>
                    <span>{metrics.edgeCrossings}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span>Overlaps:</span>
                    <span>{metrics.overlappingNodes.length}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span>Bad Flow:</span>
                    <span>{metrics.directionViolations?.length ?? 0}</span>
                </div>
                <div style={{ 
                    display: 'flex', 
                    justifyContent: 'space-between',
                    color: metrics.emptySpaceScore < 80 ? '#fbbf24' : '#d1d5db'
                }}>
                    <span>Empty Space:</span>
                    <span>{(metrics.emptySpace * 100).toFixed(0)}% ({metrics.emptySpaceScore.toFixed(0)})</span>
                </div>
            </div>

            <label style={{
                display: 'flex',
                alignItems: 'center',
                cursor: 'pointer',
                fontSize: '12px',
                userSelect: 'none'
            }}>
                <input
                    type="checkbox"
                    checked={showHeatmap}
                    onChange={(e) => onToggleHeatmap(e.target.checked)}
                    style={{ marginRight: '8px' }}
                />
                Show Badness Heatmap
            </label>
        </div>
    );
};

// components/Canvas/QualityScoreDisplay.tsx
// Display diagram quality score overlay on the canvas
import { useMemo } from 'react';
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../../types';
import { calculateDiagramQuality, type DiagramQualityMetrics } from '../../utils/diagramQuality';
import './QualityScoreDisplay.css';

interface QualityScoreDisplayProps {
    nodes: Node<C4NodeData>[];
    edges: Edge[];
    viewportSize?: { width: number; height: number };
}

export function QualityScoreDisplay({ nodes, edges, viewportSize }: QualityScoreDisplayProps) {
    const metrics = useMemo<DiagramQualityMetrics | null>(() => {
        if (nodes.length === 0) return null;
        
        try {
            return calculateDiagramQuality(
                nodes,
                edges,
                viewportSize || { width: 1920, height: 1080 }
            );
        } catch (error) {
            console.error('Error calculating quality metrics:', error);
            return null;
        }
    }, [nodes, edges, viewportSize]);

    if (!metrics) return null;

    const getGradeColor = (grade: string) => {
        switch (grade) {
            case 'A': return '#10b981'; // green
            case 'B': return '#3b82f6'; // blue
            case 'C': return '#f59e0b'; // orange
            case 'D': return '#ef4444'; // red
            case 'F': return '#dc2626'; // dark red
            default: return '#6b7280'; // gray
        }
    };

    const getScoreColor = (score: number) => {
        if (score >= 80) return '#10b981'; // green
        if (score >= 70) return '#3b82f6'; // blue
        if (score >= 60) return '#f59e0b'; // orange
        return '#ef4444'; // red
    };

    return (
        <div className="quality-score-display">
            <div className="quality-score-header">
                <span className="quality-score-label">Diagram Quality</span>
                <span 
                    className="quality-score-grade"
                    style={{ color: getGradeColor(metrics.grade) }}
                >
                    {metrics.grade}
                </span>
            </div>
            
            <div className="quality-score-main">
                <div 
                    className="quality-score-value"
                    style={{ color: getScoreColor(metrics.weightedScore) }}
                >
                    {metrics.weightedScore.toFixed(1)}
                </div>
                <div className="quality-score-out-of">/ 100</div>
            </div>

            <div className="quality-score-details">
                <div className="quality-score-item">
                    <span className="quality-score-item-label">Overall:</span>
                    <span className="quality-score-item-value">{metrics.overallScore.toFixed(1)}</span>
                </div>
                <div className="quality-score-breakdown">
                    <div className="quality-score-breakdown-item">
                        <span>Overlaps:</span>
                        <span className={metrics.overlappingNodes.length > 0 ? 'warning' : 'ok'}>
                            {metrics.overlappingNodes.length}
                        </span>
                    </div>
                    <div className="quality-score-breakdown-item">
                        <span>Crossings:</span>
                        <span className={metrics.edgeCrossings > 10 ? 'warning' : 'ok'}>
                            {metrics.edgeCrossings}
                        </span>
                    </div>
                    <div className="quality-score-breakdown-item">
                        <span>Spacing:</span>
                        <span className={metrics.spacingViolations.length > 5 ? 'warning' : 'ok'}>
                            {metrics.spacingViolations.length}
                        </span>
                    </div>
                    <div className="quality-score-breakdown-item">
                        <span>Hierarchy:</span>
                        <span className={metrics.parentChildContainment.length > 0 ? 'warning' : 'ok'}>
                            {metrics.parentChildContainment.length}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
}

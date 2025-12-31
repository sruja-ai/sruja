import { useState, useEffect } from "react";
import { CheckCircle2, XCircle, AlertCircle, BarChart3 } from "lucide-react";
import "./QualityScoreCard.css";

interface QualityMetrics {
  score: number;
  edgeCrossings: number;
  nodeOverlaps: number;
  labelOverlaps: number;
  avgEdgeLength: number;
  edgeLengthVariance: number;
  rankAlignment: number;
  clusterBalance: number;
  spacingConsistency: number;
  timestamp: number;
  nodeCount: number;
  edgeCount: number;
  level: string;
}

interface QualityScoreCardProps {
  isCollapsed?: boolean;
}

export function QualityScoreCard({ isCollapsed = false }: QualityScoreCardProps) {
  // Only show quality metrics in development environment
  // This is a developer tool, not a user-facing feature
  const isDev = import.meta.env.DEV || import.meta.env.MODE === "development";

  const [quality, setQuality] = useState<QualityMetrics | null>(null);

  // Poll for quality metrics updates (only in dev)
  useEffect(() => {
    // Skip in production
    if (!isDev) {
      return;
    }
    const checkQuality = () => {
      if (typeof window !== "undefined") {
        const metrics = (window as any).__DIAGRAM_QUALITY__ as QualityMetrics | null;
        if (metrics && metrics.timestamp) {
          // Only update if metrics are fresh (within last 30 seconds)
          const age = Date.now() - metrics.timestamp;
          if (age < 30000) {
            setQuality(metrics);
          }
        }
      }
    };

    // Check immediately
    checkQuality();

    // Poll every 1 second for updates
    const interval = setInterval(checkQuality, 1000);

    return () => clearInterval(interval);
  }, [isDev]);

  // Hide in production
  if (!isDev) {
    return null;
  }

  if (!quality) {
    return null;
  }

  const getScoreGrade = (
    score: number
  ): { grade: string; color: string; icon: React.ReactNode } => {
    if (score >= 0.85) {
      return {
        grade: "A",
        color: "var(--quality-excellent)",
        icon: <CheckCircle2 size={16} />,
      };
    } else if (score >= 0.7) {
      return {
        grade: "B",
        color: "var(--quality-good)",
        icon: <CheckCircle2 size={16} />,
      };
    } else if (score >= 0.5) {
      return {
        grade: "C",
        color: "var(--quality-fair)",
        icon: <AlertCircle size={16} />,
      };
    } else {
      return {
        grade: "F",
        color: "var(--quality-poor)",
        icon: <XCircle size={16} />,
      };
    }
  };

  const getMetricStatus = (
    value: number,
    threshold: number,
    lowerIsBetter: boolean = false
  ): "good" | "warning" | "poor" => {
    if (lowerIsBetter) {
      if (value <= threshold) return "good";
      if (value <= threshold * 2) return "warning";
      return "poor";
    } else {
      if (value >= threshold) return "good";
      if (value >= threshold * 0.8) return "warning";
      return "poor";
    }
  };

  const scoreInfo = getScoreGrade(quality.score);
  const rankAlignmentStatus = getMetricStatus(quality.rankAlignment, 0.95);
  const spacingStatus = getMetricStatus(quality.spacingConsistency, 0.8);
  const crossingsStatus = getMetricStatus(quality.edgeCrossings, 5, true);
  const overlapsStatus = quality.nodeOverlaps === 0 ? "good" : "poor";

  if (isCollapsed) {
    return (
      <div className="quality-score-card collapsed">
        <div className="quality-score-compact" style={{ color: scoreInfo.color }}>
          <BarChart3 size={16} />
          <span className="quality-grade-compact">{scoreInfo.grade}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="quality-score-card">
      <div className="quality-score-header">
        <BarChart3 size={16} />
        <span>Diagram Quality</span>
      </div>

      {/* Overall Score */}
      <div className="quality-score-main">
        <div
          className="quality-score-circle"
          style={{ color: scoreInfo.color, borderColor: scoreInfo.color }}
        >
          <div className="quality-score-value">{Math.round(quality.score * 100)}</div>
          <div className="quality-score-grade">{scoreInfo.grade}</div>
        </div>
        <div className="quality-score-info">
          <div className="quality-score-label">Overall Score</div>
          <div className="quality-score-details">
            {quality.nodeCount} nodes • {quality.edgeCount} edges • {quality.level}
          </div>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="quality-metrics-list">
        <div className={`quality-metric-item ${overlapsStatus}`}>
          <div className="quality-metric-label">Node Overlaps</div>
          <div className="quality-metric-value">
            {quality.nodeOverlaps === 0 ? (
              <>
                <CheckCircle2 size={14} />
                <span>0</span>
              </>
            ) : (
              <>
                <XCircle size={14} />
                <span>{quality.nodeOverlaps}</span>
              </>
            )}
          </div>
        </div>

        <div className={`quality-metric-item ${crossingsStatus}`}>
          <div className="quality-metric-label">Edge Crossings</div>
          <div className="quality-metric-value">
            {quality.edgeCrossings === 0 ? (
              <>
                <CheckCircle2 size={14} />
                <span>0</span>
              </>
            ) : quality.edgeCrossings <= 5 ? (
              <>
                <AlertCircle size={14} />
                <span>{quality.edgeCrossings}</span>
              </>
            ) : (
              <>
                <XCircle size={14} />
                <span>{quality.edgeCrossings}</span>
              </>
            )}
          </div>
        </div>

        <div className={`quality-metric-item ${rankAlignmentStatus}`}>
          <div className="quality-metric-label">Rank Alignment</div>
          <div className="quality-metric-value">
            {rankAlignmentStatus === "good" ? (
              <>
                <CheckCircle2 size={14} />
                <span>{(quality.rankAlignment * 100).toFixed(0)}%</span>
              </>
            ) : (
              <>
                <AlertCircle size={14} />
                <span>{(quality.rankAlignment * 100).toFixed(0)}%</span>
              </>
            )}
          </div>
        </div>

        <div className={`quality-metric-item ${spacingStatus}`}>
          <div className="quality-metric-label">Spacing Consistency</div>
          <div className="quality-metric-value">
            {spacingStatus === "good" ? (
              <>
                <CheckCircle2 size={14} />
                <span>{(quality.spacingConsistency * 100).toFixed(0)}%</span>
              </>
            ) : (
              <>
                <AlertCircle size={14} />
                <span>{(quality.spacingConsistency * 100).toFixed(0)}%</span>
              </>
            )}
          </div>
        </div>

        {quality.labelOverlaps > 0 && (
          <div className="quality-metric-item poor">
            <div className="quality-metric-label">Label Overlaps</div>
            <div className="quality-metric-value">
              <XCircle size={14} />
              <span>{quality.labelOverlaps}</span>
            </div>
          </div>
        )}
      </div>

      {/* Targets */}
      <div className="quality-targets">
        <div className="quality-target-label">Targets</div>
        <div className="quality-target-list">
          <span>Score ≥ 85</span>
          <span>•</span>
          <span>Crossings ≤ 5</span>
          <span>•</span>
          <span>Alignment ≥ 95%</span>
        </div>
      </div>
    </div>
  );
}

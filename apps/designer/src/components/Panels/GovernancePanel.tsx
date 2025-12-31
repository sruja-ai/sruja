import { useEffect, useState, useMemo } from "react";
import { Shield, AlertTriangle, Info, XCircle, RefreshCw, Award } from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useToastStore } from "../../stores/toastStore";
import { useUIStore } from "../../stores/uiStore";
import { useSelectionStore } from "../../stores/viewStore";
import { getWasmApi, logger } from "@sruja/shared";
import "./GovernancePanel.css";

// Interface for ScoreCard returned by WASM
interface Deduction {
  Rule: string;
  Points: number;
  Message: string;
  Target: string;
  Severity: "Critical" | "Warning" | "Info";
}

interface CategoryScores {
  Structural: number;
  Documentation: number;
  Traceability: number;
  Complexity: number;
  Standardization: number;
}

interface ScoreCard {
  Score: number;
  Grade: string;
  Categories: CategoryScores;
  Deductions: Deduction[];
}

export function GovernancePanel() {
  const dslSource = useArchitectureStore((s) => s.dslSource);
  const [scoreCard, setScoreCard] = useState<ScoreCard | null>(null);
  const [loading, setLoading] = useState(false);
  const showToast = useToastStore((s) => s.showToast);

  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const setCodeTab = useUIStore((s) => s.setCodeTab);
  const setTargetLine = useUIStore((s) => s.setTargetLine);
  const selectNode = useSelectionStore((s) => s.selectNode);

  const handleTargetClick = (target: string) => {
    if (!target) return;

    // Parse "filename:line" pattern, e.g. "input.sruja:46"
    const parts = target.split(":");
    if (parts.length >= 2) {
      const line = parseInt(parts[1], 10);
      if (!isNaN(line)) {
        setTargetLine(line);
        setCodeTab("dsl");
        setActiveTab("code");
        return;
      }
    }

    // Fallback: If no line but just ID (e.g. "web.api"), select it
    // which triggers DSLPanel's own ID-to-line lookup
    selectNode(target);
    setCodeTab("dsl");
    setActiveTab("code");
  };

  const calculateScore = async () => {
    if (!dslSource) {
      setScoreCard(null);
      return;
    }

    setLoading(true);
    try {
      const api = await getWasmApi();
      if (!api) {
        setScoreCard(null);
        return;
      }

      // Validate DSL by attempting to parse it first
      // This prevents score calculation errors from invalid syntax
      try {
        await api.dslToModel(dslSource);
      } catch (parseError) {
        // DSL is invalid, clear score card and return early
        setScoreCard(null);
        // Only show toast if user manually triggered the calculation
        if (loading === false) {
          showToast("DSL syntax is invalid. Please fix errors before calculating score.", "error");
        }
        return;
      }

      // DSL is valid, proceed with score calculation
      const result = await api.calculateArchitectureScore(dslSource);
      setScoreCard(result as unknown as ScoreCard);
    } catch (error) {
      // Score calculation failed (but DSL was valid)
      // Clear score card to avoid showing stale data
      setScoreCard(null);
      // Only show toast if user manually triggered the calculation
      if (loading === false) {
        showToast("Could not calculate architecture score.", "error");
      }
      // Only log in development to avoid noise in production
      if (process.env.NODE_ENV === "development") {
        logger.debug("Score calculation failed", {
          component: "GovernancePanel",
          action: "calculateScore",
          error: error instanceof Error ? error.message : String(error),
        });
      }
    } finally {
      setLoading(false);
    }
  };

  // Calculate score on mount and when DSL changes
  useEffect(() => {
    calculateScore();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dslSource]);

  const groupedDeductions = useMemo(() => {
    if (!scoreCard) return { Critical: [], Warning: [], Info: [] };

    return scoreCard.Deductions.reduce(
      (acc, d) => {
        const severity = d.Severity || "Info"; // Fallback
        if (!acc[severity]) acc[severity] = [];
        acc[severity].push(d);
        return acc;
      },
      { Critical: [], Warning: [], Info: [] } as Record<string, Deduction[]>
    );
  }, [scoreCard]);

  const getHealthColor = (grade: string) => {
    switch (grade) {
      case "A":
        return "var(--color-success)";
      case "B":
        return "var(--color-info)";
      case "C":
        return "var(--color-warning)";
      case "D":
        return "var(--color-danger)";
      default:
        return "var(--color-danger)";
    }
  };

  if (!scoreCard && loading) {
    return <div className="governance-loading">Calculating Architecture Score...</div>;
  }

  if (!scoreCard) {
    return (
      <div className="governance-empty">
        <Button onClick={calculateScore}>
          <RefreshCw size={16} className="mr-2" />
          Calculate Score
        </Button>
      </div>
    );
  }

  return (
    <div className="governance-panel">
      <div className="governance-header">
        <div className="governance-title">
          <Shield size={24} />
          <h2>Architecture Governance</h2>
        </div>
        <Button variant="ghost" size="sm" onClick={calculateScore} disabled={loading}>
          <RefreshCw size={16} className={loading ? "spin" : ""} />
        </Button>
      </div>

      <div className="governance-score-card">
        <div className="score-ring" style={{ borderColor: getHealthColor(scoreCard.Grade) }}>
          <span className="grade">{scoreCard.Grade}</span>
          <span className="score-label">{scoreCard.Score}/100</span>
        </div>
        <div className="score-summary">
          <h3>Architecture Health Index (AHI)</h3>
          <div className="ahi-metrics">
            <HealthMetric label="Structural" value={scoreCard.Categories.Structural} weight="40%" />
            <HealthMetric
              label="Documentation"
              value={scoreCard.Categories.Documentation}
              weight="20%"
            />
            <HealthMetric
              label="Traceability"
              value={scoreCard.Categories.Traceability}
              weight="15%"
            />
            <HealthMetric label="Complexity" value={scoreCard.Categories.Complexity} weight="15%" />
            <HealthMetric
              label="Standardization"
              value={scoreCard.Categories.Standardization}
              weight="10%"
            />
          </div>
        </div>
      </div>

      <div className="governance-sections">
        {/* Critical Issues */}
        {groupedDeductions.Critical.length > 0 && (
          <div className="deduction-section critical">
            <h4 className="section-title critical">
              <XCircle size={16} />
              Critical Issues ({groupedDeductions.Critical.length})
            </h4>
            <div className="deduction-list">
              {groupedDeductions.Critical.map((d, i) => (
                <div key={i} className="deduction-item">
                  <span className="deduction-points">-{d.Points}</span>
                  <div className="deduction-content">
                    <strong>{d.Rule}</strong>
                    <p>{d.Message}</p>
                    <code
                      className="target-ref clickable"
                      onClick={() => handleTargetClick(d.Target)}
                      title="Click to view in DSL editor"
                    >
                      {d.Target}
                    </code>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Warnings */}
        {groupedDeductions.Warning.length > 0 && (
          <div className="deduction-section warning">
            <h4 className="section-title warning">
              <AlertTriangle size={16} />
              Warnings ({groupedDeductions.Warning.length})
            </h4>
            <div className="deduction-list">
              {groupedDeductions.Warning.map((d, i) => (
                <div key={i} className="deduction-item">
                  <span className="deduction-points">-{d.Points}</span>
                  <div className="deduction-content">
                    <strong>{d.Rule}</strong>
                    <p>{d.Message}</p>
                    <code
                      className="target-ref clickable"
                      onClick={() => handleTargetClick(d.Target)}
                      title="Click to view in DSL editor"
                    >
                      {d.Target}
                    </code>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Recommendations */}
        {groupedDeductions.Info.length > 0 && (
          <div className="deduction-section info">
            <h4 className="section-title info">
              <Info size={16} />
              Recommendations ({groupedDeductions.Info.length})
            </h4>
            <div className="deduction-list">
              {groupedDeductions.Info.map((d, i) => (
                <div key={i} className="deduction-item">
                  <span className="deduction-points">-{d.Points}</span>
                  <div className="deduction-content">
                    <strong>{d.Rule}</strong>
                    <p>{d.Message}</p>
                    <code
                      className="target-ref clickable"
                      onClick={() => handleTargetClick(d.Target)}
                      title="Click to view in DSL editor"
                    >
                      {d.Target}
                    </code>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {scoreCard.Deductions.length === 0 && (
          <div className="governance-perfect">
            <Award size={48} />
            <h3>Perfect Score!</h3>
            <p>No issues found. Great job!</p>
          </div>
        )}
      </div>
    </div>
  );
}

function HealthMetric({ label, value, weight }: { label: string; value: number; weight: string }) {
  const getColor = (v: number) => {
    if (v >= 90) return "var(--color-success)";
    if (v >= 70) return "var(--color-warning)";
    return "var(--color-danger)";
  };

  return (
    <div className="ahi-metric-item" title={`${label} (Weight: ${weight})`}>
      <div className="ahi-metric-header">
        <span className="ahi-metric-label">{label}</span>
        <span className="ahi-metric-value">{value}%</span>
      </div>
      <div className="ahi-metric-track">
        <div
          className="ahi-metric-fill"
          style={{
            width: `${value}%`,
            backgroundColor: getColor(value),
          }}
        />
      </div>
    </div>
  );
}

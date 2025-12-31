// apps/designer/src/components/Architect/GovernanceScore.tsx
import { useEffect, useState } from "react";
import { Award, TrendingUp, TrendingDown, Minus } from "lucide-react";
import { getWasmApi, logger } from "@sruja/shared";
import { useArchitectureStore } from "../../stores";
import "./GovernanceScore.css";

interface ScoreBreakdown {
  structural: number;
  documentation: number;
  traceability: number;
  complexity: number;
  standardization: number;
  overall: number;
}

export function GovernanceScore() {
  const dslSource = useArchitectureStore((s) => s.dslSource);
  const [score, setScore] = useState<ScoreBreakdown | null>(null);
  const [loading, setLoading] = useState(false);
  const [previousScore, setPreviousScore] = useState<ScoreBreakdown | null>(null);

  useEffect(() => {
    const calculateScore = async () => {
      if (!dslSource) {
        setScore(null);
        return;
      }

      setLoading(true);
      try {
        const api = await getWasmApi();
        if (!api) {
          setScore(null);
          return;
        }

        // Validate DSL by attempting to parse it first
        // This prevents score calculation errors from invalid syntax
        try {
          await api.dslToModel(dslSource);
        } catch (parseError) {
          // DSL is invalid, clear score and return early
          setScore(null);
          return;
        }

        // DSL is valid, proceed with score calculation
        const result = await api.calculateArchitectureScore(dslSource);

        // Map WASM score to our breakdown format (using correct category names)
        const breakdown: ScoreBreakdown = {
          structural: result.Categories?.Structural || 0,
          documentation: result.Categories?.Documentation || 0,
          traceability: result.Categories?.Traceability || 0,
          complexity: result.Categories?.Complexity || 0,
          standardization: result.Categories?.Standardization || 0,
          overall: result.Score || 0,
        };

        setPreviousScore((prev) => {
          // Only update previous if we have a current score
          if (score) return score;
          return prev;
        });
        setScore(breakdown);
      } catch (error) {
        // Score calculation failed (but DSL was valid)
        // Clear score to avoid showing stale data
        setScore(null);
        // Only log in development to avoid noise in production
        if (process.env.NODE_ENV === "development") {
          logger.debug("Score calculation skipped", {
            component: "GovernanceScore",
            action: "calculateScore",
            error: error instanceof Error ? error.message : String(error),
          });
        }
      } finally {
        setLoading(false);
      }
    };

    calculateScore();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dslSource]);

  const getScoreColor = (value: number) => {
    if (value >= 80) return "#22c55e"; // Green
    if (value >= 60) return "#f59e0b"; // Yellow
    return "#ef4444"; // Red
  };

  const getScoreTrend = (current: number, previous: number | null) => {
    if (!previous) return null;
    if (current > previous) return <TrendingUp size={14} className="trend-up" />;
    if (current < previous) return <TrendingDown size={14} className="trend-down" />;
    return <Minus size={14} className="trend-neutral" />;
  };

  if (loading) {
    return (
      <div className="governance-score">
        <div className="governance-score-header">
          <h3 className="governance-score-title">
            <Award size={18} />
            Governance Score
          </h3>
        </div>
        <div className="governance-score-loading">
          <p>Calculating score...</p>
        </div>
      </div>
    );
  }

  if (!score) {
    return (
      <div className="governance-score">
        <div className="governance-score-header">
          <h3 className="governance-score-title">
            <Award size={18} />
            Governance Score
          </h3>
        </div>
        <div className="governance-score-empty">
          <p>No score available.</p>
          <p className="governance-score-empty-subtitle">
            Load an architecture with DSL source to calculate governance score.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="governance-score">
      <div className="governance-score-header">
        <h3 className="governance-score-title">
          <Award size={18} />
          Governance Score
        </h3>
        <div className="governance-score-overall">
          <span className="score-value" style={{ color: getScoreColor(score.overall) }}>
            {score.overall}/100
          </span>
          {getScoreTrend(score.overall, previousScore?.overall || null)}
        </div>
      </div>

      <div className="governance-score-breakdown">
        <div className="score-dimension">
          <div className="score-dimension-header">
            <span className="score-dimension-label">Structural</span>
            <span
              className="score-dimension-value"
              style={{ color: getScoreColor(score.structural) }}
            >
              {score.structural}/100
            </span>
          </div>
          <div className="score-bar">
            <div
              className="score-bar-fill"
              style={{
                width: `${score.structural}%`,
                backgroundColor: getScoreColor(score.structural),
              }}
            />
          </div>
        </div>

        <div className="score-dimension">
          <div className="score-dimension-header">
            <span className="score-dimension-label">Documentation</span>
            <span
              className="score-dimension-value"
              style={{ color: getScoreColor(score.documentation) }}
            >
              {score.documentation}/100
            </span>
          </div>
          <div className="score-bar">
            <div
              className="score-bar-fill"
              style={{
                width: `${score.documentation}%`,
                backgroundColor: getScoreColor(score.documentation),
              }}
            />
          </div>
        </div>

        <div className="score-dimension">
          <div className="score-dimension-header">
            <span className="score-dimension-label">Traceability</span>
            <span
              className="score-dimension-value"
              style={{ color: getScoreColor(score.traceability) }}
            >
              {score.traceability}/100
            </span>
          </div>
          <div className="score-bar">
            <div
              className="score-bar-fill"
              style={{
                width: `${score.traceability}%`,
                backgroundColor: getScoreColor(score.traceability),
              }}
            />
          </div>
        </div>

        <div className="score-dimension">
          <div className="score-dimension-header">
            <span className="score-dimension-label">Complexity</span>
            <span
              className="score-dimension-value"
              style={{ color: getScoreColor(score.complexity) }}
            >
              {score.complexity}/100
            </span>
          </div>
          <div className="score-bar">
            <div
              className="score-bar-fill"
              style={{
                width: `${score.complexity}%`,
                backgroundColor: getScoreColor(score.complexity),
              }}
            />
          </div>
        </div>

        <div className="score-dimension">
          <div className="score-dimension-header">
            <span className="score-dimension-label">Standardization</span>
            <span
              className="score-dimension-value"
              style={{ color: getScoreColor(score.standardization) }}
            >
              {score.standardization}/100
            </span>
          </div>
          <div className="score-bar">
            <div
              className="score-bar-fill"
              style={{
                width: `${score.standardization}%`,
                backgroundColor: getScoreColor(score.standardization),
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

// apps/designer/src/components/Architect/GovernanceScore.tsx
import { useEffect, useState } from "react";
import { Award, TrendingUp, TrendingDown, Minus } from "lucide-react";
import { getWasmApi, logger } from "@sruja/shared";
import { useArchitectureStore } from "../../../stores";
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
        {/* Show only categories that need improvement (below 80) sorted by worst first */}
        {[
          {
            name: "Structural",
            value: score.structural,
            actionable: "Fix cycles, orphans, and layer violations",
          },
          {
            name: "Documentation",
            value: score.documentation,
            actionable: "Add descriptions and technology to components",
          },
          {
            name: "Traceability",
            value: score.traceability,
            actionable: "Link requirements to components via tags",
          },
          {
            name: "Complexity",
            value: score.complexity,
            actionable: "Reduce coupling and split high-fan components",
          },
          {
            name: "Standardization",
            value: score.standardization,
            actionable: "Add metadata to components",
          },
        ]
          .filter((dim) => dim.value < 80) // Only show categories needing improvement
          .sort((a, b) => a.value - b.value) // Sort worst first
          .slice(0, 3) // Show top 3 worst categories
          .map((dimension) => (
            <div key={dimension.name} className="score-dimension">
              <div className="score-dimension-header">
                <span className="score-dimension-label">{dimension.name}</span>
                <span
                  className="score-dimension-value"
                  style={{ color: getScoreColor(dimension.value) }}
                >
                  {dimension.value}/100
                </span>
              </div>
              <div className="score-bar">
                <div
                  className="score-bar-fill"
                  style={{
                    width: `${dimension.value}%`,
                    backgroundColor: getScoreColor(dimension.value),
                  }}
                />
              </div>
              <div className="score-dimension-actionable">
                <strong>Improve:</strong> {dimension.actionable}
              </div>
            </div>
          ))}
        {[
          { name: "Structural", value: score.structural },
          { name: "Documentation", value: score.documentation },
          { name: "Traceability", value: score.traceability },
          { name: "Complexity", value: score.complexity },
          { name: "Standardization", value: score.standardization },
        ].filter((dim) => dim.value < 80).length === 0 && (
          <div className="score-all-good">
            <p>All categories are healthy (≥80/100)</p>
          </div>
        )}
      </div>
    </div>
  );
}

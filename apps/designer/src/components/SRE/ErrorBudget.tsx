// apps/designer/src/components/SRE/ErrorBudget.tsx
import { useMemo } from "react";
import { Wallet, TrendingDown, AlertTriangle, CheckCircle } from "lucide-react";
import "./ErrorBudget.css";

interface ErrorBudget {
  service: string;
  slo: number; // e.g., 99.9
  window: string; // e.g., "30d"
  totalBudget: number; // Total error budget in minutes
  consumed: number; // Consumed budget in minutes
  remaining: number; // Remaining budget in minutes
  burnRate: number; // Burn rate (consumed / time elapsed)
  status: "healthy" | "at-risk" | "exhausted";
}

export function ErrorBudget() {
  // Simulated error budget data (in real app, would come from metrics)
  const errorBudgets = useMemo<ErrorBudget[]>(() => {
    return [
      {
        service: "OrderService",
        slo: 99.9,
        window: "30d",
        totalBudget: 43.2, // minutes in 30 days for 99.9% SLO
        consumed: 12.5,
        remaining: 30.7,
        burnRate: 0.42, // 42% of time elapsed
        status: "healthy",
      },
      {
        service: "PaymentService",
        slo: 99.95,
        window: "30d",
        totalBudget: 21.6,
        consumed: 18.2,
        remaining: 3.4,
        burnRate: 0.84,
        status: "at-risk",
      },
      {
        service: "SearchService",
        slo: 99.9,
        window: "30d",
        totalBudget: 43.2,
        consumed: 45.0,
        remaining: -1.8,
        burnRate: 1.04,
        status: "exhausted",
      },
    ];
  }, []);

  const getStatusColor = (status: ErrorBudget["status"]) => {
    switch (status) {
      case "healthy":
        return "#22c55e";
      case "at-risk":
        return "#f59e0b";
      case "exhausted":
        return "#ef4444";
    }
  };

  const getStatusIcon = (status: ErrorBudget["status"]) => {
    switch (status) {
      case "healthy":
        return <CheckCircle size={16} className="icon-healthy" />;
      case "at-risk":
        return <AlertTriangle size={16} className="icon-at-risk" />;
      case "exhausted":
        return <TrendingDown size={16} className="icon-exhausted" />;
    }
  };

  const calculateBudgetPercentage = (budget: ErrorBudget) => {
    if (budget.totalBudget === 0) return 0;
    return Math.min(100, (budget.consumed / budget.totalBudget) * 100);
  };

  return (
    <div className="error-budget">
      <div className="error-budget-header">
        <h3 className="error-budget-title">
          <Wallet size={18} />
          Error Budget
        </h3>
        <div className="error-budget-note">
          <span className="budget-note-text">Based on SLO targets</span>
        </div>
      </div>

      <div className="error-budget-content">
        {errorBudgets.map((budget) => {
          const percentage = calculateBudgetPercentage(budget);
          const isOverBudget = budget.remaining < 0;

          return (
            <div key={budget.service} className={`budget-item budget-${budget.status}`}>
              <div className="budget-item-header">
                <div className="budget-item-info">
                  <div className="budget-item-service">{budget.service}</div>
                  <div className="budget-item-slo">
                    {budget.slo}% SLO ({budget.window})
                  </div>
                </div>
                <div
                  className="budget-item-status"
                  style={{ color: getStatusColor(budget.status) }}
                >
                  {getStatusIcon(budget.status)}
                </div>
              </div>
              <div className="budget-item-metrics">
                <div className="budget-metric">
                  <span className="budget-metric-label">Consumed:</span>
                  <span
                    className="budget-metric-value"
                    style={{ color: getStatusColor(budget.status) }}
                  >
                    {budget.consumed.toFixed(1)} min
                  </span>
                </div>
                <div className="budget-metric">
                  <span className="budget-metric-label">Remaining:</span>
                  <span
                    className="budget-metric-value"
                    style={{ color: isOverBudget ? "#ef4444" : "#22c55e" }}
                  >
                    {isOverBudget ? "Exceeded" : `${budget.remaining.toFixed(1)} min`}
                  </span>
                </div>
                <div className="budget-metric">
                  <span className="budget-metric-label">Burn Rate:</span>
                  <span
                    className="budget-metric-value"
                    style={{ color: budget.burnRate > 1 ? "#ef4444" : "#22c55e" }}
                  >
                    {(budget.burnRate * 100).toFixed(0)}%
                  </span>
                </div>
              </div>
              <div className="budget-item-bar">
                <div
                  className="budget-bar-fill"
                  style={{
                    width: `${Math.min(percentage, 100)}%`,
                    backgroundColor: getStatusColor(budget.status),
                  }}
                />
                {isOverBudget && (
                  <div className="budget-bar-over">
                    <div
                      className="budget-bar-over-fill"
                      style={{
                        width: `${(Math.abs(budget.remaining) / budget.totalBudget) * 100}%`,
                        backgroundColor: "#ef4444",
                      }}
                    />
                  </div>
                )}
              </div>
              {budget.burnRate > 1 && (
                <div className="budget-item-warning">
                  <AlertTriangle size={12} />
                  <span>Burn rate exceeds 100% - budget will be exhausted before window ends</span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

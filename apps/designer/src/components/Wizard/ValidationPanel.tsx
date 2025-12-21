/**
 * ValidationPanel
 * Real-time validation sidebar showing issues as users build
 */

import {
  AlertCircle,
  AlertTriangle,
  Info,
  CheckCircle,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { useState } from "react";
import { Button } from "@sruja/ui";
import { useValidation } from "../../hooks/useValidation";
import type { ValidationIssue, ValidationSeverity } from "../../utils/architectureValidator";
import "./ValidationPanel.css";

interface ValidationPanelProps {
  /** Compact mode for wizard sidebar */
  compact?: boolean;
}

export function ValidationPanel({ compact = false }: ValidationPanelProps) {
  const { score, isValid, errorCount, warningCount, issues } = useValidation();
  const [isExpanded, setIsExpanded] = useState(!compact);
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);

  const getScoreColor = () => {
    if (score >= 80) return "score-good";
    if (score >= 50) return "score-warning";
    return "score-error";
  };

  const getIcon = (severity: ValidationSeverity) => {
    switch (severity) {
      case "error":
        return <AlertCircle size={14} />;
      case "warning":
        return <AlertTriangle size={14} />;
      case "info":
        return <Info size={14} />;
    }
  };

  // Group issues by category
  const groupedIssues = issues.reduce(
    (acc, issue) => {
      if (!acc[issue.category]) acc[issue.category] = [];
      acc[issue.category].push(issue);
      return acc;
    },
    {} as Record<string, ValidationIssue[]>
  );

  const categoryLabels: Record<string, string> = {
    orphan: "Orphan Elements",
    duplicate: "Duplicate IDs",
    reference: "Invalid References",
    missing: "Missing Info",
    structure: "Structure",
    c4: "C4 Modeling",
    "best-practice": "Best Practices",
  };

  if (compact) {
    return (
      <div className={`validation-panel compact ${isValid ? "valid" : "invalid"}`}>
        <Button variant="ghost" size="sm" className="validation-header-compact" onClick={() => setIsExpanded(!isExpanded)}>
          <div className="validation-score-compact">
            {isValid ? (
              <CheckCircle size={16} className="icon-valid" />
            ) : (
              <AlertCircle size={16} className="icon-invalid" />
            )}
            <span className={`score ${getScoreColor()}`}>{score}</span>
          </div>
          <div className="validation-summary-compact">
            {errorCount > 0 && (
              <span className="badge error">
                <AlertCircle size={12} />
                {errorCount}
              </span>
            )}
            {warningCount > 0 && (
              <span className="badge warning">
                <AlertTriangle size={12} />
                {warningCount}
              </span>
            )}
            {isValid && errorCount === 0 && warningCount === 0 && (
              <span className="badge success">Valid</span>
            )}
          </div>
          {issues.length > 0 && (isExpanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />)}
        </Button>

        {isExpanded && issues.length > 0 && (
          <div className="validation-issues-compact">
            {issues.slice(0, 5).map((issue) => (
              <div key={issue.id} className={`issue-item ${issue.severity}`}>
                {getIcon(issue.severity)}
                <span className="issue-message">{issue.message}</span>
              </div>
            ))}
            {issues.length > 5 && (
              <div className="issues-more">+{issues.length - 5} more issues</div>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="validation-panel">
      <div className="validation-header">
        <div className="validation-title">
          {isValid ? (
            <CheckCircle size={20} className="icon-valid" />
          ) : (
            <AlertCircle size={20} className="icon-invalid" />
          )}
          <span>Quality Score</span>
        </div>
        <div className={`validation-score ${getScoreColor()}`}>{score}</div>
      </div>

      <div className="validation-summary">
        <div className={`summary-stat ${errorCount > 0 ? "has-issues" : ""}`}>
          <AlertCircle size={16} />
          <span>{errorCount} errors</span>
        </div>
        <div className={`summary-stat ${warningCount > 0 ? "has-issues" : ""}`}>
          <AlertTriangle size={16} />
          <span>{warningCount} warnings</span>
        </div>
      </div>

      {issues.length === 0 ? (
        <div className="validation-empty">
          <CheckCircle size={32} className="icon-valid" />
          <p>No issues found!</p>
        </div>
      ) : (
        <div className="validation-categories">
          {Object.entries(groupedIssues).map(([category, categoryIssues]) => (
            <div key={category} className="validation-category">
              <Button
                variant="ghost"
                size="sm"
                className="category-header"
                onClick={() => setExpandedCategory(expandedCategory === category ? null : category)}
              >
                <span className="category-label">{categoryLabels[category] || category}</span>
                <span className="category-count">{categoryIssues.length}</span>
                {expandedCategory === category ? (
                  <ChevronUp size={14} />
                ) : (
                  <ChevronDown size={14} />
                )}
              </Button>

              {expandedCategory === category && (
                <div className="category-issues">
                  {categoryIssues.map((issue) => (
                    <div key={issue.id} className={`issue-card ${issue.severity}`}>
                      <div className="issue-header">
                        {getIcon(issue.severity)}
                        {issue.elementId && (
                          <code className="issue-element">{issue.elementId}</code>
                        )}
                      </div>
                      <p className="issue-message">{issue.message}</p>
                      {issue.suggestion && (
                        <p className="issue-suggestion">💡 {issue.suggestion}</p>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

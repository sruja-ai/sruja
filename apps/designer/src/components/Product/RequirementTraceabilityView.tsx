// apps/designer/src/components/Product/RequirementTraceabilityView.tsx
import { useState, useMemo, useCallback } from "react";
import { Target, CheckCircle, AlertCircle, XCircle } from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { useTagNavigation } from "../../hooks/useTagNavigation";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import type { RequirementDump } from "@sruja/shared";
import "./RequirementTraceabilityView.css";

interface RequirementTraceabilityViewProps {
  onElementHighlight?: (elementIds: string[]) => void;
  onElementClear?: () => void;
}

export function RequirementTraceabilityView({
  onElementHighlight,
  onElementClear,
}: RequirementTraceabilityViewProps) {
  const model = useArchitectureStore((s) => s.likec4Model);
  const { navigateToTaggedElement } = useTagNavigation();

  const [selectedRequirement, setSelectedRequirement] = useState<string | null>(null);
  const [isAnimating, setIsAnimating] = useState(false);
  const [_highlightedElements, setHighlightedElements] = useState<Set<string>>(new Set());

  // Deduplicate requirements by ID
  const requirements = useMemo(() => {
    const reqs = (model?.sruja as any)?.requirements || [];
    return deduplicateRequirements(reqs);
  }, [model]);

  // Calculate requirement coverage
  const requirementCoverage = useMemo(() => {
    const coverage: Record<string, {
      elementIds: string[];
      coverage: number;
      status: "fulfilled" | "partial" | "missing";
    }> = {};

    requirements.forEach((req) => {
      const elementIds: string[] = (req as any).tags ?? [];
      const hasLinks = elementIds.length > 0;
      const status: "fulfilled" | "partial" | "missing" =
        hasLinks ? (elementIds.length >= 2 ? "fulfilled" : "partial") : "missing";

      coverage[req.id] = {
        elementIds,
        coverage: hasLinks ? Math.min(100, (elementIds.length / 3) * 100) : 0,
        status,
      };
    });

    return coverage;
  }, [requirements]);

  // Animate requirement highlighting
  const animateRequirement = useCallback(
    (requirement: RequirementDump) => {
      if (isAnimating) return;

      setIsAnimating(true);
      setSelectedRequirement(requirement.id);

      const elementIds: string[] = (requirement as any).tags ?? [];
      const highlightedSet = new Set(elementIds);

      setHighlightedElements(highlightedSet);

      // Notify parent to highlight elements
      if (onElementHighlight) {
        onElementHighlight(elementIds);
      }

      // Animate elements sequentially
      elementIds.forEach((_elementId, index) => {
        setTimeout(() => {
          // Individual element highlight animation
          // This will be handled by the canvas component
        }, index * 300); // Stagger animations
      });

      // Clear animation after duration
      setTimeout(() => {
        setIsAnimating(false);
      }, elementIds.length * 300 + 1000);
    },
    [isAnimating, onElementHighlight]
  );

  const clearHighlight = useCallback(() => {
    setSelectedRequirement(null);
    setHighlightedElements(new Set());
    if (onElementClear) {
      onElementClear();
    }
  }, [onElementClear]);

  const handleRequirementClick = (requirement: RequirementDump) => {
    if (selectedRequirement === requirement.id) {
      clearHighlight();
    } else {
      animateRequirement(requirement);
    }
  };

  const handleElementClick = (elementId: string) => {
    navigateToTaggedElement(elementId);
  };

  // Calculate overall coverage
  const overallCoverage = useMemo(() => {
    if (requirements.length === 0) return 0;
    const total = requirements.reduce((sum, req) => {
      const cov = requirementCoverage[req.id];
      return sum + (cov?.coverage ?? 0);
    }, 0);
    return Math.round(total / requirements.length);
  }, [requirements, requirementCoverage]);

  return (
    <div className="requirement-traceability-view">
      <div className="traceability-header">
        <h2>
          <Target size={20} />
          Requirement Traceability
        </h2>
        <div className="coverage-summary">
          <span className="coverage-label">Overall Coverage:</span>
          <span className={`coverage-value ${overallCoverage >= 80 ? "good" : overallCoverage >= 50 ? "medium" : "poor"}`}>
            {overallCoverage}%
          </span>
        </div>
      </div>

      <div className="traceability-content">
        <div className="requirements-panel">
          <div className="requirements-list">
            {requirements.map((req) => {
              const coverage = requirementCoverage[req.id];
              const isSelected = selectedRequirement === req.id;
              const hasLinks = (coverage?.elementIds.length ?? 0) > 0;

              return (
                <div
                  key={req.id}
                  className={`requirement-card ${isSelected ? "selected" : ""} ${coverage?.status || "missing"}`}
                  onClick={() => handleRequirementClick(req)}
                >
                  <div className="requirement-header">
                    <div className="requirement-id">{req.id}</div>
                    <div className={`status-badge ${coverage?.status || "missing"}`}>
                      {coverage?.status === "fulfilled" ? (
                        <CheckCircle size={14} />
                      ) : coverage?.status === "partial" ? (
                        <AlertCircle size={14} />
                      ) : (
                        <XCircle size={14} />
                      )}
                      <span>{coverage?.status || "missing"}</span>
                    </div>
                  </div>
                  <div className="requirement-title">{req.title}</div>
                  {coverage && (
                    <div className="requirement-coverage">
                      <div className="coverage-bar">
                        <div
                          className="coverage-fill"
                          style={{ width: `${coverage.coverage}%` }}
                        />
                      </div>
                      <span className="coverage-text">
                        {coverage.elementIds.length} element{coverage.elementIds.length !== 1 ? "s" : ""}
                      </span>
                    </div>
                  )}
                  {hasLinks && (
                    <div className="linked-elements">
                      {coverage.elementIds.map((elementId) => (
                        <Button
                          key={elementId}
                          variant="ghost"
                          size="sm"
                          className="element-tag"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleElementClick(elementId);
                          }}
                          title={`Navigate to ${elementId}`}
                        >
                          {elementId}
                        </Button>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        <div className="traceability-diagram">
          <div className="diagram-placeholder">
            <p>Architecture diagram will show here</p>
            <p className="hint">
              {selectedRequirement
                ? `Highlighting elements for ${selectedRequirement}`
                : "Click a requirement to see how it maps to architecture"}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

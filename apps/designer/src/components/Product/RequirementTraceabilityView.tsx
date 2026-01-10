// apps/designer/src/components/Product/RequirementTraceabilityView.tsx
import { useState, useMemo, useCallback } from "react";
import { Target, CheckCircle, AlertCircle, XCircle } from "lucide-react";
import { Button, Badge } from "@sruja/ui";
import { useArchitectureStore } from "../../stores";
import { useTagNavigation } from "../../hooks/useTagNavigation";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import type { RequirementDump, SrujaModelDump } from "@sruja/shared";
import "./RequirementTraceabilityView.css";

interface RequirementTraceabilityViewProps {
  onElementHighlight?: (elementIds: string[]) => void;
  onElementClear?: () => void;
}

export function RequirementTraceabilityView({
  onElementHighlight,
  onElementClear,
}: RequirementTraceabilityViewProps) {
  const model = useArchitectureStore((s) => s.model);
  const { navigateToTaggedElement } = useTagNavigation();

  const [selectedRequirement, setSelectedRequirement] = useState<string | null>(null);
  const [isAnimating, setIsAnimating] = useState(false);
  const [_highlightedElements, setHighlightedElements] = useState<Set<string>>(new Set());

  // Deduplicate requirements by ID
  const requirements = useMemo(() => {
    const reqs = (model?.sruja as SrujaModelDump["sruja"])?.requirements || [];
    return deduplicateRequirements(reqs);
  }, [model]);

  // Calculate requirement coverage
  const requirementCoverage = useMemo(() => {
    const coverage: Record<
      string,
      {
        elementIds: string[];
        coverage: number;
        status: "fulfilled" | "partial" | "missing";
      }
    > = {};

    requirements.forEach((req) => {
      const elementIds: string[] = (req as { tags?: string[] }).tags ?? [];
      const hasLinks = elementIds.length > 0;
      const status: "fulfilled" | "partial" | "missing" = hasLinks
        ? elementIds.length >= 2
          ? "fulfilled"
          : "partial"
        : "missing";

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

      const elementIds: string[] = (requirement as { tags?: string[] }).tags ?? [];
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
      setTimeout(
        () => {
          setIsAnimating(false);
        },
        elementIds.length * 300 + 1000
      );
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
          <span
            className={`coverage-value ${overallCoverage >= 80 ? "good" : overallCoverage >= 50 ? "medium" : "poor"}`}
          >
            {overallCoverage}%
          </span>
        </div>
      </div>

      <div className="traceability-content">
        <div className="requirements-panel">
          <div className="requirements-list">
            {/* Show missing/partial requirements first (High priority), then fulfilled */}
            {requirements
              .sort((a, b) => {
                const aCoverage = requirementCoverage[a.id];
                const bCoverage = requirementCoverage[b.id];
                const aStatus = aCoverage?.status || "missing";
                const bStatus = bCoverage?.status || "missing";
                if (aStatus === "missing" && bStatus !== "missing") return -1;
                if (aStatus === "partial" && bStatus === "fulfilled") return -1;
                return 0;
              })
              .map((req) => {
                const coverage = requirementCoverage[req.id];
                const isSelected = selectedRequirement === req.id;
                const hasLinks = (coverage?.elementIds.length ?? 0) > 0;
                const status = coverage?.status || "missing";

                return (
                  <div
                    key={req.id}
                    className={`requirement-card ${isSelected ? "selected" : ""} ${status}`}
                    onClick={() => handleRequirementClick(req)}
                  >
                    <div className="requirement-header">
                      <div className="requirement-id-row">
                        <div className="requirement-id">{req.id}</div>
                        {status === "missing" && (
                          <Badge color="error" className="priority-badge-small">
                            High
                          </Badge>
                        )}
                        {status === "partial" && (
                          <Badge color="warning" className="priority-badge-small">
                            Medium
                          </Badge>
                        )}
                      </div>
                      <div className={`status-badge ${status}`}>
                        {status === "fulfilled" ? (
                          <CheckCircle size={14} />
                        ) : status === "partial" ? (
                          <AlertCircle size={14} />
                        ) : (
                          <XCircle size={14} />
                        )}
                        <span>{status}</span>
                      </div>
                    </div>
                    <div className="requirement-title">{req.title}</div>
                    {status !== "fulfilled" && (
                      <div className="requirement-actionable">
                        <strong>Fix:</strong> Link requirement to components via tags:{" "}
                        <code>{req.id} #componentId</code>
                      </div>
                    )}
                    {coverage && (
                      <div className="requirement-coverage">
                        <div className="coverage-bar">
                          <div
                            className="coverage-fill"
                            style={{ width: `${coverage.coverage}%` }}
                          />
                        </div>
                        <span className="coverage-text">
                          {coverage.elementIds.length} element
                          {coverage.elementIds.length !== 1 ? "s" : ""}
                        </span>
                      </div>
                    )}
                    {hasLinks && (
                      <div className="linked-elements">
                        {coverage.elementIds.slice(0, 3).map((elementId) => (
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
                        {coverage.elementIds.length > 3 && (
                          <span className="more-elements">
                            +{coverage.elementIds.length - 3} more
                          </span>
                        )}
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

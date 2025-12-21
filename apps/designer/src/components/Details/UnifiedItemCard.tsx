// apps/designer/src/components/Details/UnifiedItemCard.tsx
import { useMemo } from "react";
import { Target, FileText, Play, Workflow, CheckCircle, AlertCircle, XCircle, Edit, Trash2 } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import type { ADRDump, ScenarioDump, FlowDump } from "@sruja/shared";
import type { UnifiedItem } from "./UnifiedDetailsList";
import "./UnifiedItemCard.css";

interface UnifiedItemCardProps {
  item: UnifiedItem;
  onClick?: () => void;
  onTagClick?: (tag: string) => void;
  onEdit?: () => void;
  onDelete?: () => void;
}

export function UnifiedItemCard({ item, onClick, onTagClick, onEdit, onDelete }: UnifiedItemCardProps) {
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Calculate coverage and status
  const { coverage, status, elementCount } = useMemo(() => {
    // Note: Assuming 'tags' might be missing on types currently or defined as undefined.
    // Casting to any to safely access if type doesn't have it yet, or accessing directly if it does.
    const tags = ((item.data as any).tags || []) as string[];
    const hasLinks = tags.length > 0;

    if (item.type === "requirement") {
      const status: "fulfilled" | "partial" | "missing" =
        hasLinks ? (tags.length >= 2 ? "fulfilled" : "partial") : "missing";
      const coverage = hasLinks ? Math.min(100, (tags.length / 3) * 100) : 0;
      return { coverage, status, elementCount: tags.length };
    } else if (item.type === "adr") {
      const adr = item.data as ADRDump;
      const adrStatus = adr.status;
      const status: "fulfilled" | "partial" | "missing" =
        adrStatus === "accepted" ? "fulfilled" : adrStatus === "deprecated" ? "missing" : "partial";
      const coverage = hasLinks ? Math.min(100, (tags.length / 2) * 100) : 0;
      return { coverage, status, elementCount: tags.length };
    } else if (item.type === "scenario") {
      const scenario = item.data as ScenarioDump;
      const stepCount = scenario.steps?.length ?? 0;
      return { coverage: stepCount > 0 ? 100 : 0, status: "fulfilled" as const, elementCount: stepCount };
    } else {
      const flow = item.data as FlowDump;
      const stepCount = flow.steps?.length ?? 0;
      return { coverage: stepCount > 0 ? 100 : 0, status: "fulfilled" as const, elementCount: stepCount };
    }
  }, [item]);

  const getIcon = () => {
    switch (item.type) {
      case "requirement":
        return <Target size={16} />;
      case "adr":
        return <FileText size={16} />;
      case "scenario":
        return <Play size={16} />;
      case "flow":
        return <Workflow size={16} />;
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case "fulfilled":
        return <CheckCircle size={12} />;
      case "partial":
        return <AlertCircle size={12} />;
      case "missing":
        return <XCircle size={12} />;
    }
  };

  const title = (item.data.title) || item.data.id || "Untitled";
  const description = (item.data as any).description;
  const tags = ((item.data as any).tags || []) as string[];

  return (
    <div
      className={`unified-item-card ${item.type} ${status}`}
      onClick={onClick}
    >
      <div className="item-header">
        <div className="item-icon-type">
          {getIcon()}
          <span className="item-id">{item.data.id}</span>
        </div>
        <div className="item-badges">
          {status !== "fulfilled" && (
            <div className={`status-badge ${status}`}>
              {getStatusIcon()}
              <span>{status}</span>
            </div>
          )}
          {isEditMode() && (
            <div className="item-actions">
              {onEdit && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-action-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit();
                  }}
                  title="Edit"
                >
                  <Edit size={12} />
                </Button>
              )}
              {onDelete && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="item-action-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete();
                  }}
                  title="Delete"
                >
                  <Trash2 size={12} />
                </Button>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="item-title">{title}</div>
      {description && <div className="item-description">{description}</div>}

      {/* Coverage Bar (for requirements and ADRs) */}
      {(item.type === "requirement" || item.type === "adr") && (
        <div className="item-coverage">
          <div className="coverage-bar">
            <div className="coverage-fill" style={{ width: `${coverage}%` }} />
          </div>
          <span className="coverage-text">
            {elementCount} element{elementCount !== 1 ? "s" : ""}
          </span>
        </div>
      )}

      {/* Step Count (for scenarios and flows) */}
      {(item.type === "scenario" || item.type === "flow") && (
        <div className="item-steps">
          <span className="steps-count">
            {(item.data as ScenarioDump | FlowDump).steps?.length ?? 0} step{((item.data as ScenarioDump | FlowDump).steps?.length ?? 0) !== 1 ? "s" : ""}
          </span>
        </div>
      )}

      {/* Tags */}
      {tags.length > 0 && (
        <div className="item-tags">
          {tags.map((tag: string) => (
            <Button
              key={tag}
              variant="ghost"
              size="sm"
              className="item-tag clickable"
              onClick={(e) => {
                e.stopPropagation();
                onTagClick?.(tag);
              }}
              title={`Navigate to ${tag} in diagram`}
            >
              {tag}
            </Button>
          ))}
        </div>
      )}
    </div>
  );
}

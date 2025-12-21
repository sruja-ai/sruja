import { memo, useCallback } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Building2, ChevronDown, ChevronUp, AlertTriangle, Target, FileText, Play, Workflow } from "lucide-react";
import { Button } from "@sruja/ui";
import type { C4NodeData } from "../../types";
import { isSystemNodeData } from "../../types/nodeData";
import { getNodeColors } from "../../utils/colorScheme";
import { useViewStore } from "../../stores";
import { useUIStore } from "../../stores/uiStore";
import "./nodes.css";

type SystemNodeProps = NodeProps<Node<C4NodeData>>;

function SystemNodeComponent({ data, selected, id }: SystemNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors("system", nodeData.isExternal);
  const isExternal = nodeData.isExternal === true;
  const isExpanded = nodeData.expanded === true;
  const hasChildren = (nodeData.childCount ?? 0) > 0;

  const toggleExpand = useViewStore((s) => s.toggleExpand);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  // Governance counts from enriched node data
  const requirementCount = isSystemNodeData(nodeData) ? (nodeData.requirementCount ?? 0) : 0;
  const adrCount = isSystemNodeData(nodeData) ? (nodeData.adrCount ?? 0) : 0;
  const scenarioCount = isSystemNodeData(nodeData) ? (nodeData.scenarioCount ?? 0) : 0;
  const flowCount = isSystemNodeData(nodeData) ? (nodeData.flowCount ?? 0) : 0;
  const totalLinkedItems = requirementCount + adrCount + scenarioCount + flowCount;

  const handleExpandClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent node selection
    toggleExpand(nodeData.id);
  };

  const handleBadgeClick = (e: React.MouseEvent, tab: "requirements" | "adrs" | "scenarios" | "flows") => {
    e.stopPropagation();
    setActiveTab("details");
    // Small delay to ensure Details tab is mounted, then trigger sub-tab switch
    setTimeout(() => {
      const event = new CustomEvent("switch-details-subtab", { detail: tab });
      window.dispatchEvent(event);
    }, 100);
  };

  // Keyboard navigation support
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        if (hasChildren) {
          toggleExpand(nodeData.id);
        }
      }
    },
    [hasChildren, toggleExpand, nodeData.id]
  );

  // Build accessible description
  const ariaDescription = [
    isExternal ? "External system" : "Internal system",
    nodeData.technology ? `Technology: ${nodeData.technology}` : "",
    nodeData.description || "",
    hasChildren
      ? `Contains ${nodeData.childCount} container${(nodeData.childCount ?? 0) > 1 ? "s" : ""}`
      : "",
  ]
    .filter(Boolean)
    .join(". ");

  return (
    <div
      className={`c4-node system-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""} ${isExpanded ? "expanded" : ""}`}
      style={{
        backgroundColor: colors.bg,
        borderColor: colors.border,
        color: colors.text,
      }}
      // Accessibility attributes
      role="group"
      aria-label={`System: ${nodeData.label}`}
      aria-expanded={hasChildren ? isExpanded : undefined}
      aria-describedby={`${id}-desc`}
      tabIndex={0}
      onKeyDown={handleKeyDown}
    >
      {/* Hidden description for screen readers */}
      <span id={`${id}-desc`} className="sr-only">
        {ariaDescription}
      </span>

      <Handle
        type="target"
        position={Position.Top}
        className="c4-handle"
        aria-label="Incoming connection"
      />

      <div className="node-header">
        <div className="node-icon" aria-hidden="true">
          <Building2 size={24} />
        </div>

        <div className="node-content">
          <div className="node-label">{nodeData.label}</div>
          {nodeData.technology && <div className="node-technology">[{nodeData.technology}]</div>}
          {nodeData.description && <div className="node-description">{nodeData.description}</div>}
        </div>

        {hasChildren && (
          <Button
            variant="ghost"
            size="sm"
            className="expand-btn"
            onClick={handleExpandClick}
            aria-label={
              isExpanded
                ? `Collapse ${nodeData.label}`
                : `Expand ${nodeData.label} to show containers`
            }
            aria-expanded={isExpanded}
          >
            {isExpanded ? (
              <ChevronUp size={16} aria-hidden="true" />
            ) : (
              <ChevronDown size={16} aria-hidden="true" />
            )}
            <span className="child-count" aria-hidden="true">
              {nodeData.childCount}
            </span>
          </Button>
        )}
      </div>

      {nodeData.pendingActionCount && nodeData.pendingActionCount > 0 && (
        <div
          className="node-warning-badge"
          aria-hidden="true"
          title={`${nodeData.pendingActionCount} pending action${nodeData.pendingActionCount > 1 ? "s" : ""} - Action required`}
        >
          <AlertTriangle size={12} />
          <span>{nodeData.pendingActionCount}</span>
        </div>
      )}
      {!isExpanded && hasChildren && (
        <div className="node-badge" aria-hidden="true">
          {nodeData.childCount} container{(nodeData.childCount ?? 0) > 1 ? "s" : ""}
        </div>
      )}

      {/* Governance badges - show linked items */}
      {totalLinkedItems > 0 && !isExpanded && (
        <div className="governance-badges">
          {requirementCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge requirement-badge"
              onClick={(e) => handleBadgeClick(e, "requirements")}
              title={`${requirementCount} requirement${requirementCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${requirementCount} requirement${requirementCount > 1 ? "s" : ""}`}
            >
              <Target size={10} />
              <span>{requirementCount}</span>
            </Button>
          )}
          {adrCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge adr-badge"
              onClick={(e) => handleBadgeClick(e, "adrs")}
              title={`${adrCount} ADR${adrCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${adrCount} ADR${adrCount > 1 ? "s" : ""}`}
            >
              <FileText size={10} />
              <span>{adrCount}</span>
            </Button>
          )}
          {scenarioCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge scenario-badge"
              onClick={(e) => handleBadgeClick(e, "scenarios")}
              title={`${scenarioCount} scenario${scenarioCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${scenarioCount} scenario${scenarioCount > 1 ? "s" : ""}`}
            >
              <Play size={10} />
              <span>{scenarioCount}</span>
            </Button>
          )}
          {flowCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="governance-badge flow-badge"
              onClick={(e) => handleBadgeClick(e, "flows")}
              title={`${flowCount} flow${flowCount > 1 ? "s" : ""} - Click to view`}
              aria-label={`${flowCount} flow${flowCount > 1 ? "s" : ""}`}
            >
              <Workflow size={10} />
              <span>{flowCount}</span>
            </Button>
          )}
        </div>
      )}

      <Handle
        type="source"
        position={Position.Bottom}
        className="c4-handle"
        aria-label="Outgoing connection"
      />
    </div>
  );
}

// Memoize to prevent unnecessary re-renders when props haven't changed
export const SystemNode = memo(SystemNodeComponent);

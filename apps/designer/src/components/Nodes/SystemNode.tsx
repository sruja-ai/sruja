import { memo, useCallback } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Building2, ChevronDown, ChevronUp, AlertTriangle } from "lucide-react";
import type { C4NodeData } from "../../types";
import { getNodeColors } from "../../utils/colorScheme";
import { useViewStore } from "../../stores";
import "./nodes.css";

type SystemNodeProps = NodeProps<Node<C4NodeData>>;

function SystemNodeComponent({ data, selected, id }: SystemNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors("system", nodeData.isExternal);
  const isExternal = nodeData.isExternal === true;
  const isExpanded = nodeData.expanded === true;
  const hasChildren = (nodeData.childCount ?? 0) > 0;

  const toggleExpand = useViewStore((s) => s.toggleExpand);

  const handleExpandClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent node selection
    toggleExpand(nodeData.id);
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
          <button
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
          </button>
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

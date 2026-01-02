import { memo } from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import type { C4Node } from "../../components/SrujaCanvas/types";
import { getNodeColors } from "../../utils/colorScheme";
import "./nodes.css";

type GroupNodeProps = NodeProps<Node<C4Node & { _isParent?: boolean; _childCount?: number }>>;

/**
 * GroupNode renders parent container nodes (systems, containers) that contain child nodes.
 *
 * This is used for L2 (system containing containers) and L3 (container containing components) views.
 * React Flow automatically handles positioning of children when parentId is set on child nodes.
 */
function GroupNodeComponent({ data, selected, width, height }: GroupNodeProps) {
  const nodeData = data;
  const colors = getNodeColors(nodeData.kind, false);
  const childCount = nodeData._childCount ?? 0;

  // Use React Flow's computed dimensions, with fallback
  const nodeWidth = width || 400;
  const nodeHeight = height || 300;

  return (
    <div
      className={`c4-node group-node ${nodeData.kind}-node ${selected ? "selected" : ""}`}
      style={{
        width: nodeWidth,
        height: nodeHeight,
        backgroundColor: "transparent",
        borderColor: colors.border,
        borderWidth: 2,
        borderStyle: "dashed",
        borderRadius: 8,
        position: "relative",
        boxShadow: selected ? `0 0 0 2px ${colors.border}` : undefined,
      }}
    >
      {/* Header showing parent container label */}
      <div
        style={{
          position: "absolute",
          top: -12,
          left: 8,
          backgroundColor: colors.bg,
          padding: "2px 8px",
          borderRadius: 4,
          border: `1px solid ${colors.border}`,
          fontSize: "12px",
          fontWeight: 600,
          color: colors.text,
          whiteSpace: "nowrap",
          zIndex: 10,
        }}
      >
        {nodeData.title}
        {childCount > 0 && (
          <span style={{ marginLeft: 4, opacity: 0.7, fontWeight: 400 }}>({childCount})</span>
        )}
      </div>

      {/* Visual indicator that this is a container */}
      <div
        style={{
          position: "absolute",
          top: 4,
          right: 4,
          fontSize: "10px",
          color: colors.text,
          opacity: 0.5,
        }}
        aria-hidden="true"
      >
        Container
      </div>
    </div>
  );
}

// Memoize to prevent unnecessary re-renders
export const GroupNode = memo(GroupNodeComponent);

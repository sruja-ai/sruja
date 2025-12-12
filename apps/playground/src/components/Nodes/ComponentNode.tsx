import { memo } from "react";
import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Component } from "lucide-react";
import type { C4NodeData } from "../../types";
import { getNodeColors } from "../../utils/colorScheme";
import "./nodes.css";

type ComponentNodeProps = NodeProps<Node<C4NodeData>>;

function ComponentNodeComponent({ data, selected }: ComponentNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors("component", nodeData.isExternal);
  const isExternal = nodeData.isExternal === true;

  return (
    <div
      className={`c4-node component-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""}`}
      style={{
        backgroundColor: colors.bg === "transparent" ? undefined : colors.bg,
        borderColor: colors.border,
        color: colors.text,
      }}
    >
      <Handle type="target" position={Position.Top} className="c4-handle" />

      <div className="node-icon">
        <Component size={18} />
      </div>

      <div className="node-content">
        <div className="node-label">{nodeData.label}</div>
        {nodeData.technology && <div className="node-technology">[{nodeData.technology}]</div>}
      </div>

      <Handle type="source" position={Position.Bottom} className="c4-handle" />
    </div>
  );
}

export const ComponentNode = memo(ComponentNodeComponent);

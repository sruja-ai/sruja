import { memo } from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { User } from "lucide-react";
import type { C4NodeData } from "../../types";
import { getNodeColors } from "../../utils/colorScheme";
import { ConnectionPorts } from "./BaseNode/ConnectionPorts";
import "./nodes.css";

type PersonNodeProps = NodeProps<Node<C4NodeData>>;

function PersonNodeComponent({ data, selected, id }: PersonNodeProps) {
  const nodeData = data as C4NodeData;
  const colors = getNodeColors("person", nodeData.isExternal);
  const isExternal = nodeData.isExternal === true;

  // Build accessible description
  const ariaDescription = [
    isExternal ? "External user" : "Internal user",
    nodeData.description || "",
  ]
    .filter(Boolean)
    .join(". ");

  return (
    <div
      className={`c4-node person-node ${selected ? "selected" : ""} ${isExternal ? "external" : ""}`}
      style={{
        backgroundColor: colors.bg === "transparent" ? undefined : colors.bg,
        borderColor: colors.border,
        color: colors.text,
      }}
      // Accessibility attributes
      role="img"
      aria-label={`Person: ${nodeData.label}`}
      aria-describedby={`${id}-desc`}
      tabIndex={0}
    >
      {/* Hidden description for screen readers */}
      <span id={`${id}-desc`} className="sr-only">
        {ariaDescription}
      </span>

      <div className="person-icon-circle" aria-hidden="true">
        <User size={32} />
      </div>

      <div className="node-content">
        <div className="node-label">{nodeData.label}</div>
        {nodeData.description && <div className="node-description">{nodeData.description}</div>}
      </div>

      {/* Person nodes typically sit at the top of the diagram and connect downwards */}
      <ConnectionPorts sides={["bottom", "left", "right"]} />
      {/* Added left/right just in case, but removed top to force flow */}
    </div>
  );
}

export const PersonNode = memo(PersonNodeComponent);

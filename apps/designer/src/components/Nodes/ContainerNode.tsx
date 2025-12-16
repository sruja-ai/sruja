import { memo } from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Box } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseCompoundNode } from "./BaseNode/BaseCompoundNode";
import "./nodes.css";

type ContainerNodeProps = NodeProps<Node<C4NodeData>>;

function ContainerNodeComponent(props: ContainerNodeProps) {
  return (
    <BaseCompoundNode
      {...props}
      icon={<Box size={20} />}
      type="container"
      // Containers inside systems still act as hubs
      ports={["top", "right", "bottom", "left"]}
    />
  );
}

// Memoize for performance
export const ContainerNode = memo(ContainerNodeComponent);

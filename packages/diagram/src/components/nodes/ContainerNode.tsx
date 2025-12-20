import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Box } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type ContainerNodeProps = NodeProps<Node<C4NodeData>>;

export const ContainerNode = React.memo(function ContainerNode({
  data,
  selected,
}: ContainerNodeProps) {
  return (
    <BaseNode
      data={data as C4NodeData}
      selected={selected}
      nodeType="container"
      icon={<Box size={20} />}
    />
  );
});


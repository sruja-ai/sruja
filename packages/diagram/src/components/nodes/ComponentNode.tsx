import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Component } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type ComponentNodeProps = NodeProps<Node<C4NodeData>>;

export const ComponentNode = React.memo(function ComponentNode({
  data,
  selected,
}: ComponentNodeProps) {
  return (
    <BaseNode
      data={data as C4NodeData}
      selected={selected}
      nodeType="component"
      icon={<Component size={18} />}
      showDescription={false}
    />
  );
});


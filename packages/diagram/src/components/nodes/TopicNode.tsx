import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { MessageSquare } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type TopicNodeProps = NodeProps<Node<C4NodeData>>;

export const TopicNode = React.memo(function TopicNode({
    data,
    selected,
}: TopicNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="topic"
            icon={<MessageSquare size={20} />}
            showDescription={false}
        />
    );
});





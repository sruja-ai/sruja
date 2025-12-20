import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { MessageSquare } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type QueueNodeProps = NodeProps<Node<C4NodeData>>;

export const QueueNode = React.memo(function QueueNode({
    data,
    selected,
}: QueueNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="queue"
            icon={<MessageSquare size={20} />}
            showDescription={false}
        />
    );
});


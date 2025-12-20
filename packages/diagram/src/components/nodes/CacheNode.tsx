import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Zap } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type CacheNodeProps = NodeProps<Node<C4NodeData>>;

export const CacheNode = React.memo(function CacheNode({
    data,
    selected,
}: CacheNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="cache"
            icon={<Zap size={20} />}
            showDescription={false}
        />
    );
});





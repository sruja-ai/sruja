import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Cloud } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type DeploymentNodeProps = NodeProps<Node<C4NodeData>>;

export const DeploymentNode = React.memo(function DeploymentNode({
    data,
    selected,
}: DeploymentNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="deployment"
            icon={<Cloud size={20} />}
            style={{ borderStyle: "dashed" }}
        />
    );
});





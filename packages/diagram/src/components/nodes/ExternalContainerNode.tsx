import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Box } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type ExternalContainerNodeProps = NodeProps<Node<C4NodeData>>;

export const ExternalContainerNode = React.memo(function ExternalContainerNode({
    data,
    selected,
}: ExternalContainerNodeProps) {
    // Force external styling by adding isExternal to data
    const externalData = { ...data, isExternal: true } as C4NodeData;

    return (
        <BaseNode
            data={externalData}
            selected={selected}
            nodeType="external-container"
            icon={<Box size={20} />}
            className="external"
            style={{ borderStyle: "dashed" }}
        />
    );
});





import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Component } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type ExternalComponentNodeProps = NodeProps<Node<C4NodeData>>;

export const ExternalComponentNode = React.memo(function ExternalComponentNode({
    data,
    selected,
}: ExternalComponentNodeProps) {
    // Force external styling by adding isExternal to data
    const externalData = { ...data, isExternal: true } as C4NodeData;

    return (
        <BaseNode
            data={externalData}
            selected={selected}
            nodeType="external-component"
            icon={<Component size={18} />}
            className="external"
            style={{ borderStyle: "dashed" }}
            showDescription={false}
        />
    );
});





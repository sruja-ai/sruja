import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Folder } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type FileSystemNodeProps = NodeProps<Node<C4NodeData>>;

export const FileSystemNode = React.memo(function FileSystemNode({
    data,
    selected,
}: FileSystemNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="filesystem"
            icon={<Folder size={20} />}
            showDescription={false}
        />
    );
});





import React from "react";
import { type NodeProps, type Node } from "@xyflow/react";
import { Database } from "lucide-react";
import type { C4NodeData } from "../../types";
import { BaseNode } from "./BaseNode";

type DataStoreNodeProps = NodeProps<Node<C4NodeData>>;

export const DataStoreNode = React.memo(function DataStoreNode({
    data,
    selected,
}: DataStoreNodeProps) {
    return (
        <BaseNode
            data={data as C4NodeData}
            selected={selected}
            nodeType="datastore"
            icon={<Database size={20} />}
            showDescription={false}
        />
    );
});


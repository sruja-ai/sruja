/**
 * Node Factory Utilities
 * 
 * FAANG best practice: Use factory pattern with configuration objects
 * to eliminate repetitive node creation logic.
 */
import type { Node } from "@xyflow/react";
import type { C4NodeData, C4NodeType } from "../types";

/** Configuration for creating a node */
export interface NodeFactoryConfig {
    /** The C4 node type */
    type: C4NodeType;
    /** Optional parent node ID for nested nodes */
    parentId?: string;
    /** Whether the node represents an external entity */
    isExternal?: boolean;
    /** Whether the node is currently expanded */
    expanded?: boolean;
    /** Number of child nodes */
    childCount?: number;
}

/** Base input data for creating a node */
export interface NodeSourceData {
    id: string;
    label?: string;
    description?: string;
    technology?: string;
}

/**
 * Generic factory function for creating React Flow nodes.
 * Eliminates duplication across create*Node functions.
 * 
 * @example
 * ```ts
 * const node = createNode(
 *   { id: "web", label: "Web App", technology: "React" },
 *   { x: 100, y: 200 },
 *   { type: "container", parentId: "system1" }
 * );
 * ```
 */
export function createNode(
    source: NodeSourceData,
    position: { x: number; y: number },
    config: NodeFactoryConfig
): Node<C4NodeData> {
    return {
        id: source.id,
        type: config.type,
        position,
        ...(config.parentId && { parentId: config.parentId }),
        data: {
            id: source.id,
            label: source.label ?? source.id,
            description: source.description,
            technology: source.technology,
            type: config.type,
            isExternal: config.isExternal ?? false,
            childCount: config.childCount,
            expanded: config.expanded,
            parentId: config.parentId,
        },
    };
}

/**
 * Factory with pre-configured node type.
 * Returns a function that creates nodes of a specific type.
 * 
 * @example
 * ```ts
 * const createContainer = nodeFactory("container");
 * const node = createContainer(containerData, { x: 0, y: 0 }, { parentId: "sys1" });
 * ```
 */
export function nodeFactory(type: C4NodeType) {
    return function (
        source: NodeSourceData,
        position: { x: number; y: number },
        options: Omit<NodeFactoryConfig, "type"> = {}
    ): Node<C4NodeData> {
        return createNode(source, position, { ...options, type });
    };
}

// Pre-configured factories for common node types
export const createPersonNode = nodeFactory("person");
export const createSystemNode = nodeFactory("system");
export const createContainerNode = nodeFactory("container");
export const createComponentNode = nodeFactory("component");
export const createDataStoreNode = nodeFactory("datastore");
export const createQueueNode = nodeFactory("queue");

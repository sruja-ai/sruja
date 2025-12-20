// apps/designer/src/types/nodeData.ts
// Extended node data types for React Flow nodes with governance and metadata

import type { C4NodeData } from "./index";

/**
 * Node data with technology information
 */
export interface NodeWithTechnology {
  technology?: string;
  tags?: string[];
  [key: string]: unknown;
}

/**
 * System node data with governance counts
 */
export interface SystemNodeData extends C4NodeData {
  type: "system";
  requirementCount?: number;
  adrCount?: number;
  scenarioCount?: number;
  flowCount?: number;
  tags?: string[];
}

/**
 * Container node data with governance counts
 */
export interface ContainerNodeData extends C4NodeData {
  type: "container";
  requirementCount?: number;
  adrCount?: number;
  scenarioCount?: number;
  flowCount?: number;
  tags?: string[];
}

/**
 * Component node data
 */
export interface ComponentNodeData extends C4NodeData {
  type: "component";
  tags?: string[];
}

/**
 * Person node data
 */
export interface PersonNodeData extends C4NodeData {
  type: "person";
}

/**
 * Data store node data
 */
export interface DataStoreNodeData extends C4NodeData {
  type: "datastore";
  tags?: string[];
}

/**
 * Queue node data
 */
export interface QueueNodeData extends C4NodeData {
  type: "queue";
  tags?: string[];
}

/**
 * Union type for all typed node data
 */
export type TypedNodeData =
  | SystemNodeData
  | ContainerNodeData
  | ComponentNodeData
  | PersonNodeData
  | DataStoreNodeData
  | QueueNodeData
  | C4NodeData;

/**
 * Type guard to check if node has technology
 */
export function hasTechnology(node: unknown): node is NodeWithTechnology {
  return (
    typeof node === "object" &&
    node !== null &&
    ("technology" in node || "tags" in node)
  );
}

/**
 * Type guard to check if node is SystemNodeData
 */
export function isSystemNodeData(
  node: C4NodeData | null | undefined
): node is SystemNodeData {
  return node !== null && node !== undefined && node.type === "system";
}

/**
 * Type guard to check if node is ContainerNodeData
 */
export function isContainerNodeData(
  node: C4NodeData
): node is ContainerNodeData {
  return node.type === "container";
}

/**
 * Type guard to check if node is ComponentNodeData
 */
export function isComponentNodeData(
  node: C4NodeData
): node is ComponentNodeData {
  return node.type === "component";
}

/**
 * Type guard to check if node is PersonNodeData
 */
export function isPersonNodeData(
  node: C4NodeData
): node is PersonNodeData {
  return node.type === "person";
}

/**
 * Safely get technology from node
 */
export function getNodeTechnology(node: unknown): string {
  if (hasTechnology(node)) {
    return String(node.technology ?? "").toLowerCase();
  }
  return "";
}

/**
 * Safely get tags from node
 */
export function getNodeTags(node: unknown): string[] {
  if (hasTechnology(node) && Array.isArray(node.tags)) {
    // Filter to only return string tags
    return node.tags.filter((tag): tag is string => typeof tag === "string");
  }
  return [];
}

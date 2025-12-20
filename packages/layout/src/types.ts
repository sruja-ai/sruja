import type { C4Id } from './brand'

export type NodeKind = 'system' | 'container' | 'component' | 'code'

export type Point = { x: number; y: number }
export type Size = { width: number; height: number }
export type Rect = Size & Point

export type TextStyle = {
  fontFamily: string
  fontSize: number
  fontWeight?: number | string
  lineHeight?: number
  maxWidth?: number
}

export type C4Element = {
  id: C4Id
  kind: NodeKind
  name: string
  description?: string
  children?: C4Element[]
}

export type TreeNode = {
  id: C4Id
  kind: NodeKind
  label: string
  children: TreeNode[]
}

export type LayoutNode = {
  id: C4Id
  bounds: Rect
  children: LayoutNode[]
}

export type LayoutConfig = {
  padding: number
  spacingY: number
  spacingX: number
}

export interface PositionedC4Node {
  nodeId: string;
  node?: any;
  bbox: Rect;
  contentBox: Rect;
  labelBox: Rect;
  parentId?: string;
  childrenIds: string[];
  depth: number;
  level: string;
  collapsed: boolean;
  visible: boolean;
  zIndex: number;
  ports: any[];
}

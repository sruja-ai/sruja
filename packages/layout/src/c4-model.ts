import { C4Id } from './brand'
import { ValidationError } from './utils/validation'

export type C4Level = 'landscape' | 'context' | 'container' | 'component' | 'deployment'
export const C4_LEVEL_DEPTH: Record<C4Level, number> = {
  landscape: 0,
  context: 1,
  container: 2,
  component: 3,
  deployment: 4
}

export type C4Kind =
  | 'Person'
  | 'SoftwareSystem'
  | 'Container'
  | 'Component'
  | 'DeploymentNode'
  | 'Database'
  | 'Queue'
  | 'Topic'
  | 'Cache'
  | 'FileSystem'
  | 'ExternalSystem'
  | 'ExternalContainer'
  | 'ExternalComponent'
  | 'EnterpriseBoundary'
  | 'SystemBoundary'
  | 'ContainerBoundary'
  | 'DeploymentGroup'

export function isContainerKind(kind: C4Kind): boolean {
  return (
    kind === 'SoftwareSystem' ||
    kind === 'Container' ||
    kind === 'DeploymentNode' ||
    kind === 'EnterpriseBoundary' ||
    kind === 'SystemBoundary' ||
    kind === 'ContainerBoundary' ||
    kind === 'DeploymentGroup'
  )
}

export function isExternalKind(kind: C4Kind): boolean {
  return kind === 'ExternalSystem' || kind === 'ExternalContainer' || kind === 'ExternalComponent'
}

export interface C4Node {
  id: C4Id
  label: string
  kind: C4Kind
  level: C4Level
  parentId?: C4Id
  description?: string
  technology?: string
  tags: ReadonlySet<string>
  links?: ReadonlyArray<{ url: string; title?: string }>
  widthHint?: number
  heightHint?: number
  aspectRatio?: number
  layoutPriority?: number
  hidden?: boolean
  pinnedPosition?: { x: number; y: number }
  collapseChildren?: boolean
  sortKey?: string
}

export interface C4Relationship {
  id: string
  from: C4Id
  to: C4Id
  label?: string
  technology?: string
  interaction?: 'sync' | 'async' | 'event'
  preferredRoute?: 'direct' | 'orthogonal' | 'curved'
  avoidNodes?: ReadonlyArray<C4Id>
  zIndex?: number
  tags?: ReadonlySet<string>
}

export interface C4Graph {
  nodes: ReadonlyMap<C4Id, C4Node>
  relationships: ReadonlyArray<C4Relationship>
  metadata?: {
    title?: string
    description?: string
    author?: string
    timestamp?: string
    version?: string
  }
}

export function createEmptyGraph(metadata?: C4Graph['metadata']): C4Graph {
  return { nodes: new Map(), relationships: [], metadata }
}

export function createC4Graph(nodes: C4Node[], relationships: C4Relationship[], metadata?: C4Graph['metadata']): C4Graph {
  const nodeMap = new Map<C4Id, C4Node>()
  const errors: ValidationError[] = []
  for (const node of nodes) {
    if (nodeMap.has(node.id)) errors.push(new ValidationError('E003', `Duplicate node ID: "${node.id}"`))
    nodeMap.set(node.id, node)
  }
  for (const node of nodes) {
    if (node.parentId && !nodeMap.has(node.parentId)) {
      errors.push(new ValidationError('E002', `Node "${node.id}" references non-existent parent "${node.parentId}"`))
    }
  }
  for (const rel of relationships) {
    if (!nodeMap.has(rel.from)) errors.push(new ValidationError('E004', `Relationship "${rel.id}" references non-existent source "${rel.from}"`))
    if (!nodeMap.has(rel.to)) errors.push(new ValidationError('E004', `Relationship "${rel.id}" references non-existent target "${rel.to}"`))
  }
  if (errors.length > 0) throw errors
  return { nodes: nodeMap, relationships, metadata }
}

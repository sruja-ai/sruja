import type { C4Id } from './brand'
import type { C4Kind, C4Level } from './c4-model'

export interface C4Viewport {
  width: number
  height: number
  padding: number
}

export function createDefaultViewport(): C4Viewport {
  return { width: 1200, height: 800, padding: 40 }
}

export interface C4ViewFilter {
  includeTags?: ReadonlySet<string>
  excludeTags?: ReadonlySet<string>
  includeKinds?: ReadonlySet<C4Kind>
  excludeKinds?: ReadonlySet<C4Kind>
  includeNodes?: ReadonlySet<C4Id>
  excludeNodes?: ReadonlySet<C4Id>
}

export function applyFilter(node: { id: C4Id; kind: C4Kind; tags: ReadonlySet<string> }, filter: C4ViewFilter): boolean {
  const passesInclude = (!filter.includeTags || hasIntersection(node.tags, filter.includeTags)) && (!filter.includeKinds || filter.includeKinds.has(node.kind)) && (!filter.includeNodes || filter.includeNodes.has(node.id))
  if (!passesInclude) return false
  const failsExclude = (filter.excludeTags && hasIntersection(node.tags, filter.excludeTags)) || (filter.excludeKinds && filter.excludeKinds.has(node.kind)) || (filter.excludeNodes && filter.excludeNodes.has(node.id))
  return !failsExclude
}

function hasIntersection<T>(a: ReadonlySet<T>, b: ReadonlySet<T>): boolean {
  for (const x of a) if (b.has(x)) return true
  return false
}

export interface C4ViewState {
  collapsedNodeIds: ReadonlySet<C4Id>
  hiddenNodeIds: ReadonlySet<C4Id>
  expandedLevels: ReadonlySet<C4Level>
  focusNodeId?: C4Id
  focusRadius?: number
  filter?: C4ViewFilter
  layoutPreset?: 'publication' | 'interactive' | 'presentation' | 'compact'
  direction?: 'TB' | 'LR' | 'RL'
  alignment?: 'start' | 'center' | 'end'
  showGrid?: boolean
  snapToGrid?: boolean
  gridSize?: number
}

export function createDefaultViewState(): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['landscape', 'context', 'container', 'component', 'deployment']),
    layoutPreset: 'interactive',
    direction: 'TB',
    alignment: 'center',
    showGrid: false,
    snapToGrid: false,
    gridSize: 20
  }
}

export function LandscapeView(): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['landscape']),
    layoutPreset: 'presentation', // Will be overridden if we set specific options, but for now we rely on the preset in c4-layout
    direction: 'TB',
    alignment: 'center',
    filter: { includeKinds: new Set(['SoftwareSystem', 'Person', 'EnterpriseBoundary'] as C4Kind[]) },
    // We can't set "strategy" directly on the view state yet as it's not in C4ViewState interface
    // But we can hint it via a new heuristic or updated interface later. 
    // For now, let's assume the consumer of this view will use it with LandscapePreset.
  }
}

export function SystemContextView(systemId: C4Id): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['context']),
    focusNodeId: systemId,
    focusRadius: 1,
    layoutPreset: 'interactive',
    direction: 'TB',
    alignment: 'center',
    filter: { includeKinds: new Set(['Person', 'SoftwareSystem', 'ExternalSystem'] as C4Kind[]) }
  }
}

export function ContainerView(systemId: C4Id): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['context', 'container']),
    focusNodeId: systemId,
    focusRadius: 2,
    layoutPreset: 'interactive',
    direction: 'TB',
    alignment: 'center',
    filter: { includeKinds: new Set(['Person', 'SoftwareSystem', 'Container', 'Database', 'Queue', 'Cache'] as C4Kind[]) }
  }
}

export function ComponentView(containerId: C4Id): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['container', 'component']),
    focusNodeId: containerId,
    focusRadius: 2,
    layoutPreset: 'interactive',
    direction: 'TB',
    alignment: 'center',
    filter: { includeKinds: new Set(['Container', 'Component', 'Database', 'Queue', 'Cache'] as C4Kind[]) }
  }
}

export function DeploymentView(environment: string): C4ViewState {
  return {
    collapsedNodeIds: new Set(),
    hiddenNodeIds: new Set(),
    expandedLevels: new Set(['deployment']),
    layoutPreset: 'presentation',
    direction: 'LR',
    alignment: 'start',
    filter: { includeTags: new Set([environment, 'infrastructure']), includeKinds: new Set(['DeploymentNode', 'Container', 'Database', 'Queue', 'DeploymentGroup'] as C4Kind[]) }
  }
}

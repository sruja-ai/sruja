import type { C4Graph, C4Node } from '../c4-model'
import type { C4Id } from '../brand'
import type { C4ViewState } from '../c4-view'
import { applyFilter } from '../c4-view'

export interface HierarchyNode {
  id: C4Id
  node: C4Node
  parent?: HierarchyNode
  children: HierarchyNode[]
  depth: number
  subtreeSize: number
  subtreeDepth: number
}

export interface HierarchyTree {
  roots: HierarchyNode[]
  nodeMap: Map<C4Id, HierarchyNode>
  maxDepth: number
}

export function buildHierarchy(graph: C4Graph, view: C4ViewState): HierarchyTree {
  const nodeMap = new Map<C4Id, HierarchyNode>()
  for (const [id, node] of graph.nodes) {
    if (view.hiddenNodeIds.has(id)) continue
    if (node.hidden) continue
    if (view.filter && !applyFilter(node, view.filter)) continue
    nodeMap.set(id, { id, node, parent: undefined, children: [], depth: 0, subtreeSize: 1, subtreeDepth: 0 })
  }
  const roots: HierarchyNode[] = []
  for (const [_id, hNode] of nodeMap) {
    const parentId = hNode.node.parentId
    if (parentId && nodeMap.has(parentId)) {
      const parent = nodeMap.get(parentId)!
      hNode.parent = parent
      parent.children.push(hNode)
    } else {
      roots.push(hNode)
    }
  }
  function calc(node: HierarchyNode, depth: number): { size: number; maxDepth: number } {
    node.depth = depth
    if (node.children.length === 0) {
      node.subtreeSize = 1
      node.subtreeDepth = 0
      return { size: 1, maxDepth: depth }
    }
    let total = 1
    let maxChildDepth = depth
    for (const child of node.children) {
      const m = calc(child, depth + 1)
      total += m.size
      maxChildDepth = Math.max(maxChildDepth, m.maxDepth)
    }
    node.subtreeSize = total
    node.subtreeDepth = maxChildDepth - depth
    return { size: total, maxDepth: maxChildDepth }
  }
  let maxDepth = 0
  for (const root of roots) {
    const m = calc(root, 0)
    maxDepth = Math.max(maxDepth, m.maxDepth)
  }
  // Stable deterministic sort
  function sortChildren(node: HierarchyNode) {
    node.children.sort((a, b) => {
      const pa = a.node.layoutPriority ?? 50
      const pb = b.node.layoutPriority ?? 50
      if (pa !== pb) return pb - pa
      const ka = a.node.sortKey ?? a.node.label
      const kb = b.node.sortKey ?? b.node.label
      return ka.localeCompare(kb)
    })
    for (const c of node.children) sortChildren(c)
  }
  roots.sort((a, b) => {
    const pa = a.node.layoutPriority ?? 50
    const pb = b.node.layoutPriority ?? 50
    if (pa !== pb) return pb - pa
    const ka = a.node.sortKey ?? a.node.label
    const kb = b.node.sortKey ?? b.node.label
    return ka.localeCompare(kb)
  })
  for (const r of roots) sortChildren(r)
  return { roots, nodeMap, maxDepth }
}

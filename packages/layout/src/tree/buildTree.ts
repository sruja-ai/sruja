import { C4Element, TreeNode } from '../types'

export function buildTree(model: C4Element): TreeNode {
  const seen = new Set<string>()
  function visit(el: C4Element): TreeNode {
    if (seen.has(el.id)) throw new Error(`Duplicate id: ${el.id}`)
    seen.add(el.id)
    const children = (el.children ?? []).map(visit)
    return { id: el.id, kind: el.kind, label: el.name, children }
  }
  return visit(model)
}

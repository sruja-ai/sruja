import type { EdgeTypes } from '@xyflow/react'
import { RoutedEdge } from './RoutedEdge'
import { RelationEdge } from './RelationEdge'

export { RoutedEdge } from './RoutedEdge'
export { RelationEdge } from './RelationEdge'

export const edgeTypes: EdgeTypes = {
  routed: RoutedEdge,
  relation: RelationEdge,
}

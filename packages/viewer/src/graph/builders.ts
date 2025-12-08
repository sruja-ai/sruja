import type { ElementDefinition } from 'cytoscape'
import type { ArchitectureJSON } from '../types'

export function buildNodes(arch: ArchitectureJSON['architecture'], getPos: (id: string) => { x: number; y: number } | undefined, nodeIds: Set<string>): ElementDefinition[] {
  const elements: ElementDefinition[] = []

  if (arch.persons) {
    for (const person of arch.persons) {
      nodeIds.add(person.id)
      elements.push({
        data: { id: person.id, label: person.label || person.id, type: 'person', metadata: person.metadata },
        position: getPos(person.id),
      })
    }
  }

  if (arch.systems) {
    for (const system of arch.systems) {
      nodeIds.add(system.id)
      elements.push({ data: { id: system.id, label: system.label || system.id, type: 'system', metadata: system.metadata } })

      if (system.containers) {
        for (const container of system.containers) {
          const id = `${system.id}.${container.id}`
          nodeIds.add(id)
          elements.push({ data: { id, label: container.label || container.id, type: 'container', parent: system.id, metadata: container.metadata }, position: getPos(id) })
          if (container.components) {
            for (const component of container.components) {
              const cid = `${system.id}.${container.id}.${component.id}`
              nodeIds.add(cid)
              elements.push({ data: { id: cid, label: component.label || component.id, type: 'component', parent: `${system.id}.${container.id}`, metadata: component.metadata }, position: getPos(cid) })
            }
          }
        }
      }
      if (system.datastores) {
        for (const ds of system.datastores) {
          const id = `${system.id}.${ds.id}`
          nodeIds.add(id)
          elements.push({ data: { id, label: ds.label || ds.id, type: 'datastore', parent: system.id, metadata: ds.metadata }, position: getPos(id) })
        }
      }
      if (system.queues) {
        for (const q of system.queues) {
          const id = `${system.id}.${q.id}`
          nodeIds.add(id)
          elements.push({ data: { id, label: q.label || q.id, type: 'queue', parent: system.id, metadata: q.metadata }, position: getPos(id) })
        }
      }
    }
  }

  if (arch.containers) {
    for (const container of arch.containers) {
      nodeIds.add(container.id)
      elements.push({ data: { id: container.id, label: container.label || container.id, type: 'container', metadata: container.metadata }, position: getPos(container.id) })
      if (container.components) {
        for (const component of container.components) {
          const id = `${container.id}.${component.id}`
          nodeIds.add(id)
          elements.push({ data: { id, label: component.label || component.id, type: 'component', parent: container.id, metadata: component.metadata }, position: getPos(id) })
        }
      }
    }
  }

  if (arch.datastores) {
    for (const ds of arch.datastores) {
      nodeIds.add(ds.id)
      elements.push({ data: { id: ds.id, label: ds.label || ds.id, type: 'datastore', metadata: ds.metadata }, position: getPos(ds.id) })
    }
  }

  if (arch.queues) {
    for (const q of arch.queues) {
      nodeIds.add(q.id)
      elements.push({ data: { id: q.id, label: q.label || q.id, type: 'queue', metadata: q.metadata } })
    }
  }

  if (arch.requirements) {
    for (const req of arch.requirements) {
      nodeIds.add(req.id)
      elements.push({ data: { id: req.id, label: req.title || req.id, type: 'requirement' }, position: getPos(req.id) })
    }
  }

  if (arch.adrs) {
    for (const adr of arch.adrs) {
      nodeIds.add(adr.id)
      elements.push({ data: { id: adr.id, label: adr.title || adr.id, type: 'adr' }, position: getPos(adr.id) })
    }
  }

  if (arch.deployment) {
    for (const node of arch.deployment) {
      nodeIds.add(node.id)
      elements.push({ data: { id: node.id, label: node.label || node.id, type: 'deployment' }, position: getPos(node.id) })
    }
  }

  return elements
}

export function buildTopLevelRelations(arch: ArchitectureJSON['architecture'], resolve: (id: string) => string | null): ElementDefinition[] {
  const edges: ElementDefinition[] = []
  if (arch.relations) {
    for (const relation of arch.relations) {
      const fromId = resolve(relation.from)
      const toId = resolve(relation.to)
      if (fromId && toId) {
        edges.push({ data: { id: `${fromId}-${toId}`, source: fromId, target: toId, label: relation.label || relation.verb || '' }, group: 'edges' })
      }
    }
  }
  return edges
}

export function buildSystemRelations(arch: ArchitectureJSON['architecture'], nodeIds: Set<string>): ElementDefinition[] {
  const edges: ElementDefinition[] = []
  if (arch.systems) {
    for (const system of arch.systems) {
      if (system.relations) {
        for (const relation of system.relations) {
          const fromId = `${system.id}.${relation.from}`
          const toId = `${system.id}.${relation.to}`
          if (nodeIds.has(fromId) && nodeIds.has(toId)) {
            edges.push({ data: { id: `${fromId}-${toId}`, source: fromId, target: toId, label: relation.label || relation.verb || '' }, group: 'edges' })
          }
        }
      }
    }
  }
  return edges
}

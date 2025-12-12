import { describe, it, expect } from 'vitest'
import { createC4Id } from '../brand'
import { createC4Graph } from '../c4-model'
import { layout } from '../c4-layout'
import { createDefaultViewState } from '../c4-view'

describe('Compound Layout', () => {
    it('places component inside container', () => {
        // Graph: System -> Container -> Component
        const system = { id: createC4Id('sys'), label: 'System', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
        const container = { id: createC4Id('cont'), parentId: system.id, label: 'Container', kind: 'Container', level: 'container', tags: new Set<string>() }
        const component = { id: createC4Id('comp'), parentId: container.id, label: 'Component', kind: 'Component', level: 'component', tags: new Set<string>() }

        const graph = createC4Graph(
            [system as any, container as any, component as any],
            []
        )

        // View: Expand System and Container
        // Use a custom view that shows all levels
        const view = createDefaultViewState()
        // Default view expands all levels: 'landscape', 'context', 'container', 'component', 'deployment'

        const result = layout(graph, view)

        const sNode = result.nodes.get(system.id)!
        const cNode = result.nodes.get(container.id)!
        const kNode = result.nodes.get(component.id)!

        expect(sNode).toBeDefined()
        expect(cNode).toBeDefined()
        expect(kNode).toBeDefined()

        // Assert Containment
        // Container should be inside System
        expect(cNode.bbox.x).toBeGreaterThanOrEqual(sNode.bbox.x)
        expect(cNode.bbox.y).toBeGreaterThanOrEqual(sNode.bbox.y)
        expect(cNode.bbox.x + cNode.bbox.width).toBeLessThanOrEqual(sNode.bbox.x + sNode.bbox.width)
        expect(cNode.bbox.y + cNode.bbox.height).toBeLessThanOrEqual(sNode.bbox.y + sNode.bbox.height)

        // Component should be inside Container
        expect(kNode.bbox.x).toBeGreaterThanOrEqual(cNode.bbox.x)
        expect(kNode.bbox.y).toBeGreaterThanOrEqual(cNode.bbox.y)
        expect(kNode.bbox.x + kNode.bbox.width).toBeLessThanOrEqual(cNode.bbox.x + cNode.bbox.width)
        expect(kNode.bbox.y + kNode.bbox.height).toBeLessThanOrEqual(cNode.bbox.y + cNode.bbox.height)
    })

    it('collapses children when collapseChildren is true', () => {
        // Graph: System -> Container (collapsed) -> Component
        const system = { id: createC4Id('sys'), label: 'System', kind: 'SoftwareSystem', level: 'context', tags: new Set<string>() }
        // Container has collapseChildren: true, so Component should NOT be laid out inside
        const container = { id: createC4Id('cont'), parentId: system.id, label: 'Container', kind: 'Container', level: 'container', tags: new Set<string>(), collapseChildren: true }
        const component = { id: createC4Id('comp'), parentId: container.id, label: 'Component', kind: 'Component', level: 'component', tags: new Set<string>() }

        const graph = createC4Graph(
            [system as any, container as any, component as any],
            []
        )

        const view = createDefaultViewState()
        const result = layout(graph, view)

        const sNode = result.nodes.get(system.id)!
        const cNode = result.nodes.get(container.id)!
        const kNode = result.nodes.get(component.id)

        expect(sNode).toBeDefined()
        expect(cNode).toBeDefined()

        // Container should still be inside System
        expect(cNode.bbox.x).toBeGreaterThanOrEqual(sNode.bbox.x)
        expect(cNode.bbox.y).toBeGreaterThanOrEqual(sNode.bbox.y)
        expect(cNode.bbox.x + cNode.bbox.width).toBeLessThanOrEqual(sNode.bbox.x + sNode.bbox.width)
        expect(cNode.bbox.y + cNode.bbox.height).toBeLessThanOrEqual(sNode.bbox.y + sNode.bbox.height)

        // Key assertion: Container should be collapsed (sized as leaf, not containing component)
        expect(cNode.collapsed).toBe(true)

        // Component might still be in the result but should NOT be positioned inside Container
        // (it could be positioned elsewhere or not laid out at all)
        if (kNode) {
            // If component is in result, it should NOT be inside the collapsed container
            const isInsideContainer =
                kNode.bbox.x >= cNode.bbox.x &&
                kNode.bbox.y >= cNode.bbox.y &&
                kNode.bbox.x + kNode.bbox.width <= cNode.bbox.x + cNode.bbox.width &&
                kNode.bbox.y + kNode.bbox.height <= cNode.bbox.y + cNode.bbox.height

            // This is the key fix - when collapseChildren is true, component should NOT be inside
            expect(isInsideContainer).toBe(false)
        }
    })
})


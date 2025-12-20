import { describe, it, expect } from 'vitest'
import {
    placeLabels,
    detectLabelCollisions,
    type EdgeForLabeling,
    type NodeForLabeling,
    type LabelPlacement
} from '../algorithms/label-placer'

describe('Label Placer', () => {
    describe('detectLabelCollisions', () => {
        it('detects overlapping labels', () => {
            const placements = new Map<string, LabelPlacement>([
                ['edge1', {
                    edgeId: 'edge1',
                    position: { x: 100, y: 100 },
                    rotation: 0,
                    anchor: 'middle',
                    bounds: { x: 60, y: 90, width: 80, height: 20 }
                }],
                ['edge2', {
                    edgeId: 'edge2',
                    position: { x: 120, y: 105 },
                    rotation: 0,
                    anchor: 'middle',
                    bounds: { x: 80, y: 95, width: 80, height: 20 }
                }]
            ])
            const nodes = new Map<string, NodeForLabeling>()

            const results = detectLabelCollisions(placements, nodes, 0)

            expect(results.get('edge1')?.hasCollision).toBe(true)
            expect(results.get('edge2')?.hasCollision).toBe(true)
        })

        it('detects label-node collisions', () => {
            const placements = new Map<string, LabelPlacement>([
                ['edge1', {
                    edgeId: 'edge1',
                    position: { x: 150, y: 150 },
                    rotation: 0,
                    anchor: 'middle',
                    bounds: { x: 110, y: 140, width: 80, height: 20 }
                }]
            ])
            const nodes = new Map<string, NodeForLabeling>([
                ['node1', {
                    id: 'node1',
                    bbox: { x: 100, y: 100, width: 100, height: 100 }
                }]
            ])

            const results = detectLabelCollisions(placements, nodes, 0)

            expect(results.get('edge1')?.hasCollision).toBe(true)
            expect(results.get('edge1')?.collidingWith).toContain('node:node1')
        })

        it('returns no collision for non-overlapping elements', () => {
            const placements = new Map<string, LabelPlacement>([
                ['edge1', {
                    edgeId: 'edge1',
                    position: { x: 50, y: 50 },
                    rotation: 0,
                    anchor: 'middle',
                    bounds: { x: 10, y: 40, width: 80, height: 20 }
                }]
            ])
            const nodes = new Map<string, NodeForLabeling>([
                ['node1', {
                    id: 'node1',
                    bbox: { x: 200, y: 200, width: 100, height: 100 }
                }]
            ])

            const results = detectLabelCollisions(placements, nodes, 0)

            expect(results.get('edge1')?.hasCollision).toBe(false)
        })
    })

    describe('placeLabels', () => {
        it('places labels at edge midpoint by default', () => {
            const edges: EdgeForLabeling[] = [{
                id: 'edge1',
                label: 'Uses',
                points: [{ x: 0, y: 0 }, { x: 200, y: 0 }]
            }]
            const nodes = new Map<string, NodeForLabeling>()

            const result = placeLabels(edges, nodes)

            expect(result.get('edge1')).toBeDefined()
            const placement = result.get('edge1')!
            expect(placement.position.x).toBeCloseTo(100, 0)
            expect(placement.anchor).toBe('middle')
        })

        it('skips edges without labels', () => {
            const edges: EdgeForLabeling[] = [
                { id: 'edge1', label: 'Has Label', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] },
                { id: 'edge2', points: [{ x: 0, y: 50 }, { x: 100, y: 50 }] } // No label
            ]
            const nodes = new Map<string, NodeForLabeling>()

            const result = placeLabels(edges, nodes)

            expect(result.has('edge1')).toBe(true)
            expect(result.has('edge2')).toBe(false)
        })

        it('repositions colliding labels', () => {
            // Two edges with nearly identical paths will have labels that collide at midpoint
            const edges: EdgeForLabeling[] = [
                { id: 'edge1', label: 'First', points: [{ x: 0, y: 0 }, { x: 200, y: 0 }] },
                { id: 'edge2', label: 'Second', points: [{ x: 0, y: 5 }, { x: 200, y: 5 }] }
            ]
            const nodes = new Map<string, NodeForLabeling>()

            const result = placeLabels(edges, nodes, { padding: 10 })

            const p1 = result.get('edge1')!
            const p2 = result.get('edge2')!

            // Verify they no longer overlap
            const p1Right = p1.bounds.x + p1.bounds.width
            const p1Bottom = p1.bounds.y + p1.bounds.height
            const p2Right = p2.bounds.x + p2.bounds.width
            const p2Bottom = p2.bounds.y + p2.bounds.height

            const overlaps = !(
                p1Right < p2.bounds.x ||
                p2Right < p1.bounds.x ||
                p1Bottom < p2.bounds.y ||
                p2Bottom < p1.bounds.y
            )

            expect(overlaps).toBe(false)
        })
    })
})

// Layout validation utilities for testing position stability
import type { Node } from '@xyflow/react';
import type { C4NodeData } from '../types';

export interface NodePosition {
    id: string;
    x: number;
    y: number;
    type?: string;
    parentId?: string;
}

export interface LayoutValidationResult {
    success: boolean;
    stableNodes: number;
    movedNodes: number;
    totalNodes: number;
    violations: LayoutViolation[];
    metrics: {
        averageMovement: number;
        maxMovement: number;
        stableNodePercentage: number;
    };
}

export interface LayoutViolation {
    nodeId: string;
    previousPosition: { x: number; y: number };
    newPosition: { x: number; y: number };
    movement: number;
    threshold: number;
    reason: string;
}

const DEFAULT_MOVEMENT_THRESHOLD = 50; // pixels

/**
 * Validate that node positions are preserved after layout changes
 */
export function validatePositionPreservation(
    previousPositions: Map<string, NodePosition>,
    currentNodes: Node<C4NodeData>[],
    options: {
        movementThreshold?: number;
        ignoreRootNodes?: boolean;
        ignoreChildren?: boolean;
    } = {}
): LayoutValidationResult {
    const {
        movementThreshold = DEFAULT_MOVEMENT_THRESHOLD,
        ignoreRootNodes = false,
        ignoreChildren = false
    } = options;

    const violations: LayoutViolation[] = [];
    let stableNodes = 0;
    let movedNodes = 0;
    let totalMovement = 0;
    let maxMovement = 0;

    currentNodes.forEach(node => {
        // Skip if we should ignore this node type
        if (ignoreRootNodes && !node.parentId) return;
        if (ignoreChildren && node.parentId) return;

        const previous = previousPositions.get(node.id);
        if (!previous) {
            // New node - not a violation, but track it
            movedNodes++;
            return;
        }

        const movement = Math.sqrt(
            Math.pow(node.position.x - previous.x, 2) +
            Math.pow(node.position.y - previous.y, 2)
        );

        totalMovement += movement;
        maxMovement = Math.max(maxMovement, movement);

        if (movement > movementThreshold) {
            movedNodes++;
            violations.push({
                nodeId: node.id,
                previousPosition: { x: previous.x, y: previous.y },
                newPosition: { x: node.position.x, y: node.position.y },
                movement,
                threshold: movementThreshold,
                reason: node.parentId 
                    ? 'Child node moved beyond threshold' 
                    : 'Root node moved beyond threshold'
            });
        } else {
            stableNodes++;
        }
    });

    const totalNodes = currentNodes.length;
    const stableNodePercentage = totalNodes > 0 
        ? (stableNodes / totalNodes) * 100 
        : 0;
    const averageMovement = totalNodes > 0 
        ? totalMovement / totalNodes 
        : 0;

    return {
        success: violations.length === 0,
        stableNodes,
        movedNodes,
        totalNodes,
        violations,
        metrics: {
            averageMovement,
            maxMovement,
            stableNodePercentage
        }
    };
}

/**
 * Extract node positions from React Flow nodes
 */
export function extractNodePositions(nodes: Node<C4NodeData>[]): Map<string, NodePosition> {
    const positions = new Map<string, NodePosition>();
    
    nodes.forEach(node => {
        positions.set(node.id, {
            id: node.id,
            x: node.position.x,
            y: node.position.y,
            type: node.data.type,
            parentId: node.parentId
        });
    });

    return positions;
}

/**
 * Compare two layout validation results
 */
export function compareValidationResults(
    before: LayoutValidationResult,
    after: LayoutValidationResult
): {
    improved: boolean;
    stabilityChange: number;
    violationsChange: number;
    summary: string;
} {
    const stabilityChange = after.metrics.stableNodePercentage - before.metrics.stableNodePercentage;
    const violationsChange = after.violations.length - before.violations.length;
    const improved = violationsChange < 0 && stabilityChange >= 0;

    const summary = `
    Stability: ${before.metrics.stableNodePercentage.toFixed(1)}% → ${after.metrics.stableNodePercentage.toFixed(1)}% (${stabilityChange >= 0 ? '+' : ''}${stabilityChange.toFixed(1)}%)
    Violations: ${before.violations.length} → ${after.violations.length} (${violationsChange >= 0 ? '+' : ''}${violationsChange})
    Average Movement: ${before.metrics.averageMovement.toFixed(1)}px → ${after.metrics.averageMovement.toFixed(1)}px
    Max Movement: ${before.metrics.maxMovement.toFixed(1)}px → ${after.metrics.maxMovement.toFixed(1)}px
    `;

    return {
        improved,
        stabilityChange,
        violationsChange,
        summary
    };
}

/**
 * Generate a detailed report of layout validation
 */
export function generateValidationReport(
    result: LayoutValidationResult,
    context?: string
): string {
    const lines: string[] = [];

    if (context) {
        lines.push(`\n=== Layout Validation Report: ${context} ===\n`);
    } else {
        lines.push('\n=== Layout Validation Report ===\n');
    }

    lines.push(`Status: ${result.success ? '✅ PASS' : '❌ FAIL'}`);
    lines.push(`Total Nodes: ${result.totalNodes}`);
    lines.push(`Stable Nodes: ${result.stableNodes} (${result.metrics.stableNodePercentage.toFixed(1)}%)`);
    lines.push(`Moved Nodes: ${result.movedNodes}`);
    lines.push(`Violations: ${result.violations.length}`);
    lines.push(`Average Movement: ${result.metrics.averageMovement.toFixed(2)}px`);
    lines.push(`Max Movement: ${result.metrics.maxMovement.toFixed(2)}px`);

    if (result.violations.length > 0) {
        lines.push('\n--- Violations ---');
        result.violations.slice(0, 10).forEach((violation, index) => {
            lines.push(
                `${index + 1}. Node "${violation.nodeId}": ` +
                `moved ${violation.movement.toFixed(1)}px ` +
                `(${violation.previousPosition.x.toFixed(0)}, ${violation.previousPosition.y.toFixed(0)}) → ` +
                `(${violation.newPosition.x.toFixed(0)}, ${violation.newPosition.y.toFixed(0)}) ` +
                `[${violation.reason}]`
            );
        });
        if (result.violations.length > 10) {
            lines.push(`... and ${result.violations.length - 10} more violations`);
        }
    }

    lines.push('\n');
    return lines.join('\n');
}

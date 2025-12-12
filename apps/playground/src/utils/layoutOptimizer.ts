// Layout optimizer that tries different configurations to maximize diagram quality
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../types';
import type { DiagramQualityMetrics, QualityWeights } from './diagramQuality';
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from './diagramQuality';
import { applySrujaLayout } from './layoutEngine';
import { applyC4LevelLayout } from './c4LevelLayout';
import { selectLayoutConfig } from './layoutRules';

export interface LayoutOptimizationOptions {
    nodes: Node<C4NodeData>[];
    edges: Edge[];
    currentLevel: string;
    focusedSystemId?: string;
    focusedContainerId?: string;
    expandedNodes?: Set<string>;
    viewportSize?: { width: number; height: number };
    weights?: QualityWeights;
    maxIterations?: number;
}

export interface OptimizationResult {
    bestLayout: { nodes: Node<C4NodeData>[]; edges: Edge[] };
    bestScore: number;
    bestMetrics: DiagramQualityMetrics;
    triedConfigurations: Array<{
        config: string;
        score: number;
        metrics: DiagramQualityMetrics;
    }>;
}

/**
 * Optimize layout by trying different configurations and selecting the best
 */
export async function optimizeLayout(
    options: LayoutOptimizationOptions
): Promise<OptimizationResult> {
    const {
        nodes,
        edges,
        currentLevel,
        focusedSystemId,
        focusedContainerId,
        expandedNodes,
        viewportSize = { width: 1920, height: 1080 },
        weights = DEFAULT_QUALITY_WEIGHTS,
        maxIterations = 5
    } = options;

    const triedConfigurations: OptimizationResult['triedConfigurations'] = [];
    let bestScore = -1;
    let bestLayout: { nodes: Node<C4NodeData>[]; edges: Edge[] } | null = null;
    let bestMetrics: DiagramQualityMetrics | null = null;

    // Use rules-based configuration selection
    const primaryConfig = selectLayoutConfig(
        nodes,
        edges,
        currentLevel as any,
        focusedSystemId,
        focusedContainerId,
        expandedNodes
    );
    
    // Generate alternative configurations to try
    const configurations = generateAlternativeConfigs(primaryConfig);
    const configsToTry = [primaryConfig, ...configurations].slice(0, maxIterations);

    for (const config of configsToTry) {
        try {
            let layoutResult: { nodes: Node<C4NodeData>[]; edges: Edge[] };

            // Apply layout based on configuration (only our custom engines)
            if (config.engine === 'c4level') {
                layoutResult = await applyC4LevelLayout(nodes, edges, {
                    level: currentLevel as any,
                    focusedSystemId,
                    focusedContainerId,
                    expandedNodes
                });
            } else {
                // sruja
                layoutResult = await applySrujaLayout(nodes, edges, {
                    direction: config.direction,
                    level: currentLevel,
                    expandedNodes
                });
            }

            // Calculate quality metrics
            const metrics = calculateDiagramQuality(
                layoutResult.nodes,
                layoutResult.edges,
                viewportSize,
                weights
            );

            const configName = `${config.engine}-${config.direction}`;
            triedConfigurations.push({
                config: configName,
                score: metrics.weightedScore,
                metrics
            });

            // Track best
            if (metrics.weightedScore > bestScore) {
                bestScore = metrics.weightedScore;
                bestLayout = layoutResult;
                bestMetrics = metrics;
            }
        } catch (error) {
            console.warn(`Layout configuration ${config.engine}-${config.direction} failed:`, error);
        }
    }

    // If no layout succeeded, return original
    if (!bestLayout || !bestMetrics) {
        const fallbackMetrics = calculateDiagramQuality(nodes, edges, viewportSize, weights);
        return {
            bestLayout: { nodes, edges },
            bestScore: fallbackMetrics.weightedScore,
            bestMetrics: fallbackMetrics,
            triedConfigurations
        };
    }

    return {
        bestLayout,
        bestScore,
        bestMetrics,
        triedConfigurations
    };
}

interface LayoutConfiguration {
    engine: 'sruja' | 'c4level'; // Only our custom engines
    direction: 'DOWN' | 'RIGHT' | 'UP' | 'LEFT';
    options?: any;
}

/**
 * Generate alternative configurations to try (variations of primary)
 */
function generateAlternativeConfigs(
    primary: LayoutConfiguration
): LayoutConfiguration[] {
    const alternatives: LayoutConfiguration[] = [];

    // Try alternative engines (only our custom engines)
    const engines: Array<'sruja' | 'c4level'> = ['sruja', 'c4level'];
    engines.forEach(engine => {
        if (engine !== primary.engine) {
            alternatives.push({
                engine,
                direction: primary.direction,
                options: primary.options
            });
        }
    });

    // Try alternative directions (if not already tried)
    if (primary.direction === 'DOWN') {
        alternatives.push({
            engine: primary.engine,
            direction: 'RIGHT',
            options: primary.options
        });
    }

    // Limit to most promising alternatives
    return alternatives.slice(0, 3);
}

/**
 * Optimize layout with incremental improvements
 * Tries to improve specific quality aspects
 */
export async function optimizeLayoutIncremental(
    options: LayoutOptimizationOptions
): Promise<OptimizationResult> {
    const {
        nodes,
        edges,
        currentLevel,
        focusedSystemId,
        focusedContainerId,
        expandedNodes,
        viewportSize = { width: 1920, height: 1080 },
        weights = DEFAULT_QUALITY_WEIGHTS
    } = options;

    // Start with current layout
    let currentNodes = nodes;
    let currentEdges = edges;
    let currentMetrics = calculateDiagramQuality(currentNodes, currentEdges, viewportSize, weights);
    let bestScore = currentMetrics.weightedScore;
    let bestLayout = { nodes: currentNodes, edges: currentEdges };
    let bestMetrics = currentMetrics;

    const triedConfigurations: OptimizationResult['triedConfigurations'] = [];

    // Try to improve specific aspects
    const improvements = [
        { aspect: 'overlap', weight: weights.overlap },
        { aspect: 'edgeCrossings', weight: weights.edgeCrossings },
        { aspect: 'edgesOverNodes', weight: weights.edgesOverNodes }
    ].sort((a, b) => b.weight - a.weight); // Try most important aspects first

    for (const improvement of improvements) {
        // If this aspect is already good, skip
        if (improvement.aspect === 'overlap' && currentMetrics.overlapScore > 95) continue;
        if (improvement.aspect === 'edgeCrossings' && currentMetrics.edgeCrossings === 0) continue;
        if (improvement.aspect === 'edgesOverNodes' && currentMetrics.edgesOverNodes === 0) continue;

        // Use rules-based config selection
        const primaryConfig = selectLayoutConfig(
            currentNodes,
            currentEdges,
            currentLevel as any,
            focusedSystemId,
            focusedContainerId,
            expandedNodes
        );
        const configs = generateAlternativeConfigs(primaryConfig);
        
        for (const config of configs.slice(0, 2)) { // Limit to 2 tries per aspect
            try {
                let layoutResult: { nodes: Node<C4NodeData>[]; edges: Edge[] };

                if (config.engine === 'c4level') {
                    layoutResult = await applyC4LevelLayout(currentNodes, currentEdges, {
                        level: currentLevel as any,
                        focusedSystemId,
                        focusedContainerId,
                        expandedNodes
                    });
                } else {
                    layoutResult = await applySrujaLayout(currentNodes, currentEdges, {
                        direction: config.direction,
                        level: currentLevel,
                        expandedNodes
                    });
                }

                const metrics = calculateDiagramQuality(
                    layoutResult.nodes,
                    layoutResult.edges,
                    viewportSize,
                    weights
                );

                const configName = `${config.engine}-${config.direction}-${improvement.aspect}`;
                triedConfigurations.push({
                    config: configName,
                    score: metrics.weightedScore,
                    metrics
                });

                // Accept if better overall score
                if (metrics.weightedScore > bestScore) {
                    bestScore = metrics.weightedScore;
                    bestLayout = layoutResult;
                    bestMetrics = metrics;
                    currentNodes = layoutResult.nodes;
                    currentEdges = layoutResult.edges;
                    currentMetrics = metrics;
                }
            } catch (error) {
                console.warn(`Incremental optimization failed:`, error);
            }
        }
    }

    return {
        bestLayout,
        bestScore,
        bestMetrics: bestMetrics!,
        triedConfigurations
    };
}

/**
 * Get optimization recommendations based on current metrics
 */
export function getOptimizationRecommendations(metrics: DiagramQualityMetrics): string[] {
    const recommendations: string[] = [];

    if (metrics.overlapScore < 80) {
        recommendations.push(`High node overlap (${metrics.overlappingNodes.length} overlaps). Consider increasing node spacing or using a different layout engine.`);
    }

    if (metrics.edgeCrossings > 5) {
        recommendations.push(`Many edge crossings (${metrics.edgeCrossings}). Try a hierarchical layout or orthogonal edge routing.`);
    }

    if (metrics.edgesOverNodes > 3) {
        recommendations.push(`Edges passing over nodes (${metrics.edgesOverNodes}). Consider edge routing or node repositioning.`);
    }

    if (metrics.edgeBends > 10) {
        recommendations.push(`Many edge bends (${metrics.edgeBends}). Consider simplifying edge routing.`);
    }

    if (metrics.spacingScore < 70) {
        recommendations.push(`Poor node spacing. Increase minimum spacing between nodes.`);
    }

    if (metrics.hierarchyScore < 80) {
        recommendations.push(`Hierarchy violations (${metrics.parentChildContainment.length}). Ensure children are properly contained within parents.`);
    }

    if (metrics.parentChildSizeViolations.length > 0) {
        recommendations.push(`Parent nodes too small for children (${metrics.parentChildSizeViolations.length}). Increase parent node sizes to properly contain children.`);
    }

    if (metrics.aspectRatioScore < 70) {
        const MIN = 0.5; const MAX = 2.0;
        if (metrics.aspectRatio < MIN) {
            recommendations.push(`Diagram is too tall (aspect ratio: ${metrics.aspectRatio.toFixed(2)}). Consider horizontal layout or reducing vertical spacing.`);
        } else if (metrics.aspectRatio > MAX) {
            recommendations.push(`Diagram is too wide (aspect ratio: ${metrics.aspectRatio.toFixed(2)}). Consider vertical layout or reducing horizontal spacing.`);
        }
    }

    if (metrics.viewportUtilization < 0.5) {
        recommendations.push(`Low viewport utilization (${(metrics.viewportUtilization * 100).toFixed(0)}%). Consider adjusting layout scale.`);
    }

    if (recommendations.length === 0) {
        recommendations.push('Layout quality is good! No major issues detected.');
    }

    return recommendations;
}

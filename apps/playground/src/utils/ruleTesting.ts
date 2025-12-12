// Utility for testing different optimization rule combinations
// Helps find optimal rule configurations by systematically testing variations

import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../types';
import type { DiagramQualityMetrics } from './diagramQuality';
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from './diagramQuality';
import { applySrujaLayout, type LayoutOptions } from './layoutEngine';
import type { OptimizationRuleConfig } from './optimizationRules';
import { 
    mergeOptimizationRules,
    createRulePreset,
    validateOptimizationRules
} from './optimizationRules';

export interface RuleTestResult {
    config: OptimizationRuleConfig;
    score: number;
    metrics: DiagramQualityMetrics;
    errors: string[];
}

export interface RuleTestOptions {
    nodes: Node<C4NodeData>[];
    edges: Edge[];
    layoutOptions?: Partial<LayoutOptions>;
    weights?: typeof DEFAULT_QUALITY_WEIGHTS;
    viewportSize?: { width: number; height: number };
}

/**
 * Test a single rule configuration
 */
export async function testRuleConfig(
    rules: OptimizationRuleConfig,
    options: RuleTestOptions
): Promise<RuleTestResult> {
    const errors = validateOptimizationRules(rules);
    
    try {
        const layoutResult = await applySrujaLayout(options.nodes, options.edges, {
            ...options.layoutOptions,
            optimizationRules: rules
        });
        
        const metrics = calculateDiagramQuality(
            layoutResult.nodes,
            layoutResult.edges,
            options.viewportSize || { width: 1920, height: 1080 },
            options.weights || DEFAULT_QUALITY_WEIGHTS
        );
        
        return {
            config: rules,
            score: metrics.weightedScore,
            metrics,
            errors
        };
    } catch (error) {
        return {
            config: rules,
            score: 0,
            metrics: {} as DiagramQualityMetrics,
            errors: [...errors, `Layout failed: ${error}`]
        };
    }
}

/**
 * Test multiple rule presets
 */
export async function testRulePresets(
    options: RuleTestOptions,
    presets: Array<'aggressive' | 'conservative' | 'balanced' | 'minimal'> = ['balanced', 'aggressive', 'conservative']
): Promise<RuleTestResult[]> {
    const results: RuleTestResult[] = [];
    
    for (const preset of presets) {
        const rules = createRulePreset(preset);
        const result = await testRuleConfig(rules, options);
        results.push({
            ...result,
            config: { ...result.config, _preset: preset } as any
        });
    }
    
    return results.sort((a, b) => b.score - a.score);
}

/**
 * Test rule variations by systematically changing parameters
 */
export async function testRuleVariations(
    baseRules: OptimizationRuleConfig,
    options: RuleTestOptions,
    variations: {
        spacing?: { enabled?: boolean; multiplier?: number };
        containment?: { enabled?: boolean; strict?: boolean };
        edgeRouting?: { enabled?: boolean; penaltyMultiplier?: number };
        overlapRemoval?: { enabled?: boolean; iterations?: number[] };
    }
): Promise<RuleTestResult[]> {
    const results: RuleTestResult[] = [];
    const tests: OptimizationRuleConfig[] = [baseRules]; // Always test base
    
    // Generate variations
    if (variations.spacing) {
        if (variations.spacing.multiplier) {
            tests.push(mergeOptimizationRules({
                spacing: {
                    ...baseRules.spacing,
                    softwareSystemSpacing: baseRules.spacing.softwareSystemSpacing * variations.spacing.multiplier,
                    containerSpacing: baseRules.spacing.containerSpacing * variations.spacing.multiplier,
                    componentSpacing: baseRules.spacing.componentSpacing * variations.spacing.multiplier
                }
            }, baseRules));
        }
        if (variations.spacing.enabled !== undefined) {
            tests.push(mergeOptimizationRules({
                spacing: { ...baseRules.spacing, enabled: variations.spacing.enabled }
            }, baseRules));
        }
    }
    
    if (variations.containment) {
        if (variations.containment.strict !== undefined) {
            tests.push(mergeOptimizationRules({
                containment: { ...baseRules.containment, strictEnforcement: variations.containment.strict }
            }, baseRules));
        }
        if (variations.containment.enabled !== undefined) {
            tests.push(mergeOptimizationRules({
                containment: { ...baseRules.containment, enabled: variations.containment.enabled }
            }, baseRules));
        }
    }
    
    if (variations.overlapRemoval?.iterations) {
        for (const iterations of variations.overlapRemoval.iterations) {
            tests.push(mergeOptimizationRules({
                overlapRemoval: { ...baseRules.overlapRemoval, iterations }
            }, baseRules));
        }
    }
    
    // Test all variations
    for (const rules of tests) {
        const result = await testRuleConfig(rules, options);
        results.push(result);
    }
    
    return results.sort((a, b) => b.score - a.score);
}

/**
 * Find best rule configuration by testing common variations
 */
export async function findBestRules(
    options: RuleTestOptions
): Promise<{ best: RuleTestResult; all: RuleTestResult[] }> {
    // Test presets first
    const presetResults = await testRulePresets(options);
    
    // Test parameter variations on the best preset
    const bestPreset = presetResults[0];
    const variations = await testRuleVariations(bestPreset.config, options, {
        spacing: { multiplier: 1.2, enabled: true },
        containment: { strict: true, enabled: true },
        overlapRemoval: { iterations: [3, 5, 7, 10] }
    });
    
    const all = [...presetResults, ...variations];
    const unique = all.filter((result, index, self) => 
        index === self.findIndex(r => 
            JSON.stringify(r.config) === JSON.stringify(result.config)
        )
    );
    
    const best = unique.sort((a, b) => b.score - a.score)[0];
    
    return { best, all: unique };
}

/**
 * Compare two rule configurations
 */
export function compareRuleResults(a: RuleTestResult, b: RuleTestResult): {
    winner: 'a' | 'b' | 'tie';
    differences: Array<{ metric: string; aValue: number; bValue: number; improvement: number }>;
} {
    const winner = a.score > b.score ? 'a' : b.score > a.score ? 'b' : 'tie';
    
    const differences: Array<{ metric: string; aValue: number; bValue: number; improvement: number }> = [];
    
    const metrics = ['overlapScore', 'spacingScore', 'edgeScore', 'hierarchyScore', 
                     'viewportScore', 'consistencyScore', 'aspectRatioScore', 'directionScore',
                     'emptySpaceScore', 'edgeLabelOverlapScore', 'clippedLabelScore'] as const;
    
    metrics.forEach(metric => {
        const aVal = a.metrics[metric] || 0;
        const bVal = b.metrics[metric] || 0;
        if (Math.abs(aVal - bVal) > 1) { // Only show significant differences
            differences.push({
                metric,
                aValue: aVal,
                bValue: bVal,
                improvement: bVal - aVal
            });
        }
    });
    
    return { winner, differences };
}

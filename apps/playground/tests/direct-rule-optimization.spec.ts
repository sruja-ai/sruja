// Direct rule optimization test - uses layout functions directly to test rule configurations
// This bypasses the UI and tests layout engine directly with different rule configs

import { test, expect } from '@playwright/test';
import type { Node, Edge } from '@xyflow/react';
import type { C4NodeData } from '../src/types';
import { applySrujaLayout, type LayoutOptions } from '../src/utils/layoutEngine';
import {
    DEFAULT_OPTIMIZATION_RULES,
    createRulePreset,
    mergeOptimizationRules,
    type OptimizationRuleConfig
} from '../src/utils/optimizationRules';
import { calculateDiagramQuality, DEFAULT_QUALITY_WEIGHTS } from '../src/utils/diagramQuality';
import { getAvailableExamples } from '@sruja/shared';
import { loadExampleFile } from '@sruja/shared';
import { jsonToReactFlow } from '../src/utils/jsonToReactFlow';
import * as fs from 'fs';
import * as path from 'path';

interface DirectOptimizationResult {
    exampleName: string;
    ruleConfig: string;
    config: OptimizationRuleConfig;
    score: number;
    grade: string;
    metrics: any;
    improvements: string[];
}

/**
 * Load and parse a Sruja example file
 */
async function loadExampleData(exampleName: string): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    try {
        const exampleContent = await loadExampleFile(exampleName);
        // Parse using WASM or JSON converter
        // For now, we'll use a simplified approach - in real implementation,
        // you'd use the WASM parser to convert DSL to JSON, then jsonToReactFlow
        // This is a placeholder - actual implementation would need the parser
        throw new Error('Example loading needs WASM parser - use UI-based tests instead');
    } catch (error) {
        // Fallback: return empty data (tests will need to use UI-based approach)
        return { nodes: [], edges: [] };
    }
}

/**
 * Generate test rule configurations
 */
function generateTestConfigs(): Array<{ name: string; config: OptimizationRuleConfig }> {
    return [
        { name: 'default', config: DEFAULT_OPTIMIZATION_RULES },
        { name: 'aggressive', config: createRulePreset('aggressive') },
        { name: 'conservative', config: createRulePreset('conservative') },
        { name: 'balanced', config: createRulePreset('balanced') },
        {
            name: 'high-spacing-strict-containment',
            config: mergeOptimizationRules({
                spacing: {
                    ...DEFAULT_OPTIMIZATION_RULES.spacing,
                    softwareSystemSpacing: 250,
                    containerSpacing: 200,
                    componentSpacing: 180,
                    softwareSystemPadding: 200,
                    containerPadding: 180,
                    componentPadding: 120
                },
                containment: {
                    ...DEFAULT_OPTIMIZATION_RULES.containment,
                    strictEnforcement: true,
                    minParentPadding: 60,
                    paddingMultiplier: 3.5
                }
            })
        },
        {
            name: 'minimal-crossings',
            config: mergeOptimizationRules({
                edgeRouting: {
                    ...DEFAULT_OPTIMIZATION_RULES.edgeRouting,
                    crossingPenalty: 50,
                    bendPenalty: 10,
                    segmentLength: 40
                }
            })
        },
        {
            name: 'aggressive-overlap-removal',
            config: mergeOptimizationRules({
                overlapRemoval: {
                    ...DEFAULT_OPTIMIZATION_RULES.overlapRemoval,
                    iterations: 8,
                    aggressive: true,
                    padding: 40
                }
            })
        }
    ];
}

/**
 * Test rule configuration on a diagram
 */
async function testRuleConfigOnDiagram(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    ruleConfig: OptimizationRuleConfig,
    ruleName: string
): Promise<DirectOptimizationResult> {
    const layoutOptions: LayoutOptions = {
        direction: 'DOWN',
        optimizationRules: ruleConfig
    };
    
    const layoutResult = await applySrujaLayout(nodes, edges, layoutOptions);
    
    const metrics = calculateDiagramQuality(
        layoutResult.nodes,
        layoutResult.edges,
        { width: 1920, height: 1080 },
        DEFAULT_QUALITY_WEIGHTS
    );
    
    const improvements: string[] = [];
    if (metrics.overlappingNodes.length === 0) improvements.push('No overlaps');
    if (metrics.parentChildContainment.length === 0) improvements.push('No containment violations');
    if (metrics.edgeCrossings <= 3) improvements.push(`Low crossings (${metrics.edgeCrossings})`);
    if (metrics.edgesOverNodes === 0) improvements.push('No edges over nodes');
    if (metrics.edgeLabelOverlaps === 0) improvements.push('No label overlaps');
    if (metrics.clippedNodeLabels === 0) improvements.push('No clipped labels');
    
    return {
        exampleName: 'test',
        ruleConfig: ruleName,
        config: ruleConfig,
        score: metrics.weightedScore,
        grade: metrics.grade,
        metrics,
        improvements
    };
}

test.describe('Direct Rule Optimization Tests', () => {
    // Note: These tests require actual diagram data
    // For now, this serves as a template - in practice you'd either:
    // 1. Load example data using WASM parser
    // 2. Use UI-based tests that inject rules via window API
    // 3. Use pre-generated test fixtures
    
    test('compare rule configurations on test diagram', async () => {
        // This is a template test - actual implementation would load real diagram data
        // For now, we'll document the approach
        
        const testConfigs = generateTestConfigs();
        const results: DirectOptimizationResult[] = [];
        
        // TODO: Load actual diagram data
        // const exampleData = await loadExampleData('ecommerce_platform');
        
        // For now, just verify the configs are valid
        testConfigs.forEach(config => {
            expect(config.config).toBeDefined();
            expect(config.config.spacing).toBeDefined();
            expect(config.config.containment).toBeDefined();
        });
        
        console.log(`Generated ${testConfigs.length} test configurations`);
        expect(testConfigs.length).toBeGreaterThan(0);
    });
});



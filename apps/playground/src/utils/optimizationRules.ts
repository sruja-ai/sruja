// Rule-based optimization system for layout engine
// Each rule can be enabled/disabled and fine-tuned with parameters

export interface OptimizationRule {
    id: string;
    name: string;
    description: string;
    enabled: boolean;
    priority: number; // Higher priority = applied first
    category: OptimizationRuleCategory;
    parameters: Record<string, any>;
}

export type OptimizationRuleCategory =
    | 'spacing'
    | 'containment'
    | 'edge-routing'
    | 'overlap-removal'
    | 'label-positioning'
    | 'node-sizing'
    | 'layout-strategy';

export interface OptimizationRuleConfig {
    spacing: SpacingRuleConfig;
    containment: ContainmentRuleConfig;
    edgeRouting: EdgeRoutingRuleConfig;
    overlapRemoval: OverlapRemovalRuleConfig;
    labelPositioning: LabelPositioningRuleConfig;
    nodeSizing: NodeSizingRuleConfig;
    layoutStrategy: LayoutStrategyRuleConfig;
}

export interface SpacingRuleConfig {
    enabled: boolean;
    softwareSystemSpacing: number;
    containerSpacing: number;
    componentSpacing: number;
    softwareSystemPadding: number;
    containerPadding: number;
    componentPadding: number;
    minNodeSpacing: number;
    dynamicSpacing: boolean; // Adjust based on diagram complexity
}

export interface ContainmentRuleConfig {
    enabled: boolean;
    strictEnforcement: boolean; // Fail fast if violation detected
    minParentPadding: number;
    paddingMultiplier: number; // Multiplier for required padding (e.g., 3.0x)
    autoResizeParents: boolean; // Automatically resize parents to fit children
    validationEnabled: boolean; // Post-layout validation
}

export interface EdgeRoutingRuleConfig {
    enabled: boolean;
    algorithm: 'orthogonal' | 'straight' | 'curved';
    bendPenalty: number;
    crossingPenalty: number;
    segmentLength: number;
    labelOffset: number;
    minEdgeLength: number;
    avoidNodes: boolean;
    preferOrthogonal: boolean;
    avoidOverlaps: boolean;
}

export interface OverlapRemovalRuleConfig {
    enabled: boolean;
    iterations: number;
    padding: number;
    aggressive: boolean; // More aggressive overlap removal
    preserveHierarchy: boolean; // Don't break parent-child relationships
}

export interface LabelPositioningRuleConfig {
    enabled: boolean;
    adjustEdgeLabels: boolean; // Adjust edge labels to avoid nodes
    minLabelDistance: number; // Minimum distance from nodes
    maxAdjustment: number; // Maximum distance to move label
    preventClipping: boolean; // Ensure labels aren't clipped
}

export interface NodeSizingRuleConfig {
    enabled: boolean;
    minWidth: number;
    minHeight: number;
    ensureContentFit: boolean; // Size nodes to fit content (labels, icons, etc.)
    contentPadding: number; // Padding for content (labels, icons)
}

export interface LayoutStrategyRuleConfig {
    enabled: boolean;
    preferVertical: boolean; // Prefer vertical layouts
    preferHorizontal: boolean; // Prefer horizontal layouts
    optimizeForViewport: boolean; // Optimize layout to fit viewport
    spaceDistribution: {
        enabled: boolean;
        minThreshold: number;
        targetRatio: number; // Target space utilization (0.0-1.0)
    };
}

/**
 * Default optimization rules configuration
 */
export const DEFAULT_OPTIMIZATION_RULES: OptimizationRuleConfig = {
    spacing: {
        enabled: true,
        softwareSystemSpacing: 200,
        containerSpacing: 160,
        componentSpacing: 160, // Increased from 140
        softwareSystemPadding: 180,
        containerPadding: 160,
        componentPadding: 100,
        minNodeSpacing: 30,
        dynamicSpacing: true
    },
    containment: {
        enabled: true,
        strictEnforcement: true,
        minParentPadding: 50,
        paddingMultiplier: 3.0,
        autoResizeParents: true,
        validationEnabled: true
    },
    edgeRouting: {
        enabled: true,
        algorithm: 'orthogonal',
        bendPenalty: 5,
        crossingPenalty: 30,
        segmentLength: 35,
        labelOffset: 50,
        minEdgeLength: 100,
        avoidNodes: true,
        preferOrthogonal: true,
        avoidOverlaps: true
    },
    overlapRemoval: {
        enabled: true,
        iterations: 10, // Increased from 5
        padding: 30,
        aggressive: false,
        preserveHierarchy: true
    },
    labelPositioning: {
        enabled: true,
        adjustEdgeLabels: true,
        minLabelDistance: 60, // Increased from 50
        maxAdjustment: 300,   // Increased from 200
        preventClipping: true
    },
    nodeSizing: {
        enabled: true,
        minWidth: 180,
        minHeight: 120,
        ensureContentFit: true,
        contentPadding: 24
    },
    layoutStrategy: {
        enabled: true,
        preferVertical: false,
        preferHorizontal: false,
        optimizeForViewport: false,
        spaceDistribution: {
            enabled: true,
            minThreshold: 50,
            targetRatio: 0.7
        }
    }
};

/**
 * Apply optimization rules to layout options
 */
export function applyOptimizationRules(
    baseOptions: any,
    rules: OptimizationRuleConfig = DEFAULT_OPTIMIZATION_RULES
): any {
    const options = { ...baseOptions };

    // Apply spacing rules
    if (rules.spacing.enabled) {
        if (!options.spacing) options.spacing = {};
        if (!options.spacing.node) options.spacing.node = {};
        if (!options.spacing.padding) options.spacing.padding = {};

        options.spacing.node.SoftwareSystem = rules.spacing.softwareSystemSpacing;
        options.spacing.node.Container = rules.spacing.containerSpacing;
        options.spacing.node.Component = rules.spacing.componentSpacing;

        options.spacing.padding.SoftwareSystem = rules.spacing.softwareSystemPadding;
        options.spacing.padding.Container = rules.spacing.containerPadding;
        options.spacing.padding.Component = rules.spacing.componentPadding;
    }

    // Apply edge routing rules
    if (rules.edgeRouting.enabled) {
        if (!options.edgeRouting) options.edgeRouting = {};
        options.edgeRouting.algorithm = rules.edgeRouting.algorithm;
        options.edgeRouting.bendPenalty = rules.edgeRouting.bendPenalty;
        options.edgeRouting.crossingPenalty = rules.edgeRouting.crossingPenalty;
        options.edgeRouting.segmentLength = rules.edgeRouting.segmentLength;
        options.edgeRouting.labelOffset = rules.edgeRouting.labelOffset;
        options.edgeRouting.minEdgeLength = rules.edgeRouting.minEdgeLength;
        options.edgeRouting.avoidNodes = rules.edgeRouting.avoidNodes;
        options.edgeRouting.preferOrthogonal = rules.edgeRouting.preferOrthogonal;
        options.edgeRouting.avoidOverlaps = rules.edgeRouting.avoidOverlaps;
    }

    // Apply optimization settings
    if (!options.optimization) options.optimization = {};

    // Overlap removal
    if (rules.overlapRemoval.enabled) {
        options.optimization.overlapRemoval = {
            iterations: rules.overlapRemoval.iterations,
            padding: rules.overlapRemoval.padding,
            aggressive: rules.overlapRemoval.aggressive,
            preserveHierarchy: rules.overlapRemoval.preserveHierarchy
        };
    } else {
        options.optimization.overlapRemoval = { enabled: false };
    }

    // Space distribution
    if (rules.layoutStrategy.spaceDistribution.enabled) {
        options.optimization.spaceDistribution = {
            enabled: true,
            minThreshold: rules.layoutStrategy.spaceDistribution.minThreshold,
            targetRatio: rules.layoutStrategy.spaceDistribution.targetRatio
        };
    }

    // Edge optimization
    if (rules.edgeRouting.enabled) {
        options.optimization.edgeOptimization = {
            enabled: true,
            minimizeCrossings: true,
            minimizeBends: true,
            preferStraight: rules.edgeRouting.preferOrthogonal
        };
    }

    // Containment optimization
    if (rules.containment.enabled) {
        options.optimization.containmentOptimization = {
            enabled: true,
            minParentPadding: rules.containment.minParentPadding,
            enforceStrict: rules.containment.strictEnforcement,
            autoResize: rules.containment.autoResizeParents
        };
    }

    // Node sizing
    if (rules.nodeSizing.enabled) {
        options.minSize = {
            width: rules.nodeSizing.minWidth,
            height: rules.nodeSizing.minHeight
        };
    }

    return options;
}

/**
 * Merge custom rules with defaults
 */
export function mergeOptimizationRules(
    customRules: Partial<OptimizationRuleConfig>,
    defaults: OptimizationRuleConfig = DEFAULT_OPTIMIZATION_RULES
): OptimizationRuleConfig {
    return {
        spacing: { ...defaults.spacing, ...customRules.spacing },
        containment: { ...defaults.containment, ...customRules.containment },
        edgeRouting: { ...defaults.edgeRouting, ...customRules.edgeRouting },
        overlapRemoval: { ...defaults.overlapRemoval, ...customRules.overlapRemoval },
        labelPositioning: { ...defaults.labelPositioning, ...customRules.labelPositioning },
        nodeSizing: { ...defaults.nodeSizing, ...customRules.nodeSizing },
        layoutStrategy: {
            ...defaults.layoutStrategy,
            ...customRules.layoutStrategy,
            spaceDistribution: {
                ...defaults.layoutStrategy.spaceDistribution,
                ...customRules.layoutStrategy?.spaceDistribution
            }
        }
    };
}

/**
 * Create a rule preset (e.g., "aggressive", "conservative", "balanced")
 */
export function createRulePreset(preset: 'aggressive' | 'conservative' | 'balanced' | 'minimal' | 'label_safe' | 'flow_forward' | 'viewport_fit'): OptimizationRuleConfig {
    const base = DEFAULT_OPTIMIZATION_RULES;

    switch (preset) {
        case 'aggressive':
            return mergeOptimizationRules({
                spacing: {
                    ...base.spacing,
                    softwareSystemSpacing: 250,
                    containerSpacing: 200,
                    componentSpacing: 180
                },
                overlapRemoval: {
                    ...base.overlapRemoval,
                    iterations: 8,
                    aggressive: true
                },
                edgeRouting: {
                    ...base.edgeRouting,
                    crossingPenalty: 50,
                    bendPenalty: 10
                }
            });

        case 'conservative':
            return mergeOptimizationRules({
                spacing: {
                    ...base.spacing,
                    softwareSystemSpacing: 150,
                    containerSpacing: 120,
                    componentSpacing: 100
                },
                overlapRemoval: {
                    ...base.overlapRemoval,
                    iterations: 2,
                    aggressive: false
                },
                edgeRouting: {
                    ...base.edgeRouting,
                    crossingPenalty: 15,
                    bendPenalty: 2
                }
            });

        case 'minimal':
            return mergeOptimizationRules({
                spacing: { ...base.spacing, enabled: false },
                containment: { ...base.containment, enabled: false },
                edgeRouting: { ...base.edgeRouting, enabled: false },
                overlapRemoval: { ...base.overlapRemoval, enabled: false },
                labelPositioning: { ...base.labelPositioning, enabled: false },
                nodeSizing: { ...base.nodeSizing, enabled: false },
                layoutStrategy: { ...base.layoutStrategy, enabled: false }
            });

        case 'balanced':
        default:
            return base;
        case 'label_safe':
            return mergeOptimizationRules({
                labelPositioning: {
                    ...base.labelPositioning,
                    adjustEdgeLabels: true,
                    minLabelDistance: 70,
                    maxAdjustment: 200,
                    preventClipping: true
                },
                edgeRouting: {
                    ...base.edgeRouting,
                    labelOffset: 70,
                    avoidNodes: true,
                    avoidOverlaps: true,
                    preferOrthogonal: true
                },
                nodeSizing: {
                    ...base.nodeSizing,
                    ensureContentFit: true,
                    minWidth: Math.max(base.nodeSizing.minWidth, 200),
                    minHeight: Math.max(base.nodeSizing.minHeight, 130),
                    contentPadding: Math.max(base.nodeSizing.contentPadding, 28)
                }
            });
        case 'flow_forward':
            return mergeOptimizationRules({
                edgeRouting: {
                    ...base.edgeRouting,
                    algorithm: 'orthogonal',
                    avoidNodes: true,
                    preferOrthogonal: true,
                    segmentLength: 40
                },
                layoutStrategy: {
                    ...base.layoutStrategy,
                    preferHorizontal: true,
                    preferVertical: false,
                    optimizeForViewport: true,
                    spaceDistribution: {
                        ...base.layoutStrategy.spaceDistribution,
                        targetRatio: 0.8
                    }
                },
                spacing: {
                    ...base.spacing,
                    dynamicSpacing: true,
                    containerSpacing: Math.max(base.spacing.containerSpacing, 170)
                }
            });
        case 'viewport_fit':
            return mergeOptimizationRules({
                layoutStrategy: {
                    ...base.layoutStrategy,
                    optimizeForViewport: true,
                    spaceDistribution: {
                        enabled: true,
                        minThreshold: 60,
                        targetRatio: 0.85
                    }
                },
                spacing: {
                    ...base.spacing,
                    softwareSystemPadding: Math.min(base.spacing.softwareSystemPadding, 180),
                    containerPadding: Math.min(base.spacing.containerPadding, 160)
                }
            });
    }
}

/**
 * Validate rule configuration
 */
export function validateOptimizationRules(rules: OptimizationRuleConfig): string[] {
    const errors: string[] = [];

    // Validate spacing
    if (rules.spacing.enabled) {
        if (rules.spacing.minNodeSpacing < 0) {
            errors.push('minNodeSpacing must be >= 0');
        }
        if (rules.spacing.softwareSystemSpacing < rules.spacing.minNodeSpacing) {
            errors.push('softwareSystemSpacing must be >= minNodeSpacing');
        }
    }

    // Validate containment
    if (rules.containment.enabled) {
        if (rules.containment.minParentPadding < 0) {
            errors.push('minParentPadding must be >= 0');
        }
        if (rules.containment.paddingMultiplier < 1.0) {
            errors.push('paddingMultiplier must be >= 1.0');
        }
    }

    // Validate edge routing
    if (rules.edgeRouting.enabled) {
        if (rules.edgeRouting.bendPenalty < 0) {
            errors.push('bendPenalty must be >= 0');
        }
        if (rules.edgeRouting.crossingPenalty < 0) {
            errors.push('crossingPenalty must be >= 0');
        }
    }

    // Validate overlap removal
    if (rules.overlapRemoval.enabled) {
        if (rules.overlapRemoval.iterations < 0) {
            errors.push('overlapRemoval.iterations must be >= 0');
        }
        if (rules.overlapRemoval.padding < 0) {
            errors.push('overlapRemoval.padding must be >= 0');
        }
    }

    return errors;
}

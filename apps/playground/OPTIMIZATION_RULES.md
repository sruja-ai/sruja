# Rule-Based Optimization System

This document describes the rule-based optimization system for the Sruja layout engine.

## Overview

The optimization system uses configurable rules that can be enabled/disabled and fine-tuned individually. This allows systematic testing and optimization of layout quality.

## Rule Categories

### 1. Spacing Rules
Controls spacing between nodes and padding inside containers.

**Parameters:**
- `softwareSystemSpacing`: Spacing between SoftwareSystem nodes (default: 200px)
- `containerSpacing`: Spacing between Container nodes (default: 160px)
- `componentSpacing`: Spacing between Component nodes (default: 140px)
- `softwareSystemPadding`: Padding inside SoftwareSystem containers (default: 180px)
- `containerPadding`: Padding inside Container nodes (default: 160px)
- `componentPadding`: Padding inside Component nodes (default: 100px)
- `minNodeSpacing`: Minimum spacing between any nodes (default: 30px)
- `dynamicSpacing`: Adjust spacing based on diagram complexity (default: true)

### 2. Containment Rules
Ensures children are properly contained within parent nodes.

**Parameters:**
- `strictEnforcement`: Fail fast if violations detected (default: true)
- `minParentPadding`: Minimum padding inside parent nodes (default: 50px)
- `paddingMultiplier`: Multiplier for required padding (default: 3.0)
- `autoResizeParents`: Automatically resize parents to fit children (default: true)
- `validationEnabled`: Post-layout validation (default: true)

### 3. Edge Routing Rules
Controls how edges are routed between nodes.

**Parameters:**
- `algorithm`: 'orthogonal' | 'straight' | 'curved' (default: 'orthogonal')
- `bendPenalty`: Penalty for edge bends (default: 5)
- `crossingPenalty`: Penalty for edge crossings (default: 30)
- `segmentLength`: Preferred edge segment length (default: 35px)
- `labelOffset`: Distance of labels from nodes (default: 50px)
- `minEdgeLength`: Minimum edge length (default: 100px)
- `avoidNodes`: Route edges away from nodes (default: true)
- `preferOrthogonal`: Prefer orthogonal routing (default: true)
- `avoidOverlaps`: Avoid overlapping edges with nodes (default: true)

### 4. Overlap Removal Rules
Removes node overlaps during layout.

**Parameters:**
- `iterations`: Number of overlap removal iterations (default: 5)
- `padding`: Padding added around nodes (default: 30px)
- `aggressive`: More aggressive overlap removal (default: false)
- `preserveHierarchy`: Don't break parent-child relationships (default: true)

### 5. Label Positioning Rules
Optimizes edge and node label positions.

**Parameters:**
- `adjustEdgeLabels`: Adjust edge labels to avoid nodes (default: true)
- `minLabelDistance`: Minimum distance from nodes (default: 50px)
- `maxAdjustment`: Maximum distance to move label (default: 150px)
- `preventClipping`: Ensure labels aren't clipped (default: true)

### 6. Node Sizing Rules
Controls minimum node sizes and content fitting.

**Parameters:**
- `minWidth`: Minimum node width (default: 180px)
- `minHeight`: Minimum node height (default: 120px)
- `ensureContentFit`: Size nodes to fit content (default: true)
- `contentPadding`: Padding for content (default: 24px)

### 7. Layout Strategy Rules
Overall layout strategy preferences.

**Parameters:**
- `preferVertical`: Prefer vertical layouts (default: false)
- `preferHorizontal`: Prefer horizontal layouts (default: false)
- `optimizeForViewport`: Optimize for viewport fit (default: false)
- `spaceDistribution`: Space distribution settings
  - `enabled`: Enable space distribution (default: true)
  - `minThreshold`: Minimum threshold (default: 50px)
  - `targetRatio`: Target space utilization 0.0-1.0 (default: 0.7)

## Usage

### Default Configuration
```typescript
import { DEFAULT_OPTIMIZATION_RULES } from './utils/optimizationRules';
import { applySrujaLayout } from './utils/layoutEngine';

const result = await applySrujaLayout(nodes, edges, {
    optimizationRules: DEFAULT_OPTIMIZATION_RULES
});
```

### Using Presets
```typescript
import { createRulePreset } from './utils/optimizationRules';

// Aggressive optimization
const aggressive = createRulePreset('aggressive');

// Conservative optimization
const conservative = createRulePreset('conservative');

// Balanced (default)
const balanced = createRulePreset('balanced');
```

### Custom Configuration
```typescript
import { mergeOptimizationRules, DEFAULT_OPTIMIZATION_RULES } from './utils/optimizationRules';

const customRules = mergeOptimizationRules({
    spacing: {
        softwareSystemSpacing: 250, // Increase spacing
    },
    containment: {
        strictEnforcement: true, // Enable strict mode
        minParentPadding: 60
    },
    overlapRemoval: {
        iterations: 8 // More iterations
    }
}, DEFAULT_OPTIMIZATION_RULES);
```

## Testing with Playwright

Tests can inject optimization rules via window API:

```typescript
// In Playwright test
await page.evaluate((rules) => {
    (window as any).__TEST_OPTIMIZATION_RULES__ = rules;
}, customRules);

// Load example and wait for layout
await loadExample(page, 'example-name');
await waitForLayoutStable(page);
```

## Best Practices

1. **Start with defaults**: Use `DEFAULT_OPTIMIZATION_RULES` as baseline
2. **Test systematically**: Use Playwright tests to compare configurations
3. **Iterate**: Make small parameter adjustments and test impact
4. **Document changes**: Record what works for different diagram types
5. **Balance trade-offs**: Higher spacing improves quality but may reduce density

## Example Test Results

Run the optimization tests to find best configurations:

```bash
npm run test:e2e -- rule-based-optimization.spec.ts
```

Results are saved to `tests/results/` with:
- JSON data with all test results
- Markdown reports with recommendations
- Best configurations per example type



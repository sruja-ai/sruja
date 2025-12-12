# Rules-Based Layout System

## Overview

The layout system now uses a **rules-based approach** instead of if-else conditions. This makes it easier to maintain, test, and improve layout quality across all examples.

## Architecture

### Layout Rules Engine (`layoutRules.ts`)

Rules are evaluated in priority order. Each rule has:
- **Condition**: When to apply this rule
- **Action**: What layout configuration to use
- **Priority**: Higher priority rules are evaluated first

### Rule Evaluation

```typescript
// Rules are evaluated in priority order
// First matching rule wins
const config = selectLayoutConfig(nodes, edges, level, ...);
// Returns: { engine: 'sruja' | 'c4level', direction: 'DOWN' | 'RIGHT' | ..., options: {...} }
```

## Default Rules (Priority Order)

1. **Simple C4 Layout** (Priority: 100)
   - Condition: Simple diagrams, no hierarchy, no expanded nodes
   - Action: Use `c4level` engine

2. **Hierarchical Sruja Layout** (Priority: 90)
   - Condition: Has hierarchy AND expanded nodes
   - Action: Use `sruja` engine

3. **L0/L1 C4 Layout** (Priority: 80)
   - Condition: L0 or L1 level, no expanded nodes
   - Action: Use `c4level` engine

4. **L2 Container Layout** (Priority: 75)
   - Condition: L2 level
   - Action: Use `sruja` engine

5. **L3 Component Layout** (Priority: 70)
   - Condition: L3 level
   - Action: Use `sruja` engine

6. **Complex Sruja Layout** (Priority: 60)
   - Condition: Complex diagrams with many edges
   - Action: Use `sruja` engine

7. **Wide Diagram Vertical** (Priority: 50)
   - Condition: Wide nodes, many nodes
   - Action: Use `sruja` with DOWN direction

8. **Tall Diagram Horizontal** (Priority: 45)
   - Condition: Tall nodes, many nodes
   - Action: Use `sruja` with RIGHT direction

9. **Default Sruja** (Priority: 10)
   - Condition: Always matches (fallback)
   - Action: Use `sruja` engine

## Layout Engines

### Sruja Layout (`sruja`)
- Best for: Hierarchical diagrams, expanded nodes, complex layouts
- Handles: Parent-child relationships, compound nodes
- Features: Custom C4 layout algorithm

### C4 Level Layout (`c4level`)
- Best for: Flat diagrams, system context views
- Handles: Level-specific layouts (L0, L1, L2, L3)
- Features: C4 model conventions

## Context Analysis

The rules engine analyzes:
- **Node count**: Simple (<10), Medium (<30), Complex (30+)
- **Edge count**: Number of relationships
- **Hierarchy**: Has parent-child relationships?
- **Expanded nodes**: Are any nodes expanded?
- **Average node size**: Width/height ratio
- **Current level**: L0, L1, L2, or L3

## Benefits

1. **No if-else chains**: Rules are declarative
2. **Easy to test**: Each rule can be tested independently
3. **Easy to extend**: Add new rules without modifying existing code
4. **Predictable**: Rules are evaluated in priority order
5. **Maintainable**: Rules are self-documenting

## Adding New Rules

```typescript
import { createLayoutRule } from './layoutRules';

const myRule = createLayoutRule(
    'my-rule-id',
    'My Custom Rule',
    85, // Priority
    (context) => {
        // Condition: when to apply
        return context.nodeCount > 50 && context.hasHierarchy;
    },
    (context) => {
        // Action: what config to return
        return {
            engine: 'sruja',
            direction: 'DOWN',
            options: {}
        };
    }
);

// Use in selectLayoutConfig with custom rules
const rules = [...DEFAULT_LAYOUT_RULES, myRule];
const config = selectLayoutConfig(nodes, edges, level, ..., rules);
```

## Testing

Playwright tests verify:
- All examples produce good quality diagrams
- Rules select appropriate layouts
- Quality scores meet thresholds

Run tests:
```bash
npm run test:e2e
```

## Future Improvements

- [ ] Machine learning to optimize rule priorities
- [ ] Rule performance metrics
- [ ] Automatic rule generation from test results
- [ ] Rule conflict detection

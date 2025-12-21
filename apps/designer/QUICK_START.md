# Quick Start - Iterative Quality Improvement

## Prerequisites

1. **Dev server running** on `http://localhost:4321`
   ```bash
   cd apps/designer
   npm run dev
   ```

2. **Playwright installed** (if not already)
   ```bash
   npx playwright install
   ```

## Run First Iteration

```bash
cd apps/designer
npm run improve:iterative
```

This will:
1. Run the e2e test to measure current quality
2. Extract metrics (grade, score, violations)
3. Identify issues in priority order
4. Generate improvement suggestions
5. Save results to `tests/results/`

## What to Expect

### Output Example

```
🚀 Starting Iteration 1
============================================================

🔍 Running e2e test to measure quality...

=== ECOMMERCE PLATFORM QUALITY METRICS ===
Grade: F
Weighted Score: 45.2
Overall Score: 42.1
Edge Crossings: 112
Overlapping Nodes: 3
Containment Violations: 5
Spacing Violations: 8
==========================================

============================================================
ITERATION 1 - QUALITY REPORT
============================================================
Grade: F
Weighted Score: 45.2
Overall Score: 42.1

--- METRICS ---
Edge Crossings: 112
Overlapping Nodes: 3
Containment Violations: 5
Spacing Violations: 8

--- IDENTIFIED ISSUES ---
  • CRITICAL: Grade is F - must fix critical violations first
  • CRITICAL: 5 containment violations (causes F grade, penalty: 30 per violation)
  • HIGH: 3 node overlaps (penalty: 25 per overlap, weight: 0.25)
  • MEDIUM: 112 edge crossings (target: 11, weight: 0.18)

--- SUGGESTED IMPROVEMENTS ---
1. Fix containment violations in packages/layout/src/phases/optimization.ts:
   - Ensure applyContainmentEnforcement runs after every optimization phase
   - Increase padding in containment checks (currently 50px)
   - Add validation in layout phase to prevent containment-breaking operations
...
```

## Next Steps

1. **Review the issues** - Focus on CRITICAL ones first
2. **Fix the code** - Follow the suggestions in the output
3. **Re-run** - `npm run improve:iterative` again
4. **Verify improvement** - Check if score increased
5. **Repeat** - Continue until Grade B or better

## Files to Modify

Based on the issues found, you'll typically modify:

- `packages/layout/src/phases/optimization.ts` - Containment, overlaps, crossings
- `packages/layout/src/phases/layout.ts` - Initial layout algorithm
- `packages/layout/src/phases/sizing.ts` - Node sizing for labels
- `packages/layout/src/phases/edge-routing.ts` - Edge routing

## Tips

- **Fix one issue type at a time** for easier verification
- **Containment violations are highest priority** (they cause F grade)
- **Check iteration history** in `tests/results/improvement-iterations.json`
- **Use debug mode**: `LAYOUT_DEBUG=true npm run improve:iterative`

## Troubleshooting

### "E2E test failed"
- Ensure dev server is running: `npm run dev`
- Check server is accessible: Open `http://localhost:4321` in browser
- Verify Playwright: `npx playwright install`

### "Failed to extract layout metrics"
- Wait longer for layout (increase timeout in test)
- Check browser console for errors
- Verify diagram loaded correctly

### Score not improving
- Check that changes are actually applied
- Verify optimization phases are enabled
- Review iteration history to see what changed

## Success Criteria

Target: **Grade B (Score >= 80)**

Key metrics:
- ✅ 0 containment violations
- ✅ 0 node overlaps
- ✅ < 20 edge crossings
- ✅ < 5 spacing violations

Good luck! 🚀

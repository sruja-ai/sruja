# Round 2 Results Analysis

**Date:** 2026-01-02  
**Test Suite:** Complex Examples (8 examples × 3 levels = 24 tests)

## Three-Way Comparison

### Edge Crossings

| Level  | Baseline | Round 1 | Round 2   | Change (B→R2)     | Change (R1→R2)    |
| ------ | -------- | ------- | --------- | ----------------- | ----------------- |
| **L1** | 0.125    | 0       | 0         | ✅ Improved       | ✅ Maintained     |
| **L2** | 43.75    | 57.13   | **42**    | ✅ **4% better**  | ✅ **26% better** |
| **L3** | 39.13    | 14.88   | **20.13** | ✅ **49% better** | ⚠️ **35% worse**  |

### Spacing Violations

| Level  | Baseline | Round 1 | Round 2 | Change (B→R2) | Change (R1→R2) |
| ------ | -------- | ------- | ------- | ------------- | -------------- |
| **L1** | 2.14     | 3.11    | 2.91    | ⚠️ Worse      | ✅ Improved    |
| **L2** | 4.26     | 4.43    | 4.54    | ⚠️ Worse      | ⚠️ Worse       |
| **L3** | 4.58     | 5.19    | 4.95    | ⚠️ Worse      | ✅ Improved    |

## Key Findings

### ✅ L2 Recovery - SUCCESS

- **Round 1:** 57.13 crossings (30% worse than baseline)
- **Round 2:** 42 crossings (4% better than baseline)
- **Improvement:** 26% reduction from Round 1
- **Status:** ✅ Recovered and slightly improved

### ⚠️ L3 Partial Regression

- **Round 1:** 14.88 crossings (62% better than baseline) - Best result
- **Round 2:** 20.13 crossings (49% better than baseline)
- **Change:** 35% increase from Round 1, but still 49% better than baseline
- **Status:** ⚠️ Lost some gains but still significantly better than baseline

### 📊 Overall Assessment

**Round 2 Strategy:**

- ✅ **L2:** Successfully recovered from Round 1 regression
- ⚠️ **L3:** Lost some gains but still much better than baseline
- ✅ **Differentiated approach:** Working better than uniform strategy

## Individual Example Analysis

### L2 Examples (Sorted by Edge Crossings)

| Example                    | Round 1 | Round 2 | Change            |
| -------------------------- | ------- | ------- | ----------------- |
| sruja_architecture_v2      | 2       | 1       | ✅ Better         |
| pattern_rag_pipeline       | 5       | 3       | ✅ Better         |
| project_ecommerce          | 2       | 3       | ⚠️ Worse          |
| demo_implied_relationships | 32      | 15      | ✅ **53% better** |
| project_saas_platform      | 78      | 23      | ✅ **71% better** |
| pattern_microservices      | 26      | 30      | ⚠️ Worse          |
| project_iot_platform       | 44      | 54      | ⚠️ Worse          |
| pattern_agentic_ai         | 268     | 207     | ✅ **23% better** |

**L2 Summary:**

- 5 examples improved
- 3 examples worse
- Average: 26% improvement from Round 1

### L3 Examples (Sorted by Edge Crossings)

| Example                    | Round 1 | Round 2 | Change           |
| -------------------------- | ------- | ------- | ---------------- |
| pattern_rag_pipeline       | 2       | 2       | ✅ Maintained    |
| project_ecommerce          | 4       | 3       | ✅ Better        |
| project_iot_platform       | 1       | 3       | ⚠️ Worse         |
| sruja_architecture_v2      | 1       | 3       | ⚠️ Worse         |
| demo_implied_relationships | 4       | 7       | ⚠️ Worse         |
| project_saas_platform      | 6       | 11      | ⚠️ Worse         |
| pattern_microservices      | 13      | 12      | ✅ Better        |
| pattern_agentic_ai         | 88      | 120     | ⚠️ **36% worse** |

**L3 Summary:**

- 3 examples improved/maintained
- 5 examples worse
- Average: 35% increase from Round 1

## Extreme Outliers

### pattern_agentic_ai.sruja

- **L2:** 268 → 207 crossings (23% improvement, but still extremely high)
- **L3:** 88 → 120 crossings (36% worse)
- **Status:** Still needs special handling

## Score Analysis

### Why Scores Are Still 0%

**L2 Example:**

- 42 crossings × 0.08 = 3.36 → capped at -0.6
- Spacing: -0.25
- Rank alignment: -0.3
- **Total:** -1.15 → score = 0

**L3 Example:**

- 20.13 crossings × 0.08 = 1.61 → capped at -0.6
- Spacing: -0.25
- Rank alignment: -0.3
- **Total:** -1.15 → score = 0

**To break above 0%, need:**

- Edge crossings < 7 (to avoid -0.6 penalty)
- OR spacing consistency > 0.85
- OR rank alignment > 0.95

## Recommendations

### Immediate Actions

1. **Hybrid Strategy**
   - Keep Round 2 approach for L2 (working well)
   - Consider reverting L3 to Round 1 spacing (had better results)
   - Or find middle ground for L3

2. **Extreme Outlier Handling**
   - `pattern_agentic_ai.sruja` needs special treatment
   - Consider different layout algorithm for 200+ crossings
   - Maybe use `rankdir=LR` or force simpler layout

3. **Fine-tune L3**
   - L3 benefited from aggressive spacing in Round 1
   - Consider keeping higher spacing for L3 (1.15-1.20x)
   - Keep minlen improvements

### Next Steps

1. **Test Hybrid Approach**
   - L2: Keep Round 2 strategy (polyline + moderate spacing)
   - L3: Revert to Round 1 spacing (1.20-1.25x) but keep minlen

2. **Address Extreme Cases**
   - Special handling for diagrams with 100+ crossings
   - Consider different splines or layout direction
   - Maybe use simpler layout for extreme cases

3. **Score Threshold Review**
   - Current scoring is too harsh (any >7 crossings = 0%)
   - Consider adjusting penalty weights
   - Or increase threshold for "acceptable" crossings

## Conclusion

### Round 2 Assessment

✅ **L2 Recovery:** Successfully recovered from Round 1 regression  
⚠️ **L3 Trade-off:** Lost some gains but still 49% better than baseline  
✅ **Differentiated Strategy:** Better than uniform approach  
⚠️ **Scores Still 0%:** Need further improvements or score adjustment

### Best Strategy So Far

**For L2:**

- ✅ Round 2 approach (polyline + moderate spacing)
- Edge crossings: 42 (4% better than baseline)

**For L3:**

- ⚠️ Round 1 had better results (14.88 crossings)
- Round 2 still good (20.13 crossings, 49% better than baseline)
- Consider hybrid: Round 1 spacing + Round 2 minlen

### Next Round Suggestions

1. **L2:** Keep current approach (working well)
2. **L3:** Increase spacing back to Round 1 levels (1.20-1.25x)
3. **Extreme Cases:** Add special handling for 100+ crossings
4. **Scoring:** Consider adjusting penalty weights to allow non-zero scores

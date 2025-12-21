// View rule normalization for LikeC4 format

/**
 * Normalizes view rules from model dump format to LikeC4 parsed format.
 * 
 * @param rule - Rule object from model dump
 * @returns Normalized rule object compatible with LikeC4
 * 
 * @remarks
 * Handles various input formats and converts them to LikeC4's expected structure:
 * - String values (wildcards, element IDs)
 * - Object formats (wildcard objects, element arrays)
 * - Already normalized arrays
 */
export function normalizeViewRule(rule: any): any {
  const normalizedRule: any = { ...rule };

  // Handle include rules
  if (rule.include !== undefined && rule.include !== null) {
    normalizedRule.include = normalizeRuleExpression(rule.include, "include");
  }

  // Handle exclude rules
  if (rule.exclude !== undefined && rule.exclude !== null) {
    normalizedRule.exclude = normalizeRuleExpression(rule.exclude, "exclude");
  }

  return normalizedRule;
}

/**
 * Normalizes a single rule expression (include/exclude).
 * 
 * @param expr - The expression to normalize
 * @param ruleType - Whether this is an "include" or "exclude" rule
 * @returns Normalized array of rule expressions
 */
function normalizeRuleExpression(expr: any, ruleType: "include" | "exclude"): any[] {
  if (typeof expr === 'string') {
    if (expr === '*') {
      return [{ wildcard: true }];
    } else if (expr === 'include' || expr === 'exclude') {
      console.warn(`[LikeC4Canvas] Rule has "${expr}" as string - it's a keyword, not an element ID. ${ruleType === 'exclude' ? 'Skipping exclude rule.' : 'Using wildcard as fallback.'}`);
      return ruleType === 'exclude' ? [] : [{ wildcard: true }];
    } else {
      return [{ ref: { model: expr } }];
    }
  } else if (expr && typeof expr === 'object') {
    if (expr.wildcard === true) {
      return [{ wildcard: true }];
    } else if (expr.elements && Array.isArray(expr.elements)) {
      const normalized = expr.elements
        .filter((el: string) => {
          if (el === 'include' || el === 'exclude') {
            console.warn(`[LikeC4Canvas] Filtering out invalid element ID "${el}" - it's a keyword, not an element ID`);
            return false;
          }
          return true;
        })
        .map((el: string) => ({ ref: { model: el } }));

      return normalized.length > 0 ? normalized : (ruleType === 'exclude' ? [] : [{ wildcard: true }]);
    } else if (Array.isArray(expr)) {
      const normalized = expr
        .map((e: any) => {
          if (typeof e === 'string') {
            if (e === '*') {
              return { wildcard: true };
            } else if (e === 'include' || e === 'exclude') {
              console.warn(`[LikeC4Canvas] Skipping invalid expression in ${ruleType} array: "${e}" is a keyword, not an element ID`);
              return null;
            } else {
              return { ref: { model: e } };
            }
          }
          if (e && typeof e === 'object' && e.wildcard === true) {
            return { wildcard: true };
          }
          if (e && typeof e === 'object' && e.ref) {
            const elementId = e.ref.model || e.ref;
            if (elementId === 'include' || elementId === 'exclude') {
              console.warn(`[LikeC4Canvas] Skipping invalid expression: references "${elementId}" which is a keyword, not an element ID`);
              return null;
            }
          }
          return e;
        })
        .filter((e: any) => e !== null);

      return normalized.length > 0 ? normalized : (ruleType === 'exclude' ? [] : [{ wildcard: true }]);
    } else if (expr.ref) {
      return [expr];
    }
  }

  console.warn(`[LikeC4Canvas] Unknown ${ruleType} rule format:`, expr);
  return ruleType === 'exclude' ? [] : [{ wildcard: true }];
}


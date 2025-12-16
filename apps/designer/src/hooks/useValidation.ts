/**
 * useValidation hook
 * Real-time architecture validation with memoization
 */

import { useMemo } from "react";
import { useArchitectureStore } from "../stores/architectureStore";
import {
  validateArchitecture,
  getIssuesForElement,
  getIssuesByCategory,
  type ValidationResult,
  type ValidationIssue,
} from "../utils/architectureValidator";

export interface UseValidationReturn {
  /** Full validation result */
  result: ValidationResult;
  /** Is the architecture valid (no errors)? */
  isValid: boolean;
  /** Validation score 0-100 */
  score: number;
  /** All issues */
  issues: ValidationIssue[];
  /** Error count */
  errorCount: number;
  /** Warning count */
  warningCount: number;
  /** Get issues for a specific element */
  getElementIssues: (elementPath: string) => ValidationIssue[];
  /** Get issues by category */
  getCategoryIssues: (category: ValidationIssue["category"]) => ValidationIssue[];
  /** Check if element has issues */
  hasIssues: (elementPath: string) => boolean;
  /** Check if element has errors */
  hasErrors: (elementPath: string) => boolean;
}

/**
 * Hook for real-time architecture validation
 */
export function useValidation(): UseValidationReturn {
  const data = useArchitectureStore((s) => s.data);

  const result = useMemo(() => validateArchitecture(data), [data]);

  const getElementIssues = useMemo(
    () => (elementPath: string) => getIssuesForElement(result, elementPath),
    [result]
  );

  const getCategoryIssues = useMemo(
    () => (category: ValidationIssue["category"]) => getIssuesByCategory(result, category),
    [result]
  );

  const hasIssues = useMemo(
    () => (elementPath: string) => result.issues.some((i) => i.elementId === elementPath),
    [result]
  );

  const hasErrors = useMemo(
    () => (elementPath: string) =>
      result.issues.some((i) => i.elementId === elementPath && i.severity === "error"),
    [result]
  );

  return {
    result,
    isValid: result.isValid,
    score: result.score,
    issues: result.issues,
    errorCount: result.summary.errors,
    warningCount: result.summary.warnings,
    getElementIssues,
    getCategoryIssues,
    hasIssues,
    hasErrors,
  };
}

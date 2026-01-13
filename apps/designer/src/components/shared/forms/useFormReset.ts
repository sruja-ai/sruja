// apps/designer/src/components/shared/forms/useFormReset.ts
// Hook for resetting form values when form opens or element changes

import { useEffect } from "react";
import type { UseFormStateReturn } from "./useFormState";

/**
 * Hook that resets form values when the form opens or dependencies change.
 *
 * This eliminates the duplicated useEffect pattern found across all Edit*Form components.
 *
 * @param form - Form state object from useFormState
 * @param isOpen - Whether the form is currently open
 * @param initialValues - Initial values to set when form opens
 * @param dependencies - Additional dependencies to watch for changes
 *
 * @example
 * ```tsx
 * const form = useFormState({ ... });
 *
 * useFormReset(
 *   form,
 *   isOpen,
 *   {
 *     name: element?.title || initialName || "",
 *     description: extractDescription(element),
 *     // ... other fields
 *   },
 *   [element, initialName]
 * );
 * ```
 */
export function useFormReset<T extends object>(
  form: UseFormStateReturn<T>,
  isOpen: boolean,
  initialValues: Partial<T>,
  dependencies: unknown[] = []
): void {
  useEffect(() => {
    if (isOpen) {
      form.setValues(initialValues);
      form.clearErrors();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen, ...dependencies]);
}

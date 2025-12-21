// apps/designer/src/components/shared/forms/useFormState.ts
// Custom hook for form state management with validation

import { useState, useCallback } from "react";

export interface FormErrors {
  [key: string]: string;
}

export interface UseFormStateOptions<T> {
  initialValues: T;
  validate?: (values: T) => FormErrors;
  onSubmit: (values: T) => Promise<void> | void;
}

export interface UseFormStateReturn<T> {
  values: T;
  errors: FormErrors;
  isSubmitting: boolean;
  setValue: <K extends keyof T>(key: K, value: T[K]) => void;
  setValues: (values: Partial<T>) => void;
  setError: (key: keyof T, message: string) => void;
  clearError: (key: keyof T) => void;
  clearErrors: () => void;
  handleSubmit: (e: React.FormEvent<HTMLFormElement>) => Promise<void>;
  reset: () => void;
}

/**
 * Custom hook for managing form state with validation
 * Provides a simpler alternative to React Hook Form for our use case
 * 
 * @example
 * ```tsx
 * const form = useFormState({
 *   initialValues: { name: "", description: "" },
 *   validate: (values) => {
 *     const errors: FormErrors = {};
 *     if (!values.name.trim()) errors.name = "Name is required";
 *     return errors;
 *   },
 *   onSubmit: async (values) => {
 *     await saveData(values);
 *   }
 * });
 * 
 * <form onSubmit={form.handleSubmit}>
 *   <FormField
 *     label="Name"
 *     name="name"
 *     value={form.values.name}
 *     onChange={(value) => form.setValue("name", value)}
 *     error={form.errors.name}
 *   />
 * </form>
 * ```
 */
export function useFormState<T extends object>({
  initialValues,
  validate,
  onSubmit,
}: UseFormStateOptions<T>): UseFormStateReturn<T> {
  const [values, setValuesState] = useState<T>(initialValues);
  const [errors, setErrors] = useState<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  const setValue = useCallback(<K extends keyof T>(key: K, value: T[K]) => {
    setValuesState((prev) => ({ ...prev, [key]: value }));
    // Clear error for this field when user starts typing
    // Also clear submit error when any field changes
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[key as string];
      delete newErrors.submit; // Clear submit error on any field change
      return newErrors;
    });
  }, []);

  const setValues = useCallback((newValues: Partial<T>) => {
    setValuesState((prev) => ({ ...prev, ...newValues }));
  }, []);

  const setError = useCallback((key: keyof T, message: string) => {
    setErrors((prev) => ({ ...prev, [key as string]: message }));
  }, []);

  const clearError = useCallback((key: keyof T) => {
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[key as string];
      return newErrors;
    });
  }, []);

  const clearErrors = useCallback(() => {
    setErrors({});
  }, []);

  const handleSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault();

      // Run validation if provided
      if (validate) {
        const validationErrors = validate(values);
        if (Object.keys(validationErrors).length > 0) {
          setErrors(validationErrors);
          return;
        }
      }

      setIsSubmitting(true);
      try {
        await onSubmit(values);
      } catch (error) {
        console.error("Form submission error:", error);
        setError("submit" as keyof T, "Failed to submit. Please try again.");
      } finally {
        setIsSubmitting(false);
      }
    },
    [values, validate, onSubmit, setError]
  );

  const reset = useCallback(() => {
    setValuesState(initialValues);
    setErrors({});
    setIsSubmitting(false);
  }, [initialValues]);

  return {
    values,
    errors,
    isSubmitting,
    setValue,
    setValues,
    setError,
    clearError,
    clearErrors,
    handleSubmit,
    reset,
  };
}

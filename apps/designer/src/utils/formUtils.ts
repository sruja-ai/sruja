// apps/designer/src/utils/formUtils.ts
// Shared utilities for form handling

import type { FormSubmitHandler, FormDataExtractor, FormValidator } from "../types/formHandlers";

/**
 * Creates a type-safe form submit handler
 * 
 * @param handler - Function to handle form submission
 * @param extractor - Optional function to extract data from form
 * @param validator - Optional validator function
 * @returns Typed form submit handler
 * 
 * @example
 * ```tsx
 * const handleSubmit = createFormSubmitHandler(
 *   async (formData) => {
 *     // Handle submission with typed form data
 *   }
 * );
 * 
 * <form onSubmit={handleSubmit}>
 *   ...
 * </form>
 * ```
 */
export function createFormSubmitHandler<T = FormData>(
  handler: (data: T) => void | Promise<void>,
  extractor?: FormDataExtractor<T>,
  validator?: FormValidator<T>
): FormSubmitHandler {
  return async (e) => {
    e.preventDefault();
    
    try {
      const form = e.currentTarget;
      const data = extractor 
        ? extractor(form)
        : (new FormData(form) as unknown as T);
      
      if (validator) {
        const validation = validator(data);
        if (!validation.isValid) {
          // Handle validation errors
          console.error("Form validation failed:", validation.errors);
          return;
        }
      }
      
      await handler(data);
    } catch (error) {
      console.error("Form submission error:", error);
      throw error;
    }
  };
}

/**
 * Extracts form data as a plain object
 * 
 * @param form - HTML form element
 * @returns Object with form field values
 */
export function extractFormData(form: HTMLFormElement): Record<string, string> {
  const formData = new FormData(form);
  const data: Record<string, string> = {};
  
  for (const [key, value] of formData.entries()) {
    data[key] = String(value);
  }
  
  return data;
}

/**
 * Creates a simple form submit handler that prevents default
 * Useful for forms that handle submission via other means
 */
export function createPreventDefaultHandler(): FormSubmitHandler {
  return (e) => {
    e.preventDefault();
  };
}

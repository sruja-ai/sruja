// apps/designer/src/components/shared/forms/useElementFormSubmit.ts
// Shared hook for element form submission logic to reduce duplication

import { useState, useCallback } from "react";
import { useArchitectureStore } from "../../../stores";
import type { SrujaModelDump, ElementDump } from "@sruja/shared";
import { generateUniqueId } from "./utils";

interface UseElementFormSubmitOptions {
  element?: ElementDump;
  kind: string;
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

interface UseElementFormSubmitReturn {
  handleSubmit: (values: {
    name: string;
    description?: string;
    technology?: string;
    customId?: boolean;
    idInput?: string;
    parentId?: string;
    tags?: string[];
    [key: string]: unknown;
  }) => Promise<void>;
  isSubmitting: boolean;
  error: Error | null;
}

/**
 * Generate a unique ID for a new element, handling custom IDs and hierarchical IDs.
 */
function generateElementId(
  values: {
    customId?: boolean;
    idInput?: string;
    parentId?: string;
    name: string;
  },
  element: ElementDump | undefined,
  model: SrujaModelDump,
  kind: string
): string {
  if (element) {
    return element.id; // Edit mode - use existing ID
  }

  // Create Mode
  let targetId: string;
  if (values.customId && values.idInput?.trim()) {
    targetId = values.idInput.trim();
  } else {
    targetId = generateUniqueId(values.name, model, kind);
  }

  // Handle hierarchical IDs (e.g., system.container)
  if (values.parentId) {
    targetId = `${values.parentId}.${targetId}`;
  }

  // Ensure uniqueness
  const existingIds = new Set(Object.keys(model.elements || {}));
  let i = 1;
  const originalId = targetId;
  while (existingIds.has(targetId)) {
    const baseId = values.parentId ? targetId.split(".").pop() || originalId : originalId;
    targetId = values.parentId ? `${values.parentId}.${baseId}-${i++}` : `${originalId}-${i++}`;
  }

  return targetId;
}

/**
 * Create element data object from form values.
 */
function createElementData(
  targetId: string,
  values: {
    name: string;
    description?: string;
    technology?: string;
    parentId?: string;
    tags?: string[];
  },
  element: ElementDump | undefined,
  kind: string
): ElementDump {
  return {
    id: targetId,
    kind: kind as ElementDump["kind"],
    title: values.name,
    description: values.description || undefined,
    technology: values.technology || undefined,
    tags: values.tags || element?.tags || [],
    links: element?.links || [],
    metadata: element?.metadata || undefined,
    style: element?.style || undefined,
    parent: values.parentId || element?.parent || undefined,
  };
}

/**
 * Shared hook for element form submission logic.
 * Handles common patterns like ID generation, element creation/update, and error handling.
 */
export function useElementFormSubmit({
  element,
  kind,
  onSuccess,
  onError,
}: UseElementFormSubmitOptions): UseElementFormSubmitReturn {
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const data = useArchitectureStore((s) => s.model);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const handleSubmit = useCallback(
    async (values: {
      name: string;
      description?: string;
      technology?: string;
      customId?: boolean;
      idInput?: string;
      parentId?: string;
      tags?: string[];
      [key: string]: unknown;
    }) => {
      setIsSubmitting(true);
      setError(null);

      try {
        await updateArchitecture((model: SrujaModelDump) => {
          const newElements = { ...model.elements };
          const targetId = generateElementId(values, element, model, kind);

          if (!targetId) return model;

          const elementData = createElementData(targetId, values, element, kind);
          newElements[targetId] = elementData;

          return { ...model, elements: newElements };
        });

        onSuccess?.();
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        onError?.(error);
      } finally {
        setIsSubmitting(false);
      }
    },
    [element, kind, updateArchitecture, onSuccess, onError, data]
  );

  return { handleSubmit, isSubmitting, error };
}

// apps/website/src/shared/hooks/useExpansion.ts
import { useState, useCallback } from 'react';

/**
 * Hook for managing expand/collapse state for multiple items
 */
export function useExpansion<T extends string | number = string>(
  initialExpanded: T[] = []
) {
  const [expanded, setExpanded] = useState<Set<T>>(new Set(initialExpanded));

  const toggle = useCallback((item: T) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(item)) {
        next.delete(item);
      } else {
        next.add(item);
      }
      return next;
    });
  }, []);

  const expand = useCallback((item: T) => {
    setExpanded((prev) => new Set(prev).add(item));
  }, []);

  const collapse = useCallback((item: T) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      next.delete(item);
      return next;
    });
  }, []);

  const expandAll = useCallback((items: T[]) => {
    setExpanded(new Set(items));
  }, []);

  const collapseAll = useCallback(() => {
    setExpanded(new Set());
  }, []);

  const isExpanded = useCallback(
    (item: T) => expanded.has(item),
    [expanded]
  );

  return {
    expanded,
    toggle,
    expand,
    collapse,
    expandAll,
    collapseAll,
    isExpanded,
  };
}

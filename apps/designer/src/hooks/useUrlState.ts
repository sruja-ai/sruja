// apps/playground/src/hooks/useUrlState.ts
import { useEffect, useRef, useCallback } from "react";
import { useViewStore } from "../stores/viewStore";
import { toC4Level } from "../types/layout";

/**
 * Syncs view state with URL search parameters for easy testing and sharing
 * URL format: ?level=L1&expanded=System1,System2
 */
export function useUrlState() {
  const currentLevel = useViewStore((s) => s.currentLevel);
  const expandedNodes = useViewStore((s) => s.expandedNodes);
  const setLevel = useViewStore((s) => s.setLevel);
  const toggleExpand = useViewStore((s) => s.toggleExpand);
  const isInitialized = useRef(false);
  const lastSearchRef = useRef<string | null>(null);

  const applyFromUrl = useCallback(() => {
    const params = new URLSearchParams(window.location.search);

    const levelParam = params.get("level");
    if (levelParam && ["L1", "L2", "L3"].includes(levelParam)) {
      setLevel(toC4Level(levelParam));
    }

    const expandedParam = params.get("expanded");
    if (expandedParam !== null) {
      const expandedIds = expandedParam.split(",").filter(Boolean);
      const currentExpanded = Array.from(expandedNodes);

      expandedIds.forEach((id) => {
        if (!expandedNodes.has(id)) {
          toggleExpand(id);
        }
      });

      currentExpanded.forEach((id) => {
        if (!expandedIds.includes(id)) {
          toggleExpand(id);
        }
      });
    }
  }, [expandedNodes, setLevel, toggleExpand]);

  // Read from URL on mount only
  useEffect(() => {
    if (isInitialized.current) return;
    isInitialized.current = true;

    applyFromUrl();
    lastSearchRef.current = window.location.search;
  }, [applyFromUrl]); // Only run on mount

  // Re-apply from URL when browser history changes (back/forward)
  useEffect(() => {
    const onPopState = () => {
      if (lastSearchRef.current !== window.location.search) {
        lastSearchRef.current = window.location.search;
        applyFromUrl();
      }
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, [expandedNodes, applyFromUrl]);

  // Write to URL when state changes (but skip initial render)
  useEffect(() => {
    if (!isInitialized.current) return;

    const params = new URLSearchParams(window.location.search);

    // Update level
    params.set("level", currentLevel);

    // Update expanded nodes
    const expandedArray = Array.from(expandedNodes);
    if (expandedArray.length > 0) {
      params.set("expanded", expandedArray.join(","));
    } else {
      params.delete("expanded");
    }

    // Update URL without page reload
    const newUrl = `${window.location.pathname}?${params.toString()}`;
    if (window.location.search !== `?${params.toString()}`) {
      window.history.replaceState({}, "", newUrl);
      lastSearchRef.current = `?${params.toString()}`;
    }
  }, [currentLevel, expandedNodes]);
}

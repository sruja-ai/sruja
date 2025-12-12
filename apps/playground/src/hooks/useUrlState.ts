// apps/playground/src/hooks/useUrlState.ts
import { useEffect, useRef } from 'react';
import { useViewStore } from '../stores/viewStore';

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

    // Read from URL on mount only
    useEffect(() => {
        if (isInitialized.current) return;
        isInitialized.current = true;

        const params = new URLSearchParams(window.location.search);

        // Set level from URL
        const levelParam = params.get('level');
        if (levelParam && ['L1', 'L2', 'L3'].includes(levelParam)) {
            setLevel(levelParam as any);
        }

        // Set expanded nodes from URL
        const expandedParam = params.get('expanded');
        if (expandedParam) {
            const expandedIds = expandedParam.split(',').filter(Boolean);
            const currentExpanded = Array.from(expandedNodes);

            // Add nodes that are in URL but not in state
            expandedIds.forEach(id => {
                if (!expandedNodes.has(id)) {
                    toggleExpand(id);
                }
            });

            // Remove nodes that are in state but not in URL
            currentExpanded.forEach(id => {
                if (!expandedIds.includes(id)) {
                    toggleExpand(id);
                }
            });
        }
    }, []); // Only run on mount

    // Write to URL when state changes (but skip initial render)
    useEffect(() => {
        if (!isInitialized.current) return;

        const params = new URLSearchParams(window.location.search);

        // Update level
        params.set('level', currentLevel);

        // Update expanded nodes
        const expandedArray = Array.from(expandedNodes);
        if (expandedArray.length > 0) {
            params.set('expanded', expandedArray.join(','));
        } else {
            params.delete('expanded');
        }

        // Update URL without page reload
        const newUrl = `${window.location.pathname}?${params.toString()}`;
        if (window.location.search !== `?${params.toString()}`) {
            window.history.replaceState({}, '', newUrl);
        }
    }, [currentLevel, expandedNodes]);
}

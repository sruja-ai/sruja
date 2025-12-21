import { useMemo } from "react";
import type { SrujaModelDump } from "@sruja/shared";

interface UseNavigationDataProps {
    likec4Model: SrujaModelDump | null;
    filterQuery: string;
}

export function useNavigationData({ likec4Model, filterQuery }: UseNavigationDataProps) {
    // Derive flat lists from elements map
    const allElements = useMemo(() => {
        return likec4Model && likec4Model.elements
            ? Object.values(likec4Model.elements)
            : [];
    }, [likec4Model]);

    const persons = useMemo(() => allElements.filter((e) => e.kind === 'person'), [allElements]);
    const systems = useMemo(() => allElements.filter((e) => e.kind === 'system'), [allElements]);

    const filteredPersons = useMemo(() => {
        if (!filterQuery) return persons;
        const q = filterQuery.toLowerCase();
        return persons.filter((p) => (p.title || p.id).toLowerCase().includes(q));
    }, [persons, filterQuery]);

    const filteredSystems = useMemo(() => {
        if (!filterQuery) return systems;
        const q = filterQuery.toLowerCase();
        return systems.filter((s) => (s.title || s.id).toLowerCase().includes(q));
    }, [systems, filterQuery]);

    // Helper to find children
    const getChildren = (parentId: string, kind: string) =>
        allElements.filter((e) => (e as any).parent === parentId && e.kind === kind);

    return {
        allElements,
        filteredPersons,
        filteredSystems,
        getChildren,
    };
}

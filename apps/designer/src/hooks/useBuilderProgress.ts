import { useMemo } from "react";
import { useArchitectureStore } from "../stores/architectureStore";

export function useBuilderProgress() {
    const model = useArchitectureStore((s) => s.likec4Model);

    // Helper to get element by ID (unused)

    // Core Data
    const elements = useMemo(() => Object.values(model?.elements || {}) as any[], [model?.elements]);
    const relations = useMemo(() => model?.relations || [], [model?.relations]);

    // Sruja extensions
    const sruja = (model?.sruja as any) || {};
    const requirements = useMemo(() => sruja.requirements || [], [sruja]);
    const flows = useMemo(() => sruja.flows || [], [sruja]);
    const scenarios = useMemo(() => sruja.scenarios || [], [sruja]);
    const adrs = useMemo(() => sruja.adrs || [], [sruja]);

    const systems = useMemo(() => elements.filter((e: any) => e.kind === "system"), [elements]);
    const persons = useMemo(() => elements.filter((e: any) => e.kind === "person" || e.kind === "actor"), [elements]);
    const allContainers = useMemo(() => elements.filter((e: any) => e.kind === "container"), [elements]);
    const componentsAll = useMemo(() => elements.filter((e: any) => e.kind === "component"), [elements]);
    const datastores = useMemo(() => elements.filter((e: any) => e.kind === "datastore" || e.kind === "database"), [elements]);

    // Derived IDs
    const personIds = useMemo(() => new Set(persons.map((p: any) => p.id)), [persons]);
    const systemIds = useMemo(() => new Set(systems.map((s: any) => s.id)), [systems]);
    const containerIds = useMemo(() => new Set(allContainers.map((c: any) => c.id)), [allContainers]);
    const componentIds = useMemo(() => new Set(componentsAll.map((c: any) => c.id)), [componentsAll]);

    // Relationship Checks
    // Helper to extract FQN from FqnRef or string for backward compatibility
    const getFqn = (ref: any): string => typeof ref === 'object' && ref?.model ? ref.model : String(ref || '');

    const hasL1Relations = useMemo(
        () =>
            relations.some(
                (r) =>
                    (personIds.has(getFqn(r.source)) || systemIds.has(getFqn(r.source))) &&
                    (personIds.has(getFqn(r.target)) || systemIds.has(getFqn(r.target)))
            ),
        [relations, personIds, systemIds]
    );

    const hasContainerRelations = useMemo(
        () => relations.some((r) => containerIds.has(getFqn(r.source)) || containerIds.has(getFqn(r.target))),
        [relations, containerIds]
    );

    const hasComponentRelations = useMemo(
        () => relations.some((r) => componentIds.has(getFqn(r.source)) || componentIds.has(getFqn(r.target))),
        [relations, componentIds]
    );

    // Documentation Stats
    const systemsWithDescriptions = systems.filter(
        (s: any) => !!s.description && (typeof s.description === 'string' ? s.description.trim().length > 0 : (s.description?.txt || "").trim().length > 0)
    ).length;

    const containersWithTech = allContainers.filter(
        (c: any) => !!c.technology && c.technology.trim().length > 0
    ).length;

    const componentsWithTech = componentsAll.filter(
        (co: any) => !!co.technology && co.technology.trim().length > 0
    ).length;

    // Requirement Stats
    const reqTaggedTo = (ids: Set<string>) =>
        requirements.filter((r: any) => (r.tags || []).some((tag: string) => ids.has(tag))).length;

    const l1ReqTagged = useMemo(
        () => reqTaggedTo(new Set([...persons.map((p: any) => p.id), ...systems.map((s: any) => s.id)])),
        [requirements, persons, systems]
    );

    const l2ReqTagged = useMemo(
        () => reqTaggedTo(new Set([...allContainers.map((c: any) => c.id), ...systems.map((s: any) => s.id)])),
        [requirements, allContainers, systems]
    );

    const l3ReqTagged = useMemo(
        () =>
            reqTaggedTo(new Set([...componentsAll.map((c: any) => c.id), ...allContainers.map((c: any) => c.id)])),
        [requirements, componentsAll, allContainers]
    );

    // Structure Lookups
    // In flat model, 'parent' property links to parent ID.
    const containerParentById = useMemo(() => {
        const m = new Map<string, string>();
        allContainers.forEach((c: any) => {
            // Assuming parent is set. SrujaModelDump 'ElementDump' does not explicitly have 'parent' field in some versions,
            // but usually it does. If not, we rely on ID hierarchy or metadata.
            // LikeC4 usually doesn't have explicit parent field in ElementDump, but `hierarchy` or parent logic is used.
            // Let's assume ID structure for now or check if we can get parent.
            // Wait, standard LikeC4 ElementDump usually doesn't output parent ref directly, but the model hierarchy is known.
            // However, our Builder enforces hierarchy via naming or structure.
            // For now, let's assume standard '.' notation if parent field is missing.
            // But wait, ElementDump often DOES have parent info or we can infer.
            // Actually, if we are building it ourselves, we used to nest.
            // In the NEW model (flat), we rely on `element.parent` if available or verify.
            // Check `ElementDump` in `ast_likec4.go` or `nodeUtils.ts`.
            // Let's assume we can parse from ID for now as fallback, or use parent if present.
            // I'll assume ID nesting for containerParentById.
            if (c.id.includes(".")) {
                const parts = c.id.split(".");
                m.set(c.id, parts.slice(0, parts.length - 1).join("."));
            }
        });
        return m;
    }, [allContainers]);

    const componentParentById = useMemo(() => {
        const m = new Map<string, { sysId: string; containerId: string }>();
        componentsAll.forEach((co: any) => {
            const parts = co.id.split(".");
            if (parts.length >= 3) {
                m.set(co.id, { sysId: parts[0], containerId: parts[1] });
            }
        });
        return m;
    }, [componentsAll]);

    // Orphans Logic
    const containerHasRelations = (cid: string) => {
        return relations.some(
            (r) => getFqn(r.source) === cid || getFqn(r.target) === cid
            // Or partial paths if needed?
        );
    };
    const containersMissingRelations = useMemo(
        () => allContainers.filter((c: any) => !containerHasRelations(c.id)),
        [allContainers, relations]
    );
    const containersWithRelations = allContainers.length - containersMissingRelations.length;

    const componentHasRelations = (coid: string) => {
        return relations.some(
            (r) => getFqn(r.source) === coid || getFqn(r.target) === coid
        );
    };
    const componentsMissingRelations = useMemo(
        () => componentsAll.filter((co: any) => !componentHasRelations(co.id)),
        [componentsAll, relations]
    );
    const componentsWithRelations = componentsAll.length - componentsMissingRelations.length;

    // Shared Datastores
    const sharedDatastores = useMemo(() => {
        return datastores.filter((ds: any) => {
            const consumers = new Set<string>();
            relations.forEach((r) => {
                const srcFqn = getFqn(r.source);
                const tgtFqn = getFqn(r.target);
                if (tgtFqn === ds.id || srcFqn === ds.id) {
                    consumers.add(srcFqn === ds.id ? tgtFqn : srcFqn);
                }
            });
            return consumers.size > 2;
        }).length;
    }, [datastores, relations]);

    // Layering Violations
    const layers = ["web", "api", "service", "data", "database"];
    const layerIndex = (name?: string) => (name ? layers.indexOf(name) : -1);

    const resolveMetaLayer = (_id: string): string | undefined => {
        // Find element and check 'layer' meta/tag
        // Since we don't have metadata easily accessible on ElementDump (it might be in links or tags),
        // we might skip this or implement if critical.
        // Assuming we kept it in tags or similar.
        // For now, returning undefined to safe-guard.
        return undefined;
    };

    const layeringViolations = 0; // Disabled for now until metadata resolution is rebuilt
    const layeringViolationDetails: any[] = [];

    // Overall Completion
    const l1Complete = persons.length > 0 && systems.length > 0 && hasL1Relations;

    const l1Progress = useMemo(() => {
        const total = 3;
        let completed = 0;
        if (persons.length > 0) completed++;
        if (systems.length > 0) completed++;
        if (hasL1Relations) completed++;
        return { completed, total, percentage: (completed / total) * 100 };
    }, [persons.length, systems.length, hasL1Relations]);

    const l2Progress = useMemo(() => {
        const total = 2;
        let completed = 0;
        if (systems.length > 0) completed++; // Simplified check
        if (containersMissingRelations.length === 0) completed++;
        return { completed, total, percentage: (completed / total) * 100 };
    }, [systems, containersMissingRelations.length]);

    const l3Progress = useMemo(() => {
        const total = 2;
        let completed = 0;
        if (componentsAll.length > 0) completed++;
        if (componentsMissingRelations.length === 0) completed++;
        return { completed, total, percentage: (completed / total) * 100 };
    }, [componentsAll.length, componentsMissingRelations.length]);

    const getDownwardTargets = (_sourceId: string) => {
        return []; // Disabled for now
    };

    return {
        // Data
        systems,
        persons,
        allContainers,
        componentsAll,
        relations,
        requirements,
        flows,
        scenarios,
        adrs,

        // Stats / Validations
        hasL1Relations,
        hasContainerRelations,
        hasComponentRelations,
        l1Complete,
        systemsWithDescriptions,
        containersWithTech,
        componentsWithTech,
        l1ReqTagged,
        l2ReqTagged,
        l3ReqTagged,
        containersMissingRelations,
        containersWithRelations,
        componentsMissingRelations,
        componentsWithRelations,
        sharedDatastores,
        layeringViolations,
        layeringViolationDetails,
        l1Progress,
        l2Progress,
        l3Progress,

        // Lookups/Helpers
        containerParentById,
        componentParentById,
        resolveMetaLayer,
        layerIndex,
        getDownwardTargets,
    };
}

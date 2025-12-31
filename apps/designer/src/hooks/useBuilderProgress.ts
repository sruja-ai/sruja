import { useMemo } from "react";
import { useArchitectureStore } from "../stores/architectureStore";
import type { ElementDump, SrujaModelDump, SrujaExtensions } from "@sruja/shared";
import { extractText } from "@sruja/shared";

export function useBuilderProgress() {
  const model = useArchitectureStore((s) => s.model) as unknown as SrujaModelDump | null;

  // Helper to get element by ID (unused)

  // Core Data
  const elements = useMemo(
    () => Object.values(model?.elements || {}) as ElementDump[],
    [model?.elements]
  );
  const relations = useMemo(() => model?.relations || [], [model?.relations]);

  // Sruja extensions
  const sruja = useMemo(() => model?.sruja || ({} as SrujaExtensions), [model?.sruja]);
  const requirements = useMemo(() => sruja.requirements || [], [sruja.requirements]);
  const flows = useMemo(() => sruja.flows || [], [sruja.flows]);
  const scenarios = useMemo(() => sruja.scenarios || [], [sruja.scenarios]);
  const adrs = useMemo(() => sruja.adrs || [], [sruja.adrs]);

  const systems = useMemo(() => elements.filter((e) => e.kind === "system"), [elements]);
  const persons = useMemo(
    () => elements.filter((e) => e.kind === "person" || e.kind === "actor"),
    [elements]
  );
  const allContainers = useMemo(() => elements.filter((e) => e.kind === "container"), [elements]);
  const componentsAll = useMemo(() => elements.filter((e) => e.kind === "component"), [elements]);
  const datastores = useMemo(
    () => elements.filter((e) => e.kind === "datastore" || e.kind === "database"),
    [elements]
  );

  // Derived IDs
  const personIds = useMemo(() => new Set(persons.map((p) => p.id)), [persons]);
  const systemIds = useMemo(() => new Set(systems.map((s) => s.id)), [systems]);
  const containerIds = useMemo(() => new Set(allContainers.map((c) => c.id)), [allContainers]);
  const componentIds = useMemo(() => new Set(componentsAll.map((c) => c.id)), [componentsAll]);

  // Relationship Checks
  // Helper to extract FQN from FqnRef or string for backward compatibility
  const getFqn = (ref: any): string =>
    typeof ref === "object" && ref?.model ? ref.model : String(ref || "");

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
    () =>
      relations.some(
        (r) => containerIds.has(getFqn(r.source)) || containerIds.has(getFqn(r.target))
      ),
    [relations, containerIds]
  );

  const hasComponentRelations = useMemo(
    () =>
      relations.some(
        (r) => componentIds.has(getFqn(r.source)) || componentIds.has(getFqn(r.target))
      ),
    [relations, componentIds]
  );

  // Documentation Stats
  const systemsWithDescriptions = systems.filter((s) => {
    const desc = extractText(s.description);
    return !!desc && desc.trim().length > 0;
  }).length;

  const containersWithTech = allContainers.filter(
    (c) => !!c.technology && c.technology.trim().length > 0
  ).length;

  const componentsWithTech = componentsAll.filter(
    (co) => !!co.technology && co.technology.trim().length > 0
  ).length;

  const l1ReqTagged = useMemo(() => {
    const reqTaggedToLocal = (ids: Set<string>) =>
      requirements.filter((r) => (r.tags || []).some((tag: string) => ids.has(tag))).length;
    return reqTaggedToLocal(new Set([...persons.map((p) => p.id), ...systems.map((s) => s.id)]));
  }, [requirements, persons, systems]);

  const l2ReqTagged = useMemo(() => {
    const reqTaggedToLocal = (ids: Set<string>) =>
      requirements.filter((r) => (r.tags || []).some((tag: string) => ids.has(tag))).length;
    return reqTaggedToLocal(
      new Set([...allContainers.map((c) => c.id), ...systems.map((s) => s.id)])
    );
  }, [requirements, allContainers, systems]);

  const l3ReqTagged = useMemo(() => {
    const reqTaggedToLocal = (ids: Set<string>) =>
      requirements.filter((r) => (r.tags || []).some((tag) => ids.has(tag))).length;
    return reqTaggedToLocal(
      new Set([...componentsAll.map((c) => c.id), ...allContainers.map((c) => c.id)])
    );
  }, [requirements, componentsAll, allContainers]);

  // Structure Lookups
  // In flat model, 'parent' property links to parent ID.
  const containerParentById = useMemo(() => {
    const m = new Map<string, string>();
    allContainers.forEach((c) => {
      // Logic for parent extraction
      if (c.parent) {
        m.set(c.id, c.parent);
      } else if (c.id.includes(".")) {
        const parts = c.id.split(".");
        m.set(c.id, parts.slice(0, parts.length - 1).join("."));
      }
    });
    return m;
  }, [allContainers]);

  const componentParentById = useMemo(() => {
    const m = new Map<string, { sysId: string; containerId: string }>();
    componentsAll.forEach((co) => {
      // Fallback to ID splitting if parent not explicitly structured, but ideally we traverse up
      const parts = co.id.split(".");
      if (parts.length >= 3) {
        m.set(co.id, { sysId: parts[0], containerId: parts[1] });
      }
    });
    return m;
  }, [componentsAll]);

  const containersMissingRelations = useMemo(() => {
    const containerHasRelationsLocal = (cid: string) => {
      return relations.some((r) => getFqn(r.source) === cid || getFqn(r.target) === cid);
    };
    return allContainers.filter((c) => !containerHasRelationsLocal(c.id));
  }, [allContainers, relations]);
  const containersWithRelations = allContainers.length - containersMissingRelations.length;

  const componentsMissingRelations = useMemo(() => {
    const componentHasRelationsLocal = (coid: string) => {
      return relations.some((r) => getFqn(r.source) === coid || getFqn(r.target) === coid);
    };
    return componentsAll.filter((co) => !componentHasRelationsLocal(co.id));
  }, [componentsAll, relations]);
  const componentsWithRelations = componentsAll.length - componentsMissingRelations.length;

  // Shared Datastores
  const sharedDatastores = useMemo(() => {
    return datastores.filter((ds) => {
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

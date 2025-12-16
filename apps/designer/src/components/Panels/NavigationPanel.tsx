import { useMemo, useState, useEffect } from "react";
import {
  ChevronRight,
  ChevronDown,
  ChevronLeft,
  Building2,
  User,
  Play,
  Box,
  X,
  Edit,
  Trash2,
  Plus,
  Layout,
  Link2,
  Package,
  Database,
  MessageSquare,
  ShieldCheck,
} from "lucide-react";
import { Button, Input, Textarea } from "@sruja/ui";
import { useArchitectureStore, useViewStore, useSelectionStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import {
  EditScenarioForm,
  EditFlowForm,
  EditRequirementForm,
  EditADRForm,
  ConfirmDialog,
  SidePanel,
} from "../shared";
import type {
  SystemJSON,
  ContainerJSON,
  ScenarioJSON,
  FlowJSON,
  PersonJSON,
  ComponentJSON,
  RequirementJSON,
  ADRJSON,
  DataStoreJSON,
  QueueJSON,
} from "../../types";
import type { DiagramQualityMetrics } from "../../utils/diagramQuality";
import "./NavigationPanel.css";

function getGradeColor(grade: string): string {
  switch (grade) {
    case "A":
      return "#4ade80"; // green-400
    case "B":
      return "#a3e635"; // lime-400
    case "C":
      return "#facc15"; // yellow-400
    case "D":
      return "#fb923c"; // orange-400
    case "F":
      return "#f87171"; // red-400
    default:
      return "#9ca3af";
  }
}

interface NavigationPanelProps {
  onClose?: () => void;
}

export function NavigationPanel({ onClose }: NavigationPanelProps) {
  const data = useArchitectureStore((s) => s.data);
  const currentLevel = useViewStore((s) => s.currentLevel);
  const focusedSystemId = useViewStore((s) => s.focusedSystemId);
  const focusedContainerId = useViewStore((s) => s.focusedContainerId);
  const drillDown = useViewStore((s) => s.drillDown);
  const goToRoot = useViewStore((s) => s.goToRoot);
  const setActiveFlow = useSelectionStore((s) => s.setActiveFlow);
  const setActiveRequirement = useSelectionStore((s) => s.setActiveRequirement);
  const activeRequirement = useSelectionStore((s) => s.activeRequirement);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
  const [filterQuery, setFilterQuery] = useState("");

  const persons = useMemo(() => data?.architecture.persons ?? [], [data]);
  const systems = useMemo(() => data?.architecture.systems ?? [], [data]);
  const filteredPersons = useMemo(() => {
    if (!filterQuery) return persons;
    const q = filterQuery.toLowerCase();
    return persons.filter((p) => (p.label ?? p.id).toLowerCase().includes(q));
  }, [persons, filterQuery]);
  const filteredSystems = useMemo(() => {
    if (!filterQuery) return systems;
    const q = filterQuery.toLowerCase();
    return systems.filter((s) => (s.label ?? s.id).toLowerCase().includes(q));
  }, [systems, filterQuery]);
  const flows = useMemo(() => data?.architecture.flows ?? [], [data]);
  const scenarios = useMemo(() => data?.architecture.scenarios ?? [], [data]);
  const requirements = useMemo(() => data?.architecture.requirements ?? [], [data]);
  // Removed unused derived lists to satisfy type checks
  const allContainers = useMemo(() => systems.flatMap((s) => s.containers || []), [systems]);
  const [editScenario, setEditScenario] = useState<ScenarioJSON | undefined>(undefined);
  const [editFlow, setEditFlow] = useState<FlowJSON | undefined>(undefined);
  const [showScenarioForm, setShowScenarioForm] = useState(false);
  const [showFlowForm, setShowFlowForm] = useState(false);
  const [deleteScenario, setDeleteScenario] = useState<ScenarioJSON | undefined>(undefined);
  const [deleteFlow, setDeleteFlow] = useState<FlowJSON | undefined>(undefined);
  const [showDeleteScenarioConfirm, setShowDeleteScenarioConfirm] = useState(false);
  const [showDeleteFlowConfirm, setShowDeleteFlowConfirm] = useState(false);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Track layout metrics from ArchitectureCanvas
  const [layoutMetrics, setLayoutMetrics] = useState<DiagramQualityMetrics | null>(null);
  const [showHeatmap, setShowHeatmap] = useState(false);

  useEffect(() => {
    const checkMetrics = () => {
      const metrics = (window as any).__LAYOUT_METRICS__ as DiagramQualityMetrics | null;
      if (metrics) {
        setLayoutMetrics(metrics);
      }
      const heatmapVisible = (window as any).__LAYOUT_HEATMAP_VISIBLE__;
      if (typeof heatmapVisible === "boolean") {
        setShowHeatmap(heatmapVisible);
      }
    };
    checkMetrics();
    const interval = setInterval(checkMetrics, 500); // Check every 500ms
    return () => clearInterval(interval);
  }, []);

  const toggleHeatmap = () => {
    const toggleFn = (window as any).__LAYOUT_TOGGLE_HEATMAP__;
    if (toggleFn) {
      toggleFn(!showHeatmap);
      setShowHeatmap(!showHeatmap);
    }
  };

  // Track expanded nodes locally
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());

  // Collapsed state with localStorage persistence
  const [isCollapsed, setIsCollapsed] = useState(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("navigation-panel-collapsed");
      return saved === "true";
    }
    return false;
  });

  useEffect(() => {
    if (typeof window !== "undefined") {
      localStorage.setItem("navigation-panel-collapsed", String(isCollapsed));
    }
  }, [isCollapsed]);

  const toggleCollapse = () => {
    setIsCollapsed((prev) => !prev);
  };

  const toggleExpand = (id: string) => {
    setExpandedNodes((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  // Simple ID utilities
  const slugify = (text: string) =>
    text
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9_-]+/g, "-")
      .replace(/^-+|-+$/g, "");

  const collectIds = () => {
    const ids = new Set<string>();
    data?.architecture?.persons?.forEach((p) => ids.add(p.id));
    data?.architecture?.systems?.forEach((s) => {
      ids.add(s.id);
      s.containers?.forEach((c) => {
        ids.add(c.id);
        c.components?.forEach((co) => ids.add(co.id));
      });
      s.datastores?.forEach((ds) => ids.add(`${s.id}.${ds.id}`));
      s.queues?.forEach((q) => ids.add(`${s.id}.${q.id}`));
    });
    return ids;
  };

  const uniqueId = (base: string) => {
    const ids = collectIds();
    let candidate = slugify(base) || "item";
    let i = 1;
    while (ids.has(candidate)) {
      candidate = `${slugify(base)}-${i++}`;
    }
    return candidate;
  };

  // Add/Edit forms state
  const [isPersonFormOpen, setIsPersonFormOpen] = useState(false);
  const [personName, setPersonName] = useState("");
  const [editPerson, setEditPerson] = useState<PersonJSON | undefined>(undefined);
  const [showEditPerson, setShowEditPerson] = useState(false);
  const [deletePerson, setDeletePerson] = useState<PersonJSON | undefined>(undefined);
  const [showDeletePersonConfirm, setShowDeletePersonConfirm] = useState(false);

  const [isSystemFormOpen, setIsSystemFormOpen] = useState(false);
  const [systemName, setSystemName] = useState("");
  const [systemDescription, setSystemDescription] = useState("");
  const [useCustomSystemId, setUseCustomSystemId] = useState(false);
  const [systemIdInput, setSystemIdInput] = useState("");
  const [systemExternal, setSystemExternal] = useState(false);
  const [editSystem, setEditSystem] = useState<SystemJSON | undefined>(undefined);
  const [showEditSystem, setShowEditSystem] = useState(false);
  const [deleteSystem, setDeleteSystem] = useState<SystemJSON | undefined>(undefined);
  const [showDeleteSystemConfirm, setShowDeleteSystemConfirm] = useState(false);

  const [isContainerFormOpen, setIsContainerFormOpen] = useState(false);
  const [containerName, setContainerName] = useState("");
  const [containerTechnology, setContainerTechnology] = useState("");
  const [containerDescription, setContainerDescription] = useState("");
  const [useCustomContainerId, setUseCustomContainerId] = useState(false);
  const [containerIdInput, setContainerIdInput] = useState("");
  const [containerParentSystemId, setContainerParentSystemId] = useState<string | null>(null);
  const [editContainer, setEditContainer] = useState<ContainerJSON | undefined>(undefined);
  const [editContainerSystemId, setEditContainerSystemId] = useState<string | null>(null);
  const [showEditContainer, setShowEditContainer] = useState(false);
  const [deleteContainer, setDeleteContainer] = useState<ContainerJSON | undefined>(undefined);
  const [deleteContainerSystemId, setDeleteContainerSystemId] = useState<string | null>(null);
  const [showDeleteContainerConfirm, setShowDeleteContainerConfirm] = useState(false);

  const [isComponentFormOpen, setIsComponentFormOpen] = useState(false);
  const [componentName, setComponentName] = useState("");
  const [componentTechnology, setComponentTechnology] = useState("");
  const [componentDescription, setComponentDescription] = useState("");
  const [useCustomComponentId, setUseCustomComponentId] = useState(false);
  const [componentIdInput, setComponentIdInput] = useState("");
  const [componentParentSystemId, setComponentParentSystemId] = useState<string | null>(null);
  const [componentParentContainerId, setComponentParentContainerId] = useState<string | null>(null);
  const componentSystemContainers = useMemo(() => {
    if (!componentParentSystemId) return [];
    const system = systems.find((s) => s.id === componentParentSystemId);
    return system?.containers || [];
  }, [systems, componentParentSystemId]);
  const [editComponent, setEditComponent] = useState<ComponentJSON | undefined>(undefined);
  const [editComponentContainerId, setEditComponentContainerId] = useState<string | null>(null);
  const [showEditComponent, setShowEditComponent] = useState(false);
  const [deleteComponent, setDeleteComponent] = useState<ComponentJSON | undefined>(undefined);
  const [deleteComponentContainerId, setDeleteComponentContainerId] = useState<string | null>(null);
  const [showDeleteComponentConfirm, setShowDeleteComponentConfirm] = useState(false);

  // Relation form state
  const [isRelationFormOpen, setIsRelationFormOpen] = useState(false);
  const [relationFromId, setRelationFromId] = useState<string>("");
  const [relationToId, setRelationToId] = useState<string>("");
  const [relationLabel, setRelationLabel] = useState<string>("");
  const [relationTech, setRelationTech] = useState<string>("");
  const [showRequirementForm, setShowRequirementForm] = useState(false);
  const [editRequirement, setEditRequirement] = useState<RequirementJSON | undefined>(undefined);
  const [showDeleteRequirementConfirm, setShowDeleteRequirementConfirm] = useState(false);
  const [deleteRequirement, setDeleteRequirement] = useState<RequirementJSON | undefined>(
    undefined
  );
  const [showADRForm, setShowADRForm] = useState(false);
  const [editADR, setEditADR] = useState<ADRJSON | undefined>(undefined);
  const [showDeleteADRConfirm, setShowDeleteADRConfirm] = useState(false);
  const [deleteADR, setDeleteADR] = useState<ADRJSON | undefined>(undefined);

  // Build hierarchical node options with FQN display
  const relationNodeOptions = useMemo(() => {
    const options: Array<{ id: string; label: string; type: string }> = [];

    // Add persons
    persons.forEach((p) => {
      options.push({
        id: p.id,
        label: `Person: ${p.label ?? p.id}`,
        type: "person",
      });
    });

    // Add systems
    systems.forEach((s) => {
      const systemName = s.label ?? s.id;
      options.push({
        id: s.id,
        label: `System: ${systemName}`,
        type: "system",
      });

      // Add containers with system prefix
      s.containers?.forEach((c) => {
        const containerName = c.label ?? c.id;
        options.push({
          id: c.id,
          label: `Container: ${systemName} -> ${containerName}`,
          type: "container",
        });

        // Add components with full path
        c.components?.forEach((co) => {
          const componentName = co.label ?? co.id;
          options.push({
            id: co.id,
            label: `Component: ${systemName} -> ${containerName} -> ${componentName}`,
            type: "component",
          });
        });
      });

      // Add datastores with system prefix
      s.datastores?.forEach((ds) => {
        const datastoreName = ds.label ?? ds.id;
        options.push({
          id: `${s.id}.${ds.id}`,
          label: `DataStore: ${systemName} -> ${datastoreName}`,
          type: "datastore",
        });
      });

      // Add queues with system prefix
      s.queues?.forEach((q) => {
        const queueName = q.label ?? q.id;
        options.push({
          id: `${s.id}.${q.id}`,
          label: `Queue: ${systemName} -> ${queueName}`,
          type: "queue",
        });
      });
    });

    return options;
  }, [persons, systems]);

  const submitAddRelation = async () => {
    if (!relationFromId || !relationToId || relationFromId === relationToId) {
      return;
    }
    await updateArchitecture((arch) => {
      const relations = [...(arch.architecture?.relations || [])];
      const exists = relations.some(
        (r) => r.from === relationFromId && r.to === relationToId && r.label === relationLabel
      );
      if (!exists) {
        relations.push({
          from: relationFromId,
          to: relationToId,
          label: relationLabel || undefined,
          technology: relationTech || undefined,
        });
      }
      return { ...arch, architecture: { ...arch.architecture, relations } };
    });
    setIsRelationFormOpen(false);
    setRelationFromId("");
    setRelationToId("");
    setRelationLabel("");
    setRelationTech("");
  };

  const submitAddPerson = async () => {
    const id = uniqueId(personName);
    await updateArchitecture((arch) => {
      const persons = [...(arch.architecture?.persons || [])];
      persons.push({ id, label: personName });
      return { ...arch, architecture: { ...arch.architecture, persons } };
    });
    setPersonName("");
    setIsPersonFormOpen(false);
  };

  const submitEditPerson = async () => {
    if (!editPerson) return;
    await updateArchitecture((arch) => {
      const persons = (arch.architecture?.persons || []).map((p) =>
        p.id === editPerson.id ? { ...p, label: personName } : p
      );
      return { ...arch, architecture: { ...arch.architecture, persons } };
    });
    setShowEditPerson(false);
    setEditPerson(undefined);
    setPersonName("");
  };

  const submitAddSystem = async () => {
    const id = (useCustomSystemId ? systemIdInput || "" : "") || uniqueId(systemName);
    await updateArchitecture((arch) => {
      const systems = [...(arch.architecture?.systems || [])];
      const metadata = systemExternal ? [{ key: "tags", value: "external" }] : undefined;
      systems.push({
        id,
        label: systemName,
        description: systemDescription || undefined,
        containers: [],
        metadata,
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setSystemName("");
    setSystemDescription("");
    setUseCustomSystemId(false);
    setSystemIdInput("");
    setSystemExternal(false);
    setIsSystemFormOpen(false);
  };

  const submitEditSystem = async () => {
    if (!editSystem) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        if (s.id !== editSystem.id) return s;
        // Preserve existing metadata, update tags for external
        const existingMetadata = s.metadata || [];
        const otherMetadata = existingMetadata.filter(
          (m) => m.key.toLowerCase() !== "tags" && m.key.toLowerCase() !== "external"
        );
        const metadata = systemExternal
          ? [...otherMetadata, { key: "tags", value: "external" }]
          : otherMetadata.length > 0
            ? otherMetadata
            : undefined;
        return { ...s, label: systemName, metadata };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setShowEditSystem(false);
    setEditSystem(undefined);
    setSystemName("");
    setSystemExternal(false);
  };

  const submitAddContainer = async () => {
    if (!containerParentSystemId) return;
    const id = (useCustomContainerId ? containerIdInput || "" : "") || uniqueId(containerName);
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        if (s.id !== containerParentSystemId) return s;
        const containers = [...(s.containers || [])];
        containers.push({
          id,
          label: containerName,
          technology: containerTechnology || undefined,
          description: containerDescription || undefined,
          components: [],
        });
        return { ...s, containers };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setContainerName("");
    setContainerTechnology("");
    setContainerDescription("");
    setUseCustomContainerId(false);
    setContainerIdInput("");
    setContainerParentSystemId(null);
    setIsContainerFormOpen(false);
  };

  const submitEditContainer = async () => {
    if (!editContainer || !editContainerSystemId) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        if (s.id !== editContainerSystemId) return s;
        const containers = (s.containers || []).map((c) =>
          c.id === editContainer.id ? { ...c, label: containerName } : c
        );
        return { ...s, containers };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setShowEditContainer(false);
    setEditContainer(undefined);
    setEditContainerSystemId(null);
    setContainerName("");
  };

  const submitAddComponent = async () => {
    if (!componentParentSystemId || !componentParentContainerId) return;
    const id = (useCustomComponentId ? componentIdInput || "" : "") || uniqueId(componentName);
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        if (s.id !== componentParentSystemId) return s;
        const containers = (s.containers || []).map((c) => {
          if (c.id !== componentParentContainerId) return c;
          const components = [...(c.components || [])];
          components.push({
            id,
            label: componentName,
            technology: componentTechnology || undefined,
            description: componentDescription || undefined,
          });
          return { ...c, components };
        });
        return { ...s, containers };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setComponentName("");
    setComponentTechnology("");
    setComponentDescription("");
    setUseCustomComponentId(false);
    setComponentIdInput("");
    setComponentParentSystemId(null);
    setComponentParentContainerId(null);
    setIsComponentFormOpen(false);
  };

  const submitEditComponent = async () => {
    if (!editComponent || !editComponentContainerId) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        const containers = (s.containers || []).map((c) => {
          if (c.id !== editComponentContainerId) return c;
          const components = (c.components || []).map((co) =>
            co.id === editComponent.id ? { ...co, label: componentName } : co
          );
          return { ...c, components };
        });
        return { ...s, containers };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setShowEditComponent(false);
    setEditComponent(undefined);
    setEditComponentContainerId(null);
    setComponentName("");
  };

  const removeRelationsForIds = (arch: any, ids: Set<string>) =>
    (arch.architecture?.relations || []).filter((r: any) => !ids.has(r.from) && !ids.has(r.to));

  const doDeletePerson = async () => {
    if (!deletePerson) return;
    await updateArchitecture((arch) => {
      const persons = (arch.architecture?.persons || []).filter((p) => p.id !== deletePerson.id);
      const ids = new Set<string>([deletePerson.id]);
      const relations = removeRelationsForIds(arch, ids);
      return { ...arch, architecture: { ...arch.architecture, persons, relations } };
    });
    setShowDeletePersonConfirm(false);
    setDeletePerson(undefined);
  };

  const doDeleteSystem = async () => {
    if (!deleteSystem) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).filter((s) => s.id !== deleteSystem.id);
      const ids = new Set<string>([deleteSystem.id]);
      (deleteSystem.containers || []).forEach((c) => {
        ids.add(c.id);
        (c.components || []).forEach((co) => ids.add(co.id));
      });
      (deleteSystem.datastores || []).forEach((ds) => ids.add(`${deleteSystem.id}.${ds.id}`));
      (deleteSystem.queues || []).forEach((q) => ids.add(`${deleteSystem.id}.${q.id}`));
      const relations = removeRelationsForIds(arch, ids);
      return { ...arch, architecture: { ...arch.architecture, systems, relations } };
    });
    setShowDeleteSystemConfirm(false);
    setDeleteSystem(undefined);
  };

  const doDeleteContainer = async () => {
    if (!deleteContainer || !deleteContainerSystemId) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        if (s.id !== deleteContainerSystemId) return s;
        const containers = (s.containers || []).filter((c) => c.id !== deleteContainer.id);
        return { ...s, containers };
      });
      const ids = new Set<string>([deleteContainer.id]);
      (deleteContainer.components || []).forEach((co) => ids.add(co.id));
      const relations = removeRelationsForIds(arch, ids);
      return { ...arch, architecture: { ...arch.architecture, systems, relations } };
    });
    setShowDeleteContainerConfirm(false);
    setDeleteContainer(undefined);
    setDeleteContainerSystemId(null);
  };

  const doDeleteComponent = async () => {
    if (!deleteComponent || !deleteComponentContainerId) return;
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        const containers = (s.containers || []).map((c) => {
          if (c.id !== deleteComponentContainerId) return c;
          const components = (c.components || []).filter((co) => co.id !== deleteComponent.id);
          return { ...c, components };
        });
        return { ...s, containers };
      });
      const ids = new Set<string>([deleteComponent.id]);
      const relations = removeRelationsForIds(arch, ids);
      return { ...arch, architecture: { ...arch.architecture, systems, relations } };
    });
    setShowDeleteComponentConfirm(false);
    setDeleteComponent(undefined);
    setDeleteComponentContainerId(null);
  };

  // removed unused layer helpers
  // Removed unused layering helpers and relation utilities to satisfy type checks

  return (
    <div className={`navigation-panel ${isCollapsed ? "collapsed" : ""}`}>
      {!isCollapsed && (
        <div className="nav-header">
          <h3 className="nav-title">Navigation</h3>
        </div>
      )}
      {!data && !isCollapsed && (
        <div className="panel-empty">Load an architecture to see navigation</div>
      )}
      {!isCollapsed && (
        <div className="nav-search-row">
          <Input
            placeholder="Search systems, actors..."
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
          />
        </div>
      )}

      {/* Context Quick Actions */}
      {!isCollapsed && (
        <div className="nav-quick-actions">
          {focusedSystemId && (
            <div className="qa-row">
              <span className="qa-label">System</span>
              <span className="qa-value">
                {systems.find((s) => s.id === focusedSystemId)?.label ?? focusedSystemId}
              </span>
              {isEditMode() && (
                <div className="qa-actions">
                  <button
                    className="qa-btn"
                    title="Add Container to this system"
                    onClick={() => {
                      setContainerParentSystemId(focusedSystemId);
                      setContainerName("");
                      setContainerTechnology("");
                      setIsContainerFormOpen(true);
                    }}
                  >
                    <Box size={12} />
                    <span>Add Container</span>
                  </button>
                  <button
                    className="qa-btn"
                    title="Relate from this system"
                    onClick={() => {
                      setRelationFromId(focusedSystemId);
                      setRelationToId("");
                      setRelationLabel("");
                      setRelationTech("");
                      setIsRelationFormOpen(true);
                    }}
                  >
                    <Link2 size={12} />
                    <span>Relate to…</span>
                  </button>
                </div>
              )}
            </div>
          )}
          {focusedContainerId && (
            <div className="qa-row">
              <span className="qa-label">Container</span>
              <span className="qa-value">
                {systems.flatMap((s) => s.containers || []).find((c) => c.id === focusedContainerId)
                  ?.label ?? focusedContainerId}
              </span>
              {isEditMode() && (
                <div className="qa-actions">
                  <button
                    className="qa-btn"
                    title="Add Component to this container"
                    onClick={() => {
                      const container = allContainers.find((c) => c.id === focusedContainerId);
                      if (container) {
                        const sysId = systems.find((s) =>
                          s.containers?.some((c) => c.id === container.id)
                        )?.id;
                        setComponentParentSystemId(sysId || null);
                        setComponentParentContainerId(focusedContainerId);
                        setComponentName("");
                        setIsComponentFormOpen(true);
                      }
                    }}
                  >
                    <Plus size={12} />
                    <span>Add Component</span>
                  </button>
                  <button
                    className="qa-btn"
                    title="Relate from this container"
                    onClick={() => {
                      setRelationFromId(focusedContainerId);
                      setRelationToId("");
                      setRelationLabel("");
                      setRelationTech("");
                      setIsRelationFormOpen(true);
                    }}
                  >
                    <Link2 size={12} />
                    <span>Relate to…</span>
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      )}
      {/* Collapse Toggle */}
      <button
        className="nav-collapse-btn"
        onClick={toggleCollapse}
        aria-label={isCollapsed ? "Expand navigation" : "Collapse navigation"}
        title={isCollapsed ? "Expand navigation" : "Collapse navigation"}
      >
        {isCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
      </button>

      {onClose && (
        <div className="panel-mobile-header">
          <span>Navigation</span>
          <button className="panel-close-btn" onClick={onClose} aria-label="Close navigation">
            <X size={18} />
          </button>
        </div>
      )}
      {/* Level Selector */}
      <div className="nav-section">
        {!isCollapsed && <div className="nav-section-title">View Level</div>}
        <div className={`level-buttons ${isCollapsed ? "collapsed" : ""}`}>
          <button
            className={`level-btn ${currentLevel === "L1" ? "active" : ""}`}
            onClick={goToRoot}
            title="System Context - Shows systems and actors"
          >
            {isCollapsed ? "L1" : "L1 Context"}
          </button>
          <button
            className={`level-btn ${currentLevel === "L2" ? "active" : ""}`}
            disabled={!focusedSystemId}
            title="Container View - Detailed system internals"
          >
            {isCollapsed ? "L2" : "L2 Container"}
          </button>
          <button
            className={`level-btn ${currentLevel === "L3" ? "active" : ""}`}
            disabled={currentLevel !== "L3"}
            title="Component View - Fine-grained components"
          >
            {isCollapsed ? "L3" : "L3 Component"}
          </button>
        </div>
      </div>

      {/* Systems Tree */}
      <div className="nav-section">
        <div className="nav-section-title">
          <Building2 size={14} />
          {!isCollapsed && <span>Systems ({filteredSystems.length})</span>}
        </div>
        <ul className="nav-tree">
          {filteredSystems.length === 0 && (
            <li className="nav-empty">
              No systems.{" "}
              {isEditMode() && (
                <button
                  className="link-btn"
                  onClick={() => {
                    setSystemName("");
                    setSystemExternal(false);
                    setIsSystemFormOpen(true);
                  }}
                >
                  Add a system
                </button>
              )}
            </li>
          )}
          {filteredSystems.map((system) => (
            <SystemTreeItem
              key={system.id}
              system={system}
              isExpanded={expandedNodes.has(system.id)}
              isSelected={focusedSystemId === system.id}
              focusedContainerId={focusedContainerId}
              isCollapsed={isCollapsed}
              onToggle={() => toggleExpand(system.id)}
              onSelect={() => drillDown(system.id, "system")}
              onSelectContainer={(containerId) => drillDown(containerId, "container", system.id)}
              onAddContainer={() => {
                setContainerParentSystemId(system.id);
                setContainerName("");
                setIsContainerFormOpen(true);
              }}
              onEditSystem={() => {
                setEditSystem(system);
                setSystemName(system.label ?? system.id);
                const isExt =
                  system.metadata?.some(
                    (m) =>
                      m.key.toLowerCase() === "tags" && m.value?.toLowerCase().includes("external")
                  ) ||
                  system.metadata?.some(
                    (m) => m.key.toLowerCase() === "external" && m.value?.toLowerCase() === "true"
                  );
                setSystemExternal(isExt || false);
                setShowEditSystem(true);
              }}
              onDeleteSystem={() => {
                setDeleteSystem(system);
                setShowDeleteSystemConfirm(true);
              }}
              onEditContainer={(container, systemId) => {
                setEditContainer(container);
                setEditContainerSystemId(systemId);
                setContainerName(container.label ?? container.id);
                setShowEditContainer(true);
              }}
              onDeleteContainer={(container, systemId) => {
                setDeleteContainer(container);
                setDeleteContainerSystemId(systemId);
                setShowDeleteContainerConfirm(true);
              }}
            />
          ))}
        </ul>
      </div>

      {/* Persons */}
      <div className="nav-section">
        <div className="nav-section-title">
          <User size={14} />
          {!isCollapsed && <span>Actors ({filteredPersons.length})</span>}
        </div>
        <ul className="nav-tree">
          {filteredPersons.length === 0 && (
            <li className="nav-empty">
              No actors.{" "}
              {isEditMode() && (
                <button
                  className="link-btn"
                  onClick={() => {
                    setPersonName("");
                    setIsPersonFormOpen(true);
                  }}
                >
                  Add a person
                </button>
              )}
            </li>
          )}
          {filteredPersons.map((person) => (
            <li key={person.id} className="nav-item">
              <div className="nav-item-row">
                <button
                  className="nav-item-btn"
                  title={isCollapsed ? (person.label ?? person.id) : undefined}
                >
                  <User size={12} />
                  {!isCollapsed && (
                    <span className="nav-item-label">{person.label ?? person.id}</span>
                  )}
                </button>
                {!isCollapsed && isEditMode() && (
                  <div className="nav-item-actions">
                    <button
                      className="nav-edit-btn"
                      title="Edit Person"
                      onClick={() => {
                        setEditPerson(person);
                        setPersonName(person.label ?? person.id);
                        setShowEditPerson(true);
                      }}
                    >
                      <Edit size={12} />
                    </button>
                    <button
                      className="nav-delete-btn"
                      title="Delete Person"
                      onClick={() => {
                        setDeletePerson(person);
                        setShowDeletePersonConfirm(true);
                      }}
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>
                )}
              </div>
            </li>
          ))}
        </ul>
      </div>

      {/* Add */}
      {!isCollapsed && isEditMode() && (
        <div className="nav-section">
          <div className="nav-section-title">
            <Plus size={14} />
            <span>Add</span>
          </div>
          <div className="nav-add-actions">
            <button
              className="nav-add-action-btn"
              onClick={() => {
                setSystemName("");
                setSystemExternal(false);
                setIsSystemFormOpen(true);
              }}
            >
              <Building2 size={14} />
              <span>System</span>
            </button>
            <button
              className="nav-add-action-btn"
              onClick={() => {
                setContainerParentSystemId(focusedSystemId || (systems[0]?.id ?? null));
                setContainerName("");
                setContainerTechnology("");
                setIsContainerFormOpen(true);
              }}
            >
              <Box size={14} />
              <span>Container</span>
            </button>
            <button
              className="nav-add-action-btn"
              onClick={() => {
                setComponentParentSystemId(null);
                setComponentParentContainerId(null);
                setComponentName("");
                setIsComponentFormOpen(true);
              }}
              disabled={systems.length === 0}
            >
              <Package size={14} />
              <span>Component</span>
            </button>
            <button
              className="nav-add-action-btn"
              onClick={() => {
                setPersonName("");
                setIsPersonFormOpen(true);
              }}
            >
              <User size={14} />
              <span>Actor</span>
            </button>
            <button className="nav-add-action-btn" onClick={() => setIsRelationFormOpen(true)}>
              <Link2 size={14} />
              <span>Relation</span>
            </button>
          </div>
        </div>
      )}

      {/* Scenarios */}
      {scenarios.length > 0 && (
        <div className="nav-section">
          <div className="nav-section-title">
            <Play size={14} />
            {!isCollapsed && <span>User Stories & Scenarios ({scenarios.length})</span>}
            {!isCollapsed && isEditMode() && (
              <button
                className="nav-add-btn"
                onClick={() => {
                  setEditScenario(undefined);
                  setShowScenarioForm(true);
                }}
                title="Add Scenario"
              >
                <Plus size={14} />
              </button>
            )}
          </div>
          <ul className="nav-tree">
            {scenarios.map((scenario) => (
              <li
                key={scenario.id}
                className="nav-item clickable"
                onClick={() => setActiveFlow(scenario as unknown as FlowJSON)}
                title={isCollapsed ? (scenario.title ?? scenario.label ?? scenario.id) : undefined}
              >
                <Play size={12} />
                {!isCollapsed && (
                  <>
                    <span className="nav-item-label">
                      {scenario.title ?? scenario.label ?? scenario.id}
                    </span>
                    {isEditMode() && (
                      <div className="nav-item-actions">
                        <button
                          className="nav-edit-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setEditScenario(scenario);
                            setShowScenarioForm(true);
                          }}
                          title="Edit Scenario"
                        >
                          <Edit size={12} />
                        </button>
                        <button
                          className="nav-delete-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setDeleteScenario(scenario);
                            setShowDeleteScenarioConfirm(true);
                          }}
                          title="Delete Scenario"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    )}
                  </>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Flows */}
      {flows.length > 0 && (
        <div className="nav-section">
          <div className="nav-section-title">
            <Play size={14} />
            {!isCollapsed && <span>Data Flows ({flows.length})</span>}
            {!isCollapsed && isEditMode() && (
              <button
                className="nav-add-btn"
                onClick={() => {
                  setEditFlow(undefined);
                  setShowFlowForm(true);
                }}
                title="Add Flow"
              >
                <Plus size={14} />
              </button>
            )}
          </div>
          <ul className="nav-tree">
            {flows.map((flow) => (
              <li
                key={flow.id}
                className="nav-item clickable"
                onClick={() => setActiveFlow(flow)}
                title={isCollapsed ? (flow.title ?? flow.label ?? flow.id) : undefined}
              >
                <Play size={12} />
                {!isCollapsed && (
                  <>
                    <span className="nav-item-label">{flow.title ?? flow.label ?? flow.id}</span>
                    {isEditMode() && (
                      <div className="nav-item-actions">
                        <button
                          className="nav-edit-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setEditFlow(flow);
                            setShowFlowForm(true);
                          }}
                          title="Edit Flow"
                        >
                          <Edit size={12} />
                        </button>
                        <button
                          className="nav-delete-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setDeleteFlow(flow);
                            setShowDeleteFlowConfirm(true);
                          }}
                          title="Delete Flow"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    )}
                  </>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Requirements */}
      {requirements.length > 0 && (
        <div className="nav-section">
          <div className="nav-section-title">
            <ShieldCheck size={14} />
            {!isCollapsed && <span>Requirements ({requirements.length})</span>}
          </div>
          <ul className="nav-tree">
            {requirements.map((req) => (
              <li
                key={req.id}
                className={`nav-item clickable ${activeRequirement === req.id ? "selected" : ""}`}
                onClick={() => setActiveRequirement(req.id === activeRequirement ? null : req.id)}
                title={isCollapsed ? req.title || req.id : undefined}
              >
                <ShieldCheck size={12} />
                {!isCollapsed && <span className="nav-item-label">{req.title || req.id}</span>}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Layout Quality Score */}
      {layoutMetrics && !isCollapsed && (
        <div className="nav-section">
          <div className="nav-section-title">
            <Layout size={14} />
            <span>Layout Quality</span>
          </div>
          <div className="layout-quality-display">
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: "8px",
              }}
            >
              <span style={{ fontSize: "13px", fontWeight: 600 }}>Score</span>
              <span
                style={{
                  fontSize: "16px",
                  fontWeight: 800,
                  color: getGradeColor(layoutMetrics.grade),
                }}
              >
                {layoutMetrics.weightedScore.toFixed(1)} ({layoutMetrics.grade})
              </span>
            </div>
            <div
              style={{
                fontSize: "11px",
                color: "#9ca3af",
                display: "flex",
                flexDirection: "column",
                gap: "4px",
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span>Crossings:</span>
                <span>{layoutMetrics.edgeCrossings}</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span>Overlaps:</span>
                <span>{layoutMetrics.overlappingNodes.length}</span>
              </div>
              {layoutMetrics.parentChildContainment &&
                layoutMetrics.parentChildContainment.length > 0 && (
                  <div
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      color: "#f87171",
                      fontWeight: 600,
                    }}
                  >
                    <span>🚨 Containment:</span>
                    <span>{layoutMetrics.parentChildContainment.length}</span>
                  </div>
                )}
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span>Bad Flow:</span>
                <span>{layoutMetrics.directionViolations?.length ?? 0}</span>
              </div>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  color: layoutMetrics.emptySpaceScore < 80 ? "#fbbf24" : "#9ca3af",
                }}
              >
                <span>Empty Space:</span>
                <span>
                  {(layoutMetrics.emptySpace * 100).toFixed(0)}% (
                  {layoutMetrics.emptySpaceScore.toFixed(0)})
                </span>
              </div>
              {(layoutMetrics.edgeLabelOverlaps > 0 || layoutMetrics.clippedNodeLabels > 0) && (
                <>
                  {layoutMetrics.edgeLabelOverlaps > 0 && (
                    <div
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        color: "#f87171",
                      }}
                    >
                      <span>Label Overlaps:</span>
                      <span>{layoutMetrics.edgeLabelOverlaps}</span>
                    </div>
                  )}
                  {layoutMetrics.clippedNodeLabels > 0 && (
                    <div
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        color: "#f87171",
                      }}
                    >
                      <span>Clipped Labels:</span>
                      <span>{layoutMetrics.clippedNodeLabels}</span>
                    </div>
                  )}
                </>
              )}
            </div>
            <label
              style={{
                display: "flex",
                alignItems: "center",
                cursor: "pointer",
                fontSize: "11px",
                marginTop: "8px",
                userSelect: "none",
                color: "#d1d5db",
              }}
            >
              <input
                type="checkbox"
                checked={showHeatmap}
                onChange={toggleHeatmap}
                style={{ marginRight: "6px" }}
              />
              Show Badness Heatmap
            </label>
          </div>
        </div>
      )}

      {/* Edit Forms */}
      <EditScenarioForm
        isOpen={showScenarioForm}
        onClose={() => {
          setShowScenarioForm(false);
          setEditScenario(undefined);
        }}
        scenario={editScenario}
      />
      <EditFlowForm
        isOpen={showFlowForm}
        onClose={() => {
          setShowFlowForm(false);
          setEditFlow(undefined);
        }}
        flow={editFlow}
      />
      <ConfirmDialog
        isOpen={showDeleteScenarioConfirm}
        onClose={() => {
          setShowDeleteScenarioConfirm(false);
          setDeleteScenario(undefined);
        }}
        onConfirm={async () => {
          if (deleteScenario) {
            await updateArchitecture((arch) => {
              if (!arch.architecture) return arch;
              const scenarios = (arch.architecture.scenarios || []).filter(
                (s) => s.id !== deleteScenario.id
              );
              return {
                ...arch,
                architecture: {
                  ...arch.architecture,
                  scenarios,
                },
              };
            });
          }
        }}
        title="Delete Scenario"
        message={`Are you sure you want to delete scenario "${deleteScenario?.id}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteFlowConfirm}
        onClose={() => {
          setShowDeleteFlowConfirm(false);
          setDeleteFlow(undefined);
        }}
        onConfirm={async () => {
          if (deleteFlow) {
            await updateArchitecture((arch) => {
              if (!arch.architecture) return arch;
              const flows = (arch.architecture.flows || []).filter((f) => f.id !== deleteFlow.id);
              return {
                ...arch,
                architecture: {
                  ...arch.architecture,
                  flows,
                },
              };
            });
          }
        }}
        title="Delete Flow"
        message={`Are you sure you want to delete flow "${deleteFlow?.id}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
      {/* Add/Edit SidePanels */}
      <SidePanel
        isOpen={isSystemFormOpen}
        onClose={() => {
          setIsSystemFormOpen(false);
          setSystemExternal(false);
        }}
        title="Add System"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setIsSystemFormOpen(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitAddSystem} type="button">
              Add
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={systemName}
            onChange={(e) => setSystemName(e.target.value)}
            required
            placeholder="System name"
          />
          <Textarea
            label="Description"
            value={systemDescription}
            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
              setSystemDescription(e.target.value)
            }
            rows={3}
            placeholder="What does this system do? Responsibilities, boundaries, integrations."
          />
          <div className="form-group" style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input
              id="system-custom-id"
              type="checkbox"
              checked={useCustomSystemId}
              onChange={(e) => setUseCustomSystemId(e.target.checked)}
            />
            <label htmlFor="system-custom-id">Set custom ID (optional)</label>
          </div>
          {useCustomSystemId && (
            <Input
              label="ID"
              value={systemIdInput}
              onChange={(e) => setSystemIdInput(e.target.value)}
              placeholder="If empty, ID is auto-generated from name"
            />
          )}
          <div className="form-group" style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input
              id="system-external"
              type="checkbox"
              checked={systemExternal}
              onChange={(e) => setSystemExternal(e.target.checked)}
            />
            <label htmlFor="system-external">External</label>
          </div>
        </div>
      </SidePanel>

      <SidePanel
        isOpen={showEditSystem}
        onClose={() => {
          setShowEditSystem(false);
          setSystemExternal(false);
        }}
        title="Edit System"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => {
                setShowEditSystem(false);
                setSystemExternal(false);
              }}
              type="button"
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={submitEditSystem} type="button">
              Save
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={systemName}
            onChange={(e) => setSystemName(e.target.value)}
            required
            placeholder="System name"
          />
          <div className="form-group" style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input
              id="edit-system-external"
              type="checkbox"
              checked={systemExternal}
              onChange={(e) => setSystemExternal(e.target.checked)}
            />
            <label htmlFor="edit-system-external">External</label>
          </div>
        </div>
      </SidePanel>

      <SidePanel
        isOpen={isPersonFormOpen}
        onClose={() => setIsPersonFormOpen(false)}
        title="Add Person"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setIsPersonFormOpen(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitAddPerson} type="button">
              Add
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={personName}
            onChange={(e) => setPersonName(e.target.value)}
            required
            placeholder="Person name"
          />
        </div>
      </SidePanel>

      <SidePanel
        isOpen={showEditPerson}
        onClose={() => setShowEditPerson(false)}
        title="Edit Person"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowEditPerson(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitEditPerson} type="button">
              Save
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={personName}
            onChange={(e) => setPersonName(e.target.value)}
            required
            placeholder="Person name"
          />
        </div>
      </SidePanel>

      <SidePanel
        isOpen={isContainerFormOpen}
        onClose={() => setIsContainerFormOpen(false)}
        title="Add Container"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setIsContainerFormOpen(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitAddContainer} type="button">
              Add
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <div className="form-group">
            <label>Parent System *</label>
            <select
              value={containerParentSystemId ?? ""}
              onChange={(e) => setContainerParentSystemId(e.target.value || null)}
              required
            >
              <option value="">Select system</option>
              {systems.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label ?? s.id}
                </option>
              ))}
            </select>
          </div>
          <Input
            label="Name *"
            value={containerName}
            onChange={(e) => setContainerName(e.target.value)}
            required
            placeholder="Container name"
          />
          <Input
            label="Technology"
            value={containerTechnology}
            onChange={(e) => setContainerTechnology(e.target.value)}
            placeholder="e.g., Node.js, Postgres"
          />
          <Textarea
            label="Description"
            value={containerDescription}
            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
              setContainerDescription(e.target.value)
            }
            rows={3}
            placeholder="Responsibilities, patterns, external dependencies"
          />
          <div className="form-group" style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input
              id="container-custom-id"
              type="checkbox"
              checked={useCustomContainerId}
              onChange={(e) => setUseCustomContainerId(e.target.checked)}
            />
            <label htmlFor="container-custom-id">Set custom ID (optional)</label>
          </div>
          {useCustomContainerId && (
            <Input
              label="ID"
              value={containerIdInput}
              onChange={(e) => setContainerIdInput(e.target.value)}
              placeholder="If empty, ID is auto-generated from name"
            />
          )}
        </div>
      </SidePanel>

      <SidePanel
        isOpen={showEditContainer}
        onClose={() => setShowEditContainer(false)}
        title="Edit Container"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowEditContainer(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitEditContainer} type="button">
              Save
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={containerName}
            onChange={(e) => setContainerName(e.target.value)}
            required
            placeholder="Container name"
          />
        </div>
      </SidePanel>

      <SidePanel
        isOpen={isComponentFormOpen}
        onClose={() => {
          setIsComponentFormOpen(false);
          setComponentParentSystemId(null);
          setComponentParentContainerId(null);
        }}
        title="Add Component"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => {
                setIsComponentFormOpen(false);
                setComponentParentSystemId(null);
                setComponentParentContainerId(null);
              }}
              type="button"
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={submitAddComponent} type="button">
              Add
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <div className="form-group">
            <label>System *</label>
            <select
              value={componentParentSystemId ?? ""}
              onChange={(e) => {
                setComponentParentSystemId(e.target.value || null);
                setComponentParentContainerId(null);
              }}
              required
            >
              <option value="">Select system</option>
              {systems.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label ?? s.id}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label>Container *</label>
            <select
              value={componentParentContainerId ?? ""}
              onChange={(e) => setComponentParentContainerId(e.target.value || null)}
              required
              disabled={!componentParentSystemId || componentSystemContainers.length === 0}
            >
              <option value="">
                {componentParentSystemId
                  ? componentSystemContainers.length === 0
                    ? "No containers available"
                    : "Select container"
                  : "Select system first"}
              </option>
              {componentSystemContainers.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.label ?? c.id}
                </option>
              ))}
            </select>
          </div>
          <Input
            label="Name *"
            value={componentName}
            onChange={(e) => setComponentName(e.target.value)}
            required
            placeholder="Component name"
          />
          <Input
            label="Technology"
            value={componentTechnology}
            onChange={(e) => setComponentTechnology(e.target.value)}
            placeholder="e.g., React, Go, Python"
          />
          <Textarea
            label="Description"
            value={componentDescription}
            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
              setComponentDescription(e.target.value)
            }
            rows={2}
            placeholder="Responsibilities, key APIs, dependencies"
          />
          <div className="form-group" style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <input
              id="component-custom-id"
              type="checkbox"
              checked={useCustomComponentId}
              onChange={(e) => setUseCustomComponentId(e.target.checked)}
            />
            <label htmlFor="component-custom-id">Set custom ID (optional)</label>
          </div>
          {useCustomComponentId && (
            <Input
              label="ID"
              value={componentIdInput}
              onChange={(e) => setComponentIdInput(e.target.value)}
              placeholder="If empty, ID is auto-generated from name"
            />
          )}
        </div>
      </SidePanel>

      <SidePanel
        isOpen={showEditComponent}
        onClose={() => setShowEditComponent(false)}
        title="Edit Component"
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={() => setShowEditComponent(false)} type="button">
              Cancel
            </Button>
            <Button variant="primary" onClick={submitEditComponent} type="button">
              Save
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={componentName}
            onChange={(e) => setComponentName(e.target.value)}
            required
            placeholder="Component name"
          />
        </div>
      </SidePanel>

      <SidePanel
        isOpen={isRelationFormOpen}
        onClose={() => {
          setIsRelationFormOpen(false);
          setRelationFromId("");
          setRelationToId("");
        }}
        title="Create Relation"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => {
                setIsRelationFormOpen(false);
                setRelationFromId("");
                setRelationToId("");
              }}
              type="button"
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={submitAddRelation} type="button">
              Create
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <div className="form-group">
            <label>From *</label>
            <select
              value={relationFromId}
              onChange={(e) => setRelationFromId(e.target.value)}
              style={{ width: "100%" }}
            >
              <option value="">Select source</option>
              {relationNodeOptions.map((opt) => (
                <option key={opt.id} value={opt.id}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label>To *</label>
            <select
              value={relationToId}
              onChange={(e) => setRelationToId(e.target.value)}
              style={{ width: "100%" }}
            >
              <option value="">Select target</option>
              {relationNodeOptions.map((opt) => (
                <option key={opt.id} value={opt.id}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <Input
            label="Label"
            value={relationLabel}
            onChange={(e) => setRelationLabel(e.target.value)}
            placeholder="e.g., uses, calls, reads"
          />
          <Input
            label="Technology"
            value={relationTech}
            onChange={(e) => setRelationTech(e.target.value)}
            placeholder="e.g., HTTP, gRPC"
          />
        </div>
      </SidePanel>
      <EditRequirementForm
        isOpen={showRequirementForm}
        onClose={() => {
          setShowRequirementForm(false);
          setEditRequirement(undefined);
        }}
        requirement={editRequirement}
      />
      <EditADRForm
        isOpen={showADRForm}
        onClose={() => {
          setShowADRForm(false);
          setEditADR(undefined);
        }}
        adr={editADR}
      />

      {/* Delete confirms */}
      <ConfirmDialog
        isOpen={showDeletePersonConfirm}
        onClose={() => setShowDeletePersonConfirm(false)}
        onConfirm={doDeletePerson}
        title="Delete Person"
        message={`Delete person "${deletePerson?.label ?? deletePerson?.id}"? This will remove related relations.`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteSystemConfirm}
        onClose={() => setShowDeleteSystemConfirm(false)}
        onConfirm={doDeleteSystem}
        title="Delete System"
        message={`Delete system "${deleteSystem?.label ?? deleteSystem?.id}" and its children? Related relations will be removed.`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteContainerConfirm}
        onClose={() => setShowDeleteContainerConfirm(false)}
        onConfirm={doDeleteContainer}
        title="Delete Container"
        message={`Delete container "${deleteContainer?.label ?? deleteContainer?.id}" and its components? Related relations will be removed.`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteComponentConfirm}
        onClose={() => setShowDeleteComponentConfirm(false)}
        onConfirm={doDeleteComponent}
        title="Delete Component"
        message={`Delete component "${deleteComponent?.label ?? deleteComponent?.id}"? Related relations will be removed.`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteRequirementConfirm}
        onClose={() => setShowDeleteRequirementConfirm(false)}
        onConfirm={async () => {
          if (!deleteRequirement) return;
          await updateArchitecture((arch) => {
            const requirements = (arch.architecture?.requirements || []).filter(
              (r) => r.id !== deleteRequirement.id
            );
            return { ...arch, architecture: { ...arch.architecture, requirements } };
          });
          setShowDeleteRequirementConfirm(false);
          setDeleteRequirement(undefined);
        }}
        title="Delete Requirement"
        message={`Delete requirement "${deleteRequirement?.id}"?`}
        confirmLabel="Delete"
        variant="danger"
      />
      <ConfirmDialog
        isOpen={showDeleteADRConfirm}
        onClose={() => setShowDeleteADRConfirm(false)}
        onConfirm={async () => {
          if (!deleteADR) return;
          await updateArchitecture((arch) => {
            const adrs = (arch.architecture?.adrs || []).filter((a) => a.id !== deleteADR.id);
            return { ...arch, architecture: { ...arch.architecture, adrs } };
          });
          setShowDeleteADRConfirm(false);
          setDeleteADR(undefined);
        }}
        title="Delete ADR"
        message={`Delete ADR "${deleteADR?.id}"?`}
        confirmLabel="Delete"
        variant="danger"
      />
    </div>
  );
}

interface SystemTreeItemProps {
  system: SystemJSON;
  isExpanded: boolean;
  isSelected: boolean;
  focusedContainerId: string | null;
  isCollapsed: boolean;
  onToggle: () => void;
  onSelect: () => void;
  onSelectContainer: (containerId: string) => void;
  onAddContainer: () => void;
  onEditSystem: () => void;
  onDeleteSystem: () => void;
  onEditContainer: (container: ContainerJSON, systemId: string) => void;
  onDeleteContainer: (container: ContainerJSON, systemId: string) => void;
}

function SystemTreeItem({
  system,
  isExpanded,
  isSelected,
  focusedContainerId,
  isCollapsed,
  onToggle,
  onSelect,
  onSelectContainer,
  onAddContainer,
  onEditSystem,
  onDeleteSystem,
  onEditContainer,
  onDeleteContainer,
}: SystemTreeItemProps) {
  const hasContainers = (system.containers?.length ?? 0) > 0;
  const hasDatastores = (system.datastores?.length ?? 0) > 0;
  const hasQueues = (system.queues?.length ?? 0) > 0;
  const hasChildren = hasContainers || hasDatastores || hasQueues;
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  return (
    <li className={`nav-item ${isSelected ? "selected" : ""}`}>
      <div className="nav-item-row">
        {/* Expand/Collapse Button */}
        {hasChildren && !isCollapsed ? (
          <button className="nav-expand-btn" onClick={onToggle}>
            {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
          </button>
        ) : (
          !isCollapsed && <span className="spacer" />
        )}

        {/* System Link */}
        <button
          className="nav-item-btn"
          onClick={onSelect}
          title={isCollapsed ? (system.label ?? system.id) : undefined}
        >
          <Building2 size={12} />
          {!isCollapsed && <span className="nav-item-label">{system.label ?? system.id}</span>}
          {hasContainers && !isCollapsed && (
            <span className="nav-item-count">{system.containers?.length}</span>
          )}
        </button>
        {!isCollapsed && isEditMode() && (
          <div className="nav-item-actions">
            <button
              className="nav-add-btn"
              onClick={(e) => {
                e.stopPropagation();
                onAddContainer();
              }}
              title="Add Container"
            >
              <Plus size={12} />
            </button>
            <button
              className="nav-edit-btn"
              onClick={(e) => {
                e.stopPropagation();
                onEditSystem();
              }}
              title="Edit System"
            >
              <Edit size={12} />
            </button>
            <button
              className="nav-delete-btn"
              onClick={(e) => {
                e.stopPropagation();
                onDeleteSystem();
              }}
              title="Delete System"
            >
              <Trash2 size={12} />
            </button>
          </div>
        )}
      </div>

      {/* Expanded Children */}
      {isExpanded && hasChildren && !isCollapsed && (
        <ul className="nav-tree nested">
          {system.containers?.map((container: ContainerJSON) => (
            <ContainerTreeItem
              key={container.id}
              container={container}
              isSelected={focusedContainerId === container.id}
              isCollapsed={isCollapsed}
              onSelect={() => onSelectContainer(container.id)}
              onEditContainer={() => onEditContainer(container, system.id)}
              onDeleteContainer={() => onDeleteContainer(container, system.id)}
            />
          ))}
          {system.datastores?.map((datastore: DataStoreJSON) => (
            <li key={datastore.id} className="nav-item">
              <button className="nav-item-btn" title={datastore.label ?? datastore.id}>
                <Database size={12} />
                {!isCollapsed && (
                  <span className="nav-item-label">{datastore.label ?? datastore.id}</span>
                )}
              </button>
            </li>
          ))}
          {system.queues?.map((queue: QueueJSON) => (
            <li key={queue.id} className="nav-item">
              <button className="nav-item-btn" title={queue.label ?? queue.id}>
                <MessageSquare size={12} />
                {!isCollapsed && <span className="nav-item-label">{queue.label ?? queue.id}</span>}
              </button>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}

interface ContainerTreeItemProps {
  container: ContainerJSON;
  isSelected: boolean;
  isCollapsed: boolean;
  onSelect: () => void;
  onEditContainer?: () => void;
  onDeleteContainer?: () => void;
}

function ContainerTreeItem({
  container,
  isSelected,
  isCollapsed,
  onSelect,
  onEditContainer,
  onDeleteContainer,
}: ContainerTreeItemProps) {
  const hasComponents = (container.components?.length ?? 0) > 0;
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
  const [isComponentFormOpenLocal, setIsComponentFormOpenLocal] = useState(false);
  const [componentNameLocal, setComponentNameLocal] = useState("");
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const submitLocalAddComponent = async () => {
    const id =
      componentNameLocal
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9_-]+/g, "-") || "component";
    await updateArchitecture((arch) => {
      const systems = (arch.architecture?.systems || []).map((s) => {
        const containers = (s.containers || []).map((c) => {
          if (c.id !== container.id) return c;
          const components = [...(c.components || [])];
          components.push({ id, label: componentNameLocal });
          return { ...c, components };
        });
        return { ...s, containers };
      });
      return { ...arch, architecture: { ...arch.architecture, systems } };
    });
    setComponentNameLocal("");
    setIsComponentFormOpenLocal(false);
  };

  return (
    <li className={`nav-item ${isSelected ? "selected" : ""}`}>
      <button
        className="nav-item-btn"
        onClick={onSelect}
        title={isCollapsed ? (container.label ?? container.id) : undefined}
      >
        <Box size={12} />
        {!isCollapsed && <span className="nav-item-label">{container.label ?? container.id}</span>}
        {hasComponents && <span className="nav-item-count">{container.components?.length}</span>}
      </button>
      {!isCollapsed && isEditMode() && (
        <div className="nav-item-actions">
          <button
            className="nav-add-btn"
            onClick={(e) => {
              e.stopPropagation();
              setIsComponentFormOpenLocal(true);
            }}
            title="Add Component"
          >
            <Plus size={12} />
          </button>
          {onEditContainer && (
            <button
              className="nav-edit-btn"
              onClick={(e) => {
                e.stopPropagation();
                onEditContainer();
              }}
              title="Edit Container"
            >
              <Edit size={12} />
            </button>
          )}
          {onDeleteContainer && (
            <button
              className="nav-delete-btn"
              onClick={(e) => {
                e.stopPropagation();
                onDeleteContainer();
              }}
              title="Delete Container"
            >
              <Trash2 size={12} />
            </button>
          )}
        </div>
      )}
      <SidePanel
        isOpen={isComponentFormOpenLocal}
        onClose={() => setIsComponentFormOpenLocal(false)}
        title="Add Component"
        size="md"
        footer={
          <>
            <Button
              variant="secondary"
              onClick={() => setIsComponentFormOpenLocal(false)}
              type="button"
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={submitLocalAddComponent} type="button">
              Add
            </Button>
          </>
        }
      >
        <div className="edit-form">
          <Input
            label="Name *"
            value={componentNameLocal}
            onChange={(e) => setComponentNameLocal(e.target.value)}
            required
            placeholder="Component name"
          />
        </div>
      </SidePanel>
    </li>
  );
}

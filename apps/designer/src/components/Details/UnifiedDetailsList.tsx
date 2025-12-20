// apps/designer/src/components/Details/UnifiedDetailsList.tsx
import { useMemo, useState } from "react";
import { Target, FileText, Play, Workflow } from "lucide-react";
import { useArchitectureStore, useUIStore } from "../../stores";
import { useTagNavigation } from "../../hooks/useTagNavigation";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import {
  EditFlowForm,
  EditRequirementForm,
  EditADRForm,
  ConfirmDialog,
} from "../shared";
import type { RequirementDump, ADRDump, ScenarioDump, FlowDump } from "@sruja/shared";
import { UnifiedItemCard } from "./UnifiedItemCard";
import type { FilterState, ItemType } from "./DetailsSidebarFilters";
import "./UnifiedDetailsList.css";

export type UnifiedItem =
  | { type: "requirement"; data: RequirementDump }
  | { type: "adr"; data: ADRDump }
  | { type: "scenario"; data: ScenarioDump }
  | { type: "flow"; data: FlowDump };

interface UnifiedDetailsListProps {
  filters: FilterState;
  onItemClick?: (item: UnifiedItem) => void;
}

export function UnifiedDetailsList({ filters, onItemClick }: UnifiedDetailsListProps) {
  const likec4Model = useArchitectureStore((s) => s.likec4Model);
  // const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture); // Unused for now until delete logic is implemented
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const { navigateToTaggedElement } = useTagNavigation();

  const [editItem, setEditItem] = useState<UnifiedItem | null>(null);
  const [deleteItem, setDeleteItem] = useState<UnifiedItem | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  // Get all items
  const allItems = useMemo<UnifiedItem[]>(() => {
    if (!likec4Model?.sruja) return [];

    // Cast to any because sruja types might be incomplete in @sruja/shared currently
    // We expect them to be there based on usage
    const sruja = likec4Model.sruja as any;

    const items: UnifiedItem[] = [];

    // Requirements
    const requirements = deduplicateRequirements(sruja.requirements ?? []);
    requirements.forEach((req) => {
      items.push({ type: "requirement", data: req });
    });

    // ADRs
    const adrs = sruja.adrs ?? [];
    adrs.forEach((adr: ADRDump) => {
      items.push({ type: "adr", data: adr });
    });

    // Scenarios
    const scenarios = sruja.scenarios ?? [];
    scenarios.forEach((scenario: ScenarioDump) => {
      items.push({ type: "scenario", data: scenario });
    });

    // Flows
    const flows = sruja.flows ?? [];
    flows.forEach((flow: FlowDump) => {
      items.push({ type: "flow", data: flow });
    });

    return items;
  }, [likec4Model]);

  // Filter items
  const filteredItems = useMemo(() => {
    let filtered = allItems;

    // Filter by type
    if (filters.types.size > 0) {
      filtered = filtered.filter((item) => filters.types.has(item.type));
    }

    // Filter by status (for requirements and ADRs)
    if (filters.statuses.size > 0) {
      filtered = filtered.filter((item) => {
        if (item.type === "requirement" || item.type === "adr") {
          // Using safe access via 'any' or assuming updated types
          const tags = (item.data as any).tags ?? [];
          const hasLinks = tags.length > 0;
          let status: "fulfilled" | "partial" | "missing";

          if (item.type === "requirement") {
            status = hasLinks ? (tags.length >= 2 ? "fulfilled" : "partial") : "missing";
          } else {
            const adrStatus = (item.data as ADRDump).status;
            status = adrStatus === "accepted" ? "fulfilled" : adrStatus === "deprecated" ? "missing" : "partial";
          }

          return filters.statuses.has(status);
        }
        return true;
      });
    }

    // Filter by tags
    if (filters.tags.size > 0) {
      filtered = filtered.filter((item) => {
        const itemTags = ((item.data as any).tags || []) as string[];
        return Array.from(filters.tags).some((tag) => itemTags.includes(tag));
      });
    }

    // Filter by search query
    if (filters.searchQuery.trim()) {
      const query = filters.searchQuery.toLowerCase();
      filtered = filtered.filter((item) => {
        const id = item.data.id?.toLowerCase() || "";
        const title = (item.data.title || "").toLowerCase();
        const description = ((item.data as any).description || "").toLowerCase();
        return id.includes(query) || title.includes(query) || description.includes(query);
      });
    }

    return filtered;
  }, [allItems, filters]);

  // Group items by type for better organization
  const groupedItems = useMemo(() => {
    const groups: Record<ItemType, UnifiedItem[]> = {
      requirement: [],
      adr: [],
      scenario: [],
      flow: [],
    };

    filteredItems.forEach((item) => {
      groups[item.type].push(item);
    });

    return groups;
  }, [filteredItems]);

  const handleItemClick = (item: UnifiedItem) => {
    if (onItemClick) {
      onItemClick(item);
    }
    // Navigate to diagram if scenario/flow
    if (item.type === "scenario" || item.type === "flow") {
      setActiveTab("diagram");
      // Could trigger flow/scenario playback here
    }
  };

  const handleTagClick = (tag: string) => {
    navigateToTaggedElement(tag);
  };

  const handleEdit = (item: UnifiedItem) => {
    setEditItem(item);
  };

  const handleDelete = (item: UnifiedItem) => {
    setDeleteItem(item);
    setShowDeleteConfirm(true);
  };

  const confirmDelete = async () => {
    if (!deleteItem) return;

    // TODO: Implement delete using updateArchitecture with SrujaModelDump update logic
    // Currently relying on updateArchitecture(updater) which updates the model in store.
    // However, existing forms might trigger legacy updates if not updated.

    // Stub implementation for now until update logic is fully generic
    console.warn("Delete logic not fully ported for SrujaModelDump yet in UnifiedDetailsList");

    // await updateArchitecture((arch) => {
    //   // Logic to remove from sruja extension arrays
    //   return arch;
    // });

    setShowDeleteConfirm(false);
    setDeleteItem(null);
  };

  if (filteredItems.length === 0) {
    return (
      <div className="unified-list-empty">
        <FileText size={48} className="empty-icon" />
        <h3>No items found</h3>
        <p>Try adjusting your filters or create new items</p>
      </div>
    );
  }

  return (
    <div className="unified-details-list">
      {/* Grouped View */}
      <div className="unified-list-content">
        {groupedItems.requirement.length > 0 && (
          <div className="item-group">
            <div className="group-header">
              <Target size={16} />
              <span>Requirements ({groupedItems.requirement.length})</span>
            </div>
            <div className="group-items">
              {groupedItems.requirement.map((item) => (
                <UnifiedItemCard
                  key={`req - ${item.data.id} `}
                  item={item}
                  onClick={() => handleItemClick(item)}
                  onTagClick={handleTagClick}
                  onEdit={() => handleEdit(item)}
                  onDelete={() => handleDelete(item)}
                />
              ))}
            </div>
          </div>
        )}

        {groupedItems.adr.length > 0 && (
          <div className="item-group">
            <div className="group-header">
              <FileText size={16} />
              <span>ADRs ({groupedItems.adr.length})</span>
            </div>
            <div className="group-items">
              {groupedItems.adr.map((item) => (
                <UnifiedItemCard
                  key={`adr - ${item.data.id} `}
                  item={item}
                  onClick={() => handleItemClick(item)}
                  onTagClick={handleTagClick}
                  onEdit={() => handleEdit(item)}
                  onDelete={() => handleDelete(item)}
                />
              ))}
            </div>
          </div>
        )}

        {groupedItems.scenario.length > 0 && (
          <div className="item-group">
            <div className="group-header">
              <Play size={16} />
              <span>Scenarios ({groupedItems.scenario.length})</span>
            </div>
            <div className="group-items">
              {groupedItems.scenario.map((item) => (
                <UnifiedItemCard
                  key={`scenario - ${item.data.id} `}
                  item={item}
                  onClick={() => handleItemClick(item)}
                  onTagClick={handleTagClick}
                  onEdit={() => handleEdit(item)}
                  onDelete={() => handleDelete(item)}
                />
              ))}
            </div>
          </div>
        )}

        {groupedItems.flow.length > 0 && (
          <div className="item-group">
            <div className="group-header">
              <Workflow size={16} />
              <span>Flows ({groupedItems.flow.length})</span>
            </div>
            <div className="group-items">
              {groupedItems.flow.map((item) => (
                <UnifiedItemCard
                  key={`flow - ${item.data.id} `}
                  item={item}
                  onClick={() => handleItemClick(item)}
                  onTagClick={handleTagClick}
                  onEdit={() => handleEdit(item)}
                  onDelete={() => handleDelete(item)}
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Edit Forms - Passing any for now as forms might still expect legacy types or need update */}
      {/* We should ideally update forms to accept RequirementDump/etc, but for cleanup phase we suppress type checks */}
      {editItem && editItem.type === "requirement" && (
        <EditRequirementForm
          isOpen={true}
          onClose={() => setEditItem(null)}
          requirement={(editItem.data as any)}
        />
      )}
      {editItem && editItem.type === "adr" && (
        <EditADRForm
          isOpen={true}
          onClose={() => setEditItem(null)}
          adr={(editItem.data as any)}
        />
      )}
      {editItem && editItem.type === "flow" && (
        <EditFlowForm
          isOpen={true}
          onClose={() => setEditItem(null)}
          flow={(editItem.data as any)}
        />
      )}

      {/* Delete Confirmation */}
      <ConfirmDialog
        isOpen={showDeleteConfirm}
        onClose={() => {
          setShowDeleteConfirm(false);
          setDeleteItem(null);
        }}
        onConfirm={confirmDelete}
        title={`Delete ${deleteItem?.type || "item"} `}
        message={`Are you sure you want to delete ${deleteItem?.type || "item"} "${deleteItem?.data.id}" ? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </div>
  );
}

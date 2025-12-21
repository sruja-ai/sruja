// apps/designer/src/components/Details/DetailsSidebarFilters.tsx
// useState not currently used
import { Target, FileText, Play, Workflow, CheckCircle, AlertCircle, XCircle } from "lucide-react";
import { Button } from "@sruja/ui";
import "./DetailsSidebarFilters.css";

export type ItemType = "requirement" | "adr" | "scenario" | "flow";
export type ItemStatus = "fulfilled" | "partial" | "missing";

export interface FilterState {
  types: Set<ItemType>;
  statuses: Set<ItemStatus>;
  tags: Set<string>;
  searchQuery: string;
}

interface DetailsSidebarFiltersProps {
  filters: FilterState;
  onFilterChange: (filters: FilterState) => void;
  availableTags: string[];
  stats: {
    requirements: number;
    adrs: number;
    scenarios: number;
    flows: number;
  };
}

export function DetailsSidebarFilters({
  filters,
  onFilterChange,
  availableTags,
  stats,
}: DetailsSidebarFiltersProps) {
  const toggleType = (type: ItemType) => {
    const newTypes = new Set(filters.types);
    if (newTypes.has(type)) {
      newTypes.delete(type);
    } else {
      newTypes.add(type);
    }
    onFilterChange({ ...filters, types: newTypes });
  };

  const toggleStatus = (status: ItemStatus) => {
    const newStatuses = new Set(filters.statuses);
    if (newStatuses.has(status)) {
      newStatuses.delete(status);
    } else {
      newStatuses.add(status);
    }
    onFilterChange({ ...filters, statuses: newStatuses });
  };

  const toggleTag = (tag: string) => {
    const newTags = new Set(filters.tags);
    if (newTags.has(tag)) {
      newTags.delete(tag);
    } else {
      newTags.add(tag);
    }
    onFilterChange({ ...filters, tags: newTags });
  };

  const clearFilters = () => {
    onFilterChange({
      types: new Set(),
      statuses: new Set(),
      tags: new Set(),
      searchQuery: "",
    });
  };

  const hasActiveFilters =
    filters.types.size > 0 || filters.statuses.size > 0 || filters.tags.size > 0 || filters.searchQuery.trim() !== "";

  return (
    <div className="details-sidebar-filters">
      <div className="filters-header">
        <h3>Filters</h3>
        {hasActiveFilters && (
          <Button variant="ghost" size="sm" className="clear-filters-btn" onClick={clearFilters} title="Clear all filters">
            Clear
          </Button>
        )}
      </div>

      {/* Type Filter */}
      <div className="filter-section">
        <div className="filter-section-title">Type</div>
        <div className="filter-options">
          <Button
            variant={filters.types.has("requirement") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option ${filters.types.has("requirement") ? "active" : ""}`}
            onClick={() => toggleType("requirement")}
          >
            <Target size={14} />
            <span>Requirements</span>
            <span className="filter-count">{stats.requirements}</span>
          </Button>
          <Button
            variant={filters.types.has("adr") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option ${filters.types.has("adr") ? "active" : ""}`}
            onClick={() => toggleType("adr")}
          >
            <FileText size={14} />
            <span>ADRs</span>
            <span className="filter-count">{stats.adrs}</span>
          </Button>
          <Button
            variant={filters.types.has("scenario") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option ${filters.types.has("scenario") ? "active" : ""}`}
            onClick={() => toggleType("scenario")}
          >
            <Play size={14} />
            <span>Scenarios</span>
            <span className="filter-count">{stats.scenarios}</span>
          </Button>
          <Button
            variant={filters.types.has("flow") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option ${filters.types.has("flow") ? "active" : ""}`}
            onClick={() => toggleType("flow")}
          >
            <Workflow size={14} />
            <span>Flows</span>
            <span className="filter-count">{stats.flows}</span>
          </Button>
        </div>
      </div>

      {/* Status Filter */}
      <div className="filter-section">
        <div className="filter-section-title">Status</div>
        <div className="filter-options">
          <Button
            variant={filters.statuses.has("fulfilled") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option status fulfilled ${filters.statuses.has("fulfilled") ? "active" : ""}`}
            onClick={() => toggleStatus("fulfilled")}
          >
            <CheckCircle size={14} />
            <span>Fulfilled</span>
          </Button>
          <Button
            variant={filters.statuses.has("partial") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option status partial ${filters.statuses.has("partial") ? "active" : ""}`}
            onClick={() => toggleStatus("partial")}
          >
            <AlertCircle size={14} />
            <span>Partial</span>
          </Button>
          <Button
            variant={filters.statuses.has("missing") ? "primary" : "ghost"}
            size="sm"
            className={`filter-option status missing ${filters.statuses.has("missing") ? "active" : ""}`}
            onClick={() => toggleStatus("missing")}
          >
            <XCircle size={14} />
            <span>Missing</span>
          </Button>
        </div>
      </div>

      {/* Tags Filter */}
      {availableTags.length > 0 && (
        <div className="filter-section">
          <div className="filter-section-title">Tags</div>
          <div className="filter-tags">
            {availableTags.slice(0, 10).map((tag) => (
              <Button
                key={tag}
                variant={filters.tags.has(tag) ? "primary" : "ghost"}
                size="sm"
                className={`filter-tag ${filters.tags.has(tag) ? "active" : ""}`}
                onClick={() => toggleTag(tag)}
              >
                {tag}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Quick Stats */}
      <div className="filter-section stats-section">
        <div className="filter-section-title">Quick Stats</div>
        <div className="stats-grid">
          <div className="stat-item">
            <span className="stat-label">Total</span>
            <span className="stat-value">
              {stats.requirements + stats.adrs + stats.scenarios + stats.flows}
            </span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Requirements</span>
            <span className="stat-value">{stats.requirements}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">ADRs</span>
            <span className="stat-value">{stats.adrs}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

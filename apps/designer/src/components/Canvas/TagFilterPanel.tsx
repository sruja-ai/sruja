// apps/designer/src/components/Canvas/TagFilterPanel.tsx
import { useState } from "react";
import { Filter, X, Target, FileText, Play, Workflow } from "lucide-react";
import { Button } from "@sruja/ui";
import "./TagFilterPanel.css";

export type TagFilterType = "requirements" | "adrs" | "scenarios" | "flows" | null;

interface TagFilterPanelProps {
  filters: Set<TagFilterType>;
  onFilterChange: (filters: Set<TagFilterType>) => void;
}

export function TagFilterPanel({ filters, onFilterChange }: TagFilterPanelProps) {
  const [isOpen, setIsOpen] = useState(false);

  const toggleFilter = (filterType: TagFilterType) => {
    const newFilters = new Set(filters);
    if (newFilters.has(filterType)) {
      newFilters.delete(filterType);
    } else {
      newFilters.add(filterType);
    }
    onFilterChange(newFilters);
  };

  const clearFilters = () => {
    onFilterChange(new Set());
  };

  const hasActiveFilters = filters.size > 0;

  return (
    <div className={`tag-filter-panel ${isOpen ? "open" : ""}`}>
      <Button
        variant="ghost"
        size="sm"
        className="filter-toggle-btn"
        onClick={() => setIsOpen(!isOpen)}
        title={isOpen ? "Hide filters" : "Show filters"}
        aria-label={isOpen ? "Hide filters" : "Show filters"}
      >
        <Filter size={16} />
        {hasActiveFilters && <span className="filter-badge">{filters.size}</span>}
      </Button>

      {isOpen && (
        <div className="filter-content">
          <div className="filter-header">
            <h3>Filter by Linked Items</h3>
            {hasActiveFilters && (
              <Button variant="ghost" size="sm" className="clear-filters-btn" onClick={clearFilters} title="Clear all filters">
                <X size={14} />
                Clear
              </Button>
            )}
          </div>

          <div className="filter-options">
            <Button
              variant={filters.has("requirements") ? "primary" : "ghost"}
              size="sm"
              className={`filter-option ${filters.has("requirements") ? "active" : ""}`}
              onClick={() => toggleFilter("requirements")}
            >
              <Target size={14} />
              <span>Requirements</span>
              {filters.has("requirements") && <span className="check">✓</span>}
            </Button>

            <Button
              variant={filters.has("adrs") ? "primary" : "ghost"}
              size="sm"
              className={`filter-option ${filters.has("adrs") ? "active" : ""}`}
              onClick={() => toggleFilter("adrs")}
            >
              <FileText size={14} />
              <span>ADRs</span>
              {filters.has("adrs") && <span className="check">✓</span>}
            </Button>

            <Button
              variant={filters.has("scenarios") ? "primary" : "ghost"}
              size="sm"
              className={`filter-option ${filters.has("scenarios") ? "active" : ""}`}
              onClick={() => toggleFilter("scenarios")}
            >
              <Play size={14} />
              <span>Scenarios</span>
              {filters.has("scenarios") && <span className="check">✓</span>}
            </Button>

            <Button
              variant={filters.has("flows") ? "primary" : "ghost"}
              size="sm"
              className={`filter-option ${filters.has("flows") ? "active" : ""}`}
              onClick={() => toggleFilter("flows")}
            >
              <Workflow size={14} />
              <span>Flows</span>
              {filters.has("flows") && <span className="check">✓</span>}
            </Button>
          </div>

          {hasActiveFilters && (
            <div className="filter-info">
              Showing only elements with: {Array.from(filters).join(", ")}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

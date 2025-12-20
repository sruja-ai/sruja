// apps/designer/src/components/Views/DetailsView.tsx
import { useState, useEffect, useMemo } from "react";
import { Search } from "lucide-react";
import { useArchitectureStore } from "../../stores";
import { DetailsSidebarFilters, type FilterState } from "../Details/DetailsSidebarFilters";
import { UnifiedDetailsList } from "../Details/UnifiedDetailsList";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import { Input } from "@sruja/ui";
import "./DetailsView.css";

export function DetailsView() {
  const likec4Model = useArchitectureStore((s) => s.likec4Model);
  const [filters, setFilters] = useState<FilterState>({
    types: new Set(),
    statuses: new Set(),
    tags: new Set(),
    searchQuery: "",
  });

  // Listen for sub-tab switch events from diagram badges
  useEffect(() => {
    const handleSubTabSwitch = (event: CustomEvent<"requirements" | "adrs" | "scenarios" | "flows">) => {
      const tab = event.detail;
      // Set filter to show only the selected type
      setFilters((prev) => ({
        ...prev,
        types: new Set([tab === "requirements" ? "requirement" : tab === "adrs" ? "adr" : tab === "scenarios" ? "scenario" : "flow"]),
      }));
    };

    window.addEventListener("switch-details-subtab", handleSubTabSwitch as EventListener);
    return () => {
      window.removeEventListener("switch-details-subtab", handleSubTabSwitch as EventListener);
    };
  }, []);

  const sruja = (likec4Model?.sruja as any) || {};

  const scenarios = sruja.scenarios ?? [];
  const flows = sruja.flows ?? [];
  const allRequirements = sruja.requirements ?? [];
  const adrs = sruja.adrs ?? [];

  // Deduplicate requirements to match what UnifiedDetailsList displays
  const requirements = useMemo(() => deduplicateRequirements(allRequirements), [allRequirements]);

  // Collect all unique tags
  const availableTags = useMemo(() => {
    const tagSet = new Set<string>();
    requirements.forEach((r: any) => (r.tags ?? []).forEach((t: string) => tagSet.add(t)));
    adrs.forEach((a: any) => (a.tags ?? []).forEach((t: string) => tagSet.add(t)));
    scenarios.forEach((s: any) => (s.steps ?? []).forEach((step: any) => (step.tags ?? []).forEach((t: string) => tagSet.add(t))));
    flows.forEach((f: any) => (f.steps ?? []).forEach((step: any) => (step.tags ?? []).forEach((t: string) => tagSet.add(t))));
    return Array.from(tagSet).sort();
  }, [requirements, adrs, scenarios, flows]);

  const stats = {
    requirements: requirements.length,
    adrs: adrs.length,
    scenarios: scenarios.length,
    flows: flows.length,
  };

  return (
    <div className="details-view-unified">
      {/* Search Bar */}
      <div className="details-search-bar">
        <div className="search-input-wrapper">
          <Search size={16} className="search-icon" />
          <Input
            placeholder="Search requirements, ADRs, scenarios, flows..."
            value={filters.searchQuery}
            onChange={(e) => setFilters({ ...filters, searchQuery: e.target.value })}
            className="details-search-input"
          />
        </div>
      </div>

      {/* Main Content */}
      <div className="details-content-unified">
        {/* Sidebar Filters */}
        <DetailsSidebarFilters
          filters={filters}
          onFilterChange={setFilters}
          availableTags={availableTags}
          stats={stats}
        />

        {/* Unified List */}
        <UnifiedDetailsList filters={filters} />
      </div>
    </div>
  );
}

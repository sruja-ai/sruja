// apps/playground/src/components/ViewTabs.tsx
import { Layout, FileCode, List, Hammer } from "lucide-react";
import type { ViewTab } from "../types";
import { useFeatureFlagsStore } from "../stores";

interface ViewTabsProps {
  activeTab: ViewTab;
  onTabChange: (tab: ViewTab) => void;
  counts: {
    requirements: number;
    adrs: number;
  };
}

export function ViewTabs({ activeTab, onTabChange, counts }: ViewTabsProps) {
  const editMode = useFeatureFlagsStore((s) => s.editMode);

  // Builder is now first and always visible in edit mode
  const tabs: ViewTab[] =
    editMode === "edit"
      ? ["builder", "diagram", "details", "code"]
      : ["diagram", "details", "code"];

  const index = tabs.indexOf(activeTab);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowRight") {
      const next = tabs[(index + 1) % tabs.length];
      onTabChange(next);
      e.preventDefault();
    } else if (e.key === "ArrowLeft") {
      const prev = tabs[(index - 1 + tabs.length) % tabs.length];
      onTabChange(prev);
      e.preventDefault();
    }
  };

  return (
    <div
      className="view-tabs"
      role="tablist"
      aria-label="View tabs"
      tabIndex={0}
      onKeyDown={handleKeyDown}
    >
      {editMode === "edit" && (
        <button
          className={`view-tab view-tab-primary ${activeTab === "builder" ? "active" : ""}`}
          onClick={() => onTabChange("builder")}
          role="tab"
          aria-selected={activeTab === "builder"}
          id="tab-builder"
          aria-controls="tabpanel-builder"
          title="Builder - Step-by-step architecture design guide"
        >
          <Hammer size={16} />
          <span>Builder</span>
        </button>
      )}
      <button
        className={`view-tab ${activeTab === "diagram" ? "active" : ""}`}
        onClick={() => onTabChange("diagram")}
        role="tab"
        aria-selected={activeTab === "diagram"}
        id="tab-diagram"
        aria-controls="tabpanel-diagram"
        title="Diagram - Visual architecture diagram and layout"
      >
        <Layout size={16} />
        <span>Diagram</span>
      </button>
      <button
        className={`view-tab ${activeTab === "details" ? "active" : ""}`}
        onClick={() => onTabChange("details")}
        role="tab"
        aria-selected={activeTab === "details"}
        id="tab-details"
        aria-controls="tabpanel-details"
        title="Details - Requirements, ADRs, scenarios, and flows"
      >
        <List size={16} />
        <span>Details</span>
        {counts.requirements + counts.adrs > 0 && (
          <span className="tab-badge">{counts.requirements + counts.adrs}</span>
        )}
      </button>
      <button
        className={`view-tab ${activeTab === "code" ? "active" : ""}`}
        onClick={() => onTabChange("code")}
        role="tab"
        aria-selected={activeTab === "code"}
        id="tab-code"
        aria-controls="tabpanel-code"
        title="Code - View and edit Sruja DSL source code"
      >
        <FileCode size={16} />
        <span>Code</span>
      </button>
    </div>
  );
}

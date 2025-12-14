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

  // Builder (guided) is now first and always visible in edit mode
  const tabs: ViewTab[] =
    editMode === "edit" ? ["guided", "diagram", "details", "code"] : ["diagram", "details", "code"];

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
          className={`view-tab view-tab-primary ${activeTab === "guided" ? "active" : ""}`}
          onClick={() => onTabChange("guided")}
          role="tab"
          aria-selected={activeTab === "guided"}
          title="Builder - Design your architecture step by step"
        >
          <Hammer size={16} />
          Builder
        </button>
      )}
      <button
        className={`view-tab ${activeTab === "diagram" ? "active" : ""}`}
        onClick={() => onTabChange("diagram")}
        role="tab"
        aria-selected={activeTab === "diagram"}
        title="Diagram"
      >
        <Layout size={16} />
        Diagram
      </button>
      <button
        className={`view-tab ${activeTab === "details" ? "active" : ""}`}
        onClick={() => onTabChange("details")}
        role="tab"
        aria-selected={activeTab === "details"}
        title="Details"
      >
        <List size={16} />
        Details
        {counts.requirements + counts.adrs > 0 && (
          <span className="tab-badge">{counts.requirements + counts.adrs}</span>
        )}
      </button>
      <button
        className={`view-tab ${activeTab === "code" ? "active" : ""}`}
        onClick={() => onTabChange("code")}
        role="tab"
        aria-selected={activeTab === "code"}
        title="Code"
      >
        <FileCode size={16} />
        Code
      </button>
    </div>
  );
}

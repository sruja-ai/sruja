import { Layout, FileCode, List, Hammer, Users, Home } from "lucide-react";
import { Button } from "@sruja/ui";
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

  // Tabs order: Frequency-based - Overview, Diagram, Code (most used), then Builder, Details, Roles
  const tabs: ViewTab[] = ["overview", "diagram", "code", "builder", "details", "roles"];

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
      <Button
        variant={activeTab === "overview" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "overview" ? "active" : ""}`}
        onClick={() => onTabChange("overview")}
        role="tab"
        aria-selected={activeTab === "overview"}
        id="tab-overview"
        data-testid="tab-overview"
        aria-controls="tabpanel-overview"
        title="Overview - Architecture summary and quick navigation"
      >
        <div className="view-tab-content">
          <Home size={16} />
          <span>Overview</span>
        </div>
      </Button>
      <Button
        variant={
          activeTab === "builder" && editMode === "edit"
            ? "primary"
            : activeTab === "builder"
              ? "secondary"
              : "ghost"
        }
        size="sm"
        className={`view-tab ${editMode === "edit" ? "view-tab-primary" : ""} ${activeTab === "builder" ? "active" : ""}`}
        onClick={() => onTabChange("builder")}
        role="tab"
        aria-selected={activeTab === "builder"}
        id="tab-builder"
        data-testid="tab-builder"
        aria-controls="tabpanel-builder"
        title={
          editMode === "edit"
            ? "Builder - Step-by-step architecture design guide"
            : "Builder - Architecture guide (view mode)"
        }
      >
        <div className="view-tab-content">
          <Hammer size={16} />
          <span>Builder</span>
          {editMode === "edit" && <span className="tab-badge edit-badge">Edit</span>}
        </div>
      </Button>
      <Button
        variant={activeTab === "diagram" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "diagram" ? "active" : ""}`}
        onClick={() => onTabChange("diagram")}
        role="tab"
        aria-selected={activeTab === "diagram"}
        id="tab-diagram"
        data-testid="tab-diagram"
        aria-controls="tabpanel-diagram"
        title="Diagram - Visual architecture diagram and layout"
      >
        <div className="view-tab-content">
          <Layout size={16} />
          <span>Diagram</span>
        </div>
      </Button>
      <Button
        variant={activeTab === "details" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "details" ? "active" : ""}`}
        onClick={() => onTabChange("details")}
        role="tab"
        aria-selected={activeTab === "details"}
        id="tab-details"
        data-testid="tab-details"
        aria-controls="tabpanel-details"
        title="Details - Requirements, ADRs, scenarios, and flows"
      >
        <div className="view-tab-content">
          <List size={16} />
          <span>Details</span>
          {counts.requirements + counts.adrs > 0 && (
            <span className="tab-badge">{counts.requirements + counts.adrs}</span>
          )}
        </div>
      </Button>
      <Button
        variant={activeTab === "code" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "code" ? "active" : ""}`}
        onClick={() => onTabChange("code")}
        role="tab"
        aria-selected={activeTab === "code"}
        id="tab-code"
        data-testid="tab-code"
        aria-controls="tabpanel-code"
        title="Code - View and edit Sruja DSL source code"
      >
        <div className="view-tab-content">
          <FileCode size={16} />
          <span>Code</span>
        </div>
      </Button>
      <Button
        variant={activeTab === "roles" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "roles" ? "active" : ""}`}
        onClick={() => onTabChange("roles")}
        role="tab"
        aria-selected={activeTab === "roles"}
        id="tab-roles"
        data-testid="tab-roles"
        aria-controls="tabpanel-roles"
        title="Roles - View architecture through different role perspectives"
      >
        <div className="view-tab-content">
          <Users size={16} />
          <span>Roles</span>
        </div>
      </Button>
    </div>
  );
}

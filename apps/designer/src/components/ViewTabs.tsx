import { Layout, FileCode, FileText } from "lucide-react";
import { Button } from "@sruja/ui";
import type { ViewTab } from "../types";
import "./ViewTabs.css";

interface ViewTabsProps {
  activeTab: ViewTab;
  onTabChange: (tab: ViewTab) => void;
}

/**
 * ViewTabs - Three parent tabs: Diagram, Code, Docs
 * No sub-tabs for simplicity
 */
export function ViewTabs({ activeTab, onTabChange }: ViewTabsProps) {
  const tabs: { id: ViewTab; icon: React.ReactNode; label: string }[] = [
    { id: "diagram", icon: <Layout size={16} />, label: "Diagram" },
    { id: "code", icon: <FileCode size={16} />, label: "Code" },
    { id: "docs", icon: <FileText size={16} />, label: "Docs" },
  ];

  return (
    <div className="view-tabs view-tabs-simple" role="tablist" aria-label="View tabs">
      {tabs.map((tab) => (
        <Button
          key={tab.id}
          variant={activeTab === tab.id ? "secondary" : "ghost"}
          size="sm"
          className={`view-tab ${activeTab === tab.id ? "active" : ""}`}
          onClick={() => onTabChange(tab.id)}
          role="tab"
          aria-selected={activeTab === tab.id}
          id={`tab-${tab.id}`}
          data-testid={`tab-${tab.id}`}
          aria-controls={`tabpanel-${tab.id}`}
        >
          <div className="view-tab-content">
            {tab.icon}
            <span>{tab.label}</span>
          </div>
        </Button>
      ))}
    </div>
  );
}

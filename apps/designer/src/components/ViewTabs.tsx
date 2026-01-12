import { Workflow, FileCode, FileText, Hammer } from "lucide-react";
import { Button } from "@sruja/ui";
import type { ViewTab } from "../types";
import "./ViewTabs.css";

interface ViewTabsProps {
  activeId: string | null;
  onTabChange: (id: string) => void;
  tabs: { id: string; icon: React.ReactNode; label: string }[];
  className?: string;
}

/**
 * ViewTabs - Generic tab strip component
 */
export function ViewTabs({ activeId, onTabChange, tabs, className = "" }: ViewTabsProps) {
  return (
    <div
      className={`view-tabs view-tabs-simple ${className}`}
      role="tablist"
      aria-label="View tabs"
    >
      {tabs.map((tab) => (
        <Button
          key={tab.id}
          variant={activeId === tab.id ? "secondary" : "ghost"}
          size="sm"
          className={`view-tab ${activeId === tab.id ? "active" : ""}`}
          onClick={() => onTabChange(tab.id)}
          role="tab"
          aria-selected={activeId === tab.id}
          id={`tab-${tab.id}`}
          data-testid={`tab-${tab.id}`}
          aria-controls={`tabpanel-${tab.id}`}
          title={tab.label}
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

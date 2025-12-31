import { useEffect, useState } from "react";
import { Layout, FileCode, List, Hammer, Shield } from "lucide-react";
import { Button } from "@sruja/ui";
import type { ViewTab } from "../types";
import { useFeatureFlagsStore, useArchitectureStore } from "../stores";
import { getWasmApi, logger } from "@sruja/shared";

interface ViewTabsProps {
  activeTab: ViewTab;
  onTabChange: (tab: ViewTab) => void;
  counts: {
    requirements: number;
    adrs: number;
  };
}

interface ScoreCard {
  Score: number;
  Grade: string;
}

export function ViewTabs({ activeTab, onTabChange, counts }: ViewTabsProps) {
  const editMode = useFeatureFlagsStore((s) => s.editMode);
  const dslSource = useArchitectureStore((s) => s.dslSource);
  const [scoreCard, setScoreCard] = useState<ScoreCard | null>(null);

  // Calculate score when DSL changes
  useEffect(() => {
    const calculateScore = async () => {
      if (!dslSource) {
        setScoreCard(null);
        return;
      }

      try {
        const api = await getWasmApi();
        if (!api) {
          setScoreCard(null);
          return;
        }

        // Validate DSL by attempting to parse it first
        // This prevents score calculation errors from invalid syntax
        try {
          await api.dslToModel(dslSource);
        } catch (parseError) {
          // DSL is invalid, clear score card and return early
          setScoreCard(null);
          return;
        }

        // DSL is valid, proceed with score calculation
        const result = await api.calculateArchitectureScore(dslSource);
        setScoreCard(result as unknown as ScoreCard);
      } catch (error) {
        // Score calculation failed (but DSL was valid)
        // This could be due to missing model items or other issues
        // Clear the score card to avoid showing stale data
        setScoreCard(null);
        // Only log in development to avoid noise in production
        if (process.env.NODE_ENV === "development") {
          logger.debug("Score calculation skipped", {
            component: "ViewTabs",
            action: "calculateScore",
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }
    };
    calculateScore();
  }, [dslSource]);

  const getGradeColor = (grade: string) => {
    switch (grade) {
      case "A":
        return "var(--mantine-color-green-filled)";
      case "B":
        return "var(--mantine-color-blue-filled)";
      case "C":
        return "var(--mantine-color-yellow-filled)";
      case "D":
        return "var(--mantine-color-red-filled)";
      default:
        return "var(--mantine-color-gray-filled)";
    }
  };

  // Builder is now always visible (first tab)
  const tabs: ViewTab[] = ["builder", "diagram", "details", "code", "governance"];

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
        aria-controls="tabpanel-code"
        title="Code - View and edit Sruja DSL source code"
      >
        <div className="view-tab-content">
          <FileCode size={16} />
          <span>Code</span>
        </div>
      </Button>
      <Button
        variant={activeTab === "governance" ? "secondary" : "ghost"}
        size="sm"
        className={`view-tab ${activeTab === "governance" ? "active" : ""}`}
        onClick={() => onTabChange("governance")}
        role="tab"
        aria-selected={activeTab === "governance"}
        id="tab-governance"
        aria-controls="tabpanel-governance"
        title="Governance - Architecture health score and recommendations"
      >
        <div className="view-tab-content">
          <Shield size={16} />
          <span>Governance</span>
          {scoreCard && (
            <span
              className="tab-badge"
              style={{
                backgroundColor: getGradeColor(scoreCard.Grade),
                color: "#fff",
                fontWeight: "bold",
              }}
            >
              {scoreCard.Grade}
            </span>
          )}
        </div>
      </Button>
    </div>
  );
}

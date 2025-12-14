import { useState } from "react";
import { Target, FileText, Play, Workflow } from "lucide-react";
import { RequirementsPanel } from "../Panels/RequirementsPanel";
import { ADRsPanel } from "../Panels/ADRsPanel";
import { useArchitectureStore } from "../../stores";
import "./DetailsView.css";

type DetailSubTab = "requirements" | "adrs" | "scenarios" | "flows";

export function DetailsView() {
  const [activeSubTab, setActiveSubTab] = useState<DetailSubTab>("requirements");
  const data = useArchitectureStore((s) => s.data);

  const scenarios = data?.architecture?.scenarios ?? [];
  const flows = data?.architecture?.flows ?? [];
  const requirements = data?.architecture?.requirements ?? [];
  const adrs = data?.architecture?.adrs ?? [];

  const tabs: { id: DetailSubTab; label: string; icon: React.ReactNode; count: number }[] = [
    {
      id: "requirements",
      label: "Requirements",
      icon: <Target size={14} />,
      count: requirements.length,
    },
    { id: "adrs", label: "ADRs", icon: <FileText size={14} />, count: adrs.length },
    { id: "scenarios", label: "Scenarios", icon: <Play size={14} />, count: scenarios.length },
    { id: "flows", label: "Flows", icon: <Workflow size={14} />, count: flows.length },
  ];

  return (
    <div className="details-view">
      <div className="details-subnav">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`subnav-item ${activeSubTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveSubTab(tab.id)}
          >
            {tab.icon}
            <span>{tab.label}</span>
            {tab.count > 0 && <span className="subnav-count">{tab.count}</span>}
          </button>
        ))}
      </div>
      <div className="details-content">
        {activeSubTab === "requirements" && <RequirementsPanel />}
        {activeSubTab === "adrs" && <ADRsPanel />}
        {activeSubTab === "scenarios" && (
          <div className="details-placeholder">
            <Play size={48} className="placeholder-icon" />
            <h3>Scenarios</h3>
            <p>
              {scenarios.length > 0
                ? `${scenarios.length} scenario${scenarios.length !== 1 ? "s" : ""} defined. View and manage scenarios in the Diagram tab.`
                : "No scenarios defined yet. Create scenarios to document user journeys and use cases."}
            </p>
          </div>
        )}
        {activeSubTab === "flows" && (
          <div className="details-placeholder">
            <Workflow size={48} className="placeholder-icon" />
            <h3>Flows</h3>
            <p>
              {flows.length > 0
                ? `${flows.length} flow${flows.length !== 1 ? "s" : ""} defined. View and manage flows in the Diagram tab.`
                : "No flows defined yet. Create flows to document data movement and interactions."}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

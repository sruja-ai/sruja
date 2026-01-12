import { Shield, FileText, Play, LayoutDashboard, Users } from "lucide-react";
import { useUIStore } from "../../../stores";
import { Button } from "@sruja/ui";
import { GovernanceScore } from "../../Architect/GovernanceScore";
import { useArchitectureStore } from "../../../stores";
import type { SrujaModelDump } from "@sruja/shared";

export function ProjectInspector() {
  const model = useArchitectureStore((s) => s.model) as SrujaModelDump;

  if (!model) return <div className="p-4 text-center text-gray-500">No model loaded</div>;

  const systemCount = Object.values(model.elements).filter((e) => e.kind === "system").length;
  const containerCount = Object.values(model.elements).filter((e) => e.kind === "container").length;
  const relationCount = model.relations?.length ?? 0;

  return (
    <div className="inspector-content">
      {/* Governance Score Widget */}
      <section className="inspector-section">
        <GovernanceScore />
      </section>

      {/* Project Stats */}
      <section className="inspector-section">
        <div className="inspector-section-header">
          <span>Project Stats</span>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <div className="inspector-item flex flex-col items-center justify-center p-2">
            <span className="text-2xl font-bold text-gray-700">{systemCount}</span>
            <span className="text-xs text-gray-500 uppercase">Systems</span>
          </div>
          <div className="inspector-item flex flex-col items-center justify-center p-2">
            <span className="text-2xl font-bold text-gray-700">{containerCount}</span>
            <span className="text-xs text-gray-500 uppercase">Containers</span>
          </div>
          <div className="inspector-item flex flex-col items-center justify-center p-2 col-span-2">
            <span className="text-lg font-bold text-gray-700">{relationCount}</span>
            <span className="text-xs text-gray-500 uppercase">Relationships</span>
          </div>
        </div>
      </section>

      {/* Quick Actions */}
      <section className="inspector-section">
        <div className="inspector-section-header">
          <span>Quick Actions</span>
        </div>
        <div className="flex flex-col gap-2">
          <Button
            variant="outline"
            className="justify-start gap-2"
            onClick={() => useUIStore.getState().setActiveTab("overview")}
          >
            <LayoutDashboard size={14} />
            <span>Full Dashboard</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start gap-2"
            onClick={() => useUIStore.getState().setActiveTab("roles")}
          >
            <Users size={14} />
            <span>Manage Roles</span>
          </Button>
          <div className="h-px bg-gray-100 my-1" />
          <Button variant="outline" className="justify-start gap-2">
            <Shield size={14} />
            <span>Verifications</span>
          </Button>
          <Button variant="outline" className="justify-start gap-2">
            <FileText size={14} />
            <span>Generate Docs</span>
          </Button>
          <Button variant="outline" className="justify-start gap-2">
            <Play size={14} />
            <span>Run Simulation</span>
          </Button>
        </div>
      </section>
    </div>
  );
}

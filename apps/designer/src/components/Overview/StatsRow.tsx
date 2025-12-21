import { Layers, Users, Target, Info, Plus } from "lucide-react";
import { Button } from "@sruja/ui";

interface Stats {
  systems: number;
  persons: number;
  requirements: number;
  adrs: number;
  policies: number;
  flows: number;
}

interface StatsRowProps {
  stats: Stats | null;
  onAddRequirement: () => void;
  onAddADR: () => void;
}

export function StatsRow({ stats, onAddRequirement, onAddADR }: StatsRowProps) {
  if (!stats) return null;

  return (
    <div className="overview-stats">
      <div className="stat-card">
        <Layers size={20} />
        <span className="stat-value">{stats.systems}</span>
        <span className="stat-label">Systems</span>
      </div>
      <div className="stat-card">
        <Users size={20} />
        <span className="stat-value">{stats.persons}</span>
        <span className="stat-label">Actors</span>
      </div>
      <div className="stat-card stat-card-action">
        <Target size={20} />
        <span className="stat-value">{stats.requirements}</span>
        <span className="stat-label">Requirements</span>
        <Button variant="ghost" size="sm" className="stat-add-btn" onClick={onAddRequirement} title="Add Requirement">
          <Plus size={14} />
        </Button>
      </div>
      <div className="stat-card stat-card-action">
        <Info size={20} />
        <span className="stat-value">{stats.adrs}</span>
        <span className="stat-label">ADRs</span>
        <Button variant="ghost" size="sm" className="stat-add-btn" onClick={onAddADR} title="Add ADR">
          <Plus size={14} />
        </Button>
      </div>
    </div>
  );
}

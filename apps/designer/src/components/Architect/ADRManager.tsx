// apps/designer/src/components/Architect/ADRManager.tsx
import { useState, useMemo } from "react";
import {
  BookOpen,
  Link2,
  Search,
  Plus,
  CheckCircle,
  Clock,
  XCircle,
  AlertCircle,
} from "lucide-react";
import { Input, Button, Badge } from "@sruja/ui";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { useArchitectureStore } from "../../stores";
import "./ADRManager.css";

interface ADRWithComponents {
  id: string;
  title?: string;
  status?: string;
  decision?: string;
  affectedComponents: string[]; // Component IDs linked to this ADR
}

export function ADRManager() {
  const model = getArchitectureModel();
  const adrs = model.getADRs();
  const nodes = model.getNodes();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedADR, setSelectedADR] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [newADR, setNewADR] = useState({ title: "", decision: "", status: "proposed" });

  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const handleSaveADR = () => {
    if (!newADR.title.trim()) return;

    updateArchitecture((model) => {
      const existingADRs = model.sruja?.adrs || [];
      const id = `ADR-${String(existingADRs.length + 1).padStart(3, "0")}`;

      return {
        ...model,
        sruja: {
          ...model.sruja,
          adrs: [
            ...existingADRs,
            {
              id,
              title: newADR.title,
              decision: newADR.decision,
              status: newADR.status as "proposed" | "accepted" | "rejected" | "superseded",
              date: new Date().toISOString().split("T")[0],
            },
          ],
        },
      };
    });

    setNewADR({ title: "", decision: "", status: "proposed" });
    setIsCreating(false);
  };

  // Map ADRs to their affected components
  const adrsWithComponents = useMemo<ADRWithComponents[]>(() => {
    return adrs.map((adr) => {
      const adrId = adr.id || "unknown";
      const affectedComponents: string[] = [];

      // Find components linked to this ADR via tags or metadata
      for (const [nodeId, node] of nodes.entries()) {
        const nodeData = node as unknown as {
          tags?: string[];
          metadata?: { adrs?: string[] };
          adrs?: { id: string }[];
        };
        if (
          nodeData.tags?.includes(adrId) ||
          nodeData.metadata?.adrs?.includes(adrId) ||
          nodeData.adrs?.some((a) => a.id === adrId)
        ) {
          affectedComponents.push(nodeId);
        }
      }

      return {
        id: adrId,
        title: adr.title || adrId,
        status: adr.status || "proposed",
        decision: adr.decision || adr.context,
        affectedComponents,
      };
    });
  }, [adrs, nodes]);

  const filteredADRs = useMemo(() => {
    if (!searchQuery.trim()) return adrsWithComponents;
    const query = searchQuery.toLowerCase();
    return adrsWithComponents.filter(
      (adr) =>
        adr.id.toLowerCase().includes(query) ||
        adr.title?.toLowerCase().includes(query) ||
        adr.status?.toLowerCase().includes(query) ||
        adr.decision?.toLowerCase().includes(query)
    );
  }, [adrsWithComponents, searchQuery]);

  const statusConfig: Record<
    string,
    { icon: React.ReactNode; color: "success" | "warning" | "error" | "info" }
  > = {
    accepted: { icon: <CheckCircle size={14} />, color: "success" },
    proposed: { icon: <Clock size={14} />, color: "warning" },
    deprecated: { icon: <XCircle size={14} />, color: "error" },
    superseded: { icon: <AlertCircle size={14} />, color: "info" },
  };

  if (adrs.length === 0) {
    return (
      <div className="adr-manager">
        <div className="adr-manager-header">
          <h3 className="adr-manager-title">
            <BookOpen size={18} />
            ADR Manager
          </h3>
        </div>
        <div className="adr-manager-empty">
          <p>No ADRs defined.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="adr-manager">
      <div className="adr-manager-header">
        <h3 className="adr-manager-title">
          <BookOpen size={18} />
          ADR Manager
        </h3>
        <div className="adr-manager-actions">
          {!isCreating && (
            <Button variant="secondary" size="sm" onClick={() => setIsCreating(true)}>
              <Plus size={14} />
              New
            </Button>
          )}
        </div>
      </div>

      {isCreating && (
        <div className="adr-creation-form">
          <div className="form-group">
            <label>Title</label>
            <Input
              autoFocus
              placeholder="ADR Title"
              value={newADR.title}
              onChange={(e) => setNewADR({ ...newADR, title: e.target.value })}
            />
          </div>
          <div className="form-group">
            <label>Decision</label>
            <textarea
              className="adr-textarea"
              placeholder="What was decided..."
              rows={3}
              value={newADR.decision}
              onChange={(e) => setNewADR({ ...newADR, decision: e.target.value })}
            />
          </div>
          <div className="form-group">
            <label>Status</label>
            <select
              className="adr-select"
              value={newADR.status}
              onChange={(e) => setNewADR({ ...newADR, status: e.target.value })}
            >
              <option value="proposed">Proposed</option>
              <option value="accepted">Accepted</option>
              <option value="rejected">Rejected</option>
              <option value="superseded">Superseded</option>
            </select>
          </div>
          <div className="form-actions">
            <Button variant="primary" size="sm" onClick={handleSaveADR}>
              Save
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setIsCreating(false)}>
              Cancel
            </Button>
          </div>
        </div>
      )}

      <div className="adr-manager-filters">
        <div className="adr-manager-search">
          <Search size={16} className="search-icon" />
          <Input
            type="text"
            placeholder="Search ADRs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="adr-search-input"
          />
        </div>
      </div>

      <div className="adr-manager-list">
        {filteredADRs.length === 0 ? (
          <div className="adr-manager-empty-state">
            <BookOpen size={24} className="empty-icon" />
            <p>No ADRs match your search.</p>
          </div>
        ) : (
          filteredADRs.map((adr) => {
            const status = statusConfig[adr.status || "proposed"] || statusConfig.proposed;
            const isSelected = selectedADR === adr.id;

            return (
              <div
                key={adr.id}
                className={`adr-item ${isSelected ? "selected" : ""}`}
                onClick={() => setSelectedADR(isSelected ? null : adr.id)}
              >
                <div className="adr-item-header">
                  <Badge color={status.color} className="adr-item-status">
                    {status.icon}
                    <span style={{ marginLeft: "4px" }}>{adr.status || "proposed"}</span>
                  </Badge>
                  <div className="adr-item-content">
                    <div className="adr-item-id-row">
                      <div className="adr-item-id">{adr.id}</div>
                      {(adr.status === "superseded" || adr.status === "deprecated") && (
                        <Badge color="warning" className="priority-badge-small">
                          Review needed
                        </Badge>
                      )}
                    </div>
                    <div className="adr-item-title">{adr.title}</div>
                    {adr.decision && (
                      <div className="adr-item-decision">
                        {adr.decision.length > 100
                          ? `${adr.decision.substring(0, 100)}...`
                          : adr.decision}
                      </div>
                    )}
                  </div>
                </div>
                {adr.affectedComponents.length > 0 && (
                  <div className="adr-item-components">
                    <Link2 size={12} />
                    <span>
                      Affects: {adr.affectedComponents.slice(0, 3).join(", ")}
                      {adr.affectedComponents.length > 3 &&
                        ` (+${adr.affectedComponents.length - 3} more)`}
                    </span>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}

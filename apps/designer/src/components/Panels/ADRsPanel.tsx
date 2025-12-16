import { useState, useMemo, useEffect } from "react";
import {
  BookOpen,
  ChevronDown,
  ChevronUp,
  CheckCircle,
  Clock,
  XCircle,
  AlertCircle,
  Edit,
  Plus,
  Trash2,
} from "lucide-react";
import { useArchitectureStore, useUIStore, useSelectionStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { EditADRForm, ConfirmDialog } from "../shared";
import { Input } from "@sruja/ui";
import type { ADRJSON } from "../../types";
import "./ADRsPanel.css";

const STATUS_CONFIG: Record<string, { icon: React.ReactNode; color: string }> = {
  accepted: { icon: <CheckCircle size={14} />, color: "#22c55e" },
  proposed: { icon: <Clock size={14} />, color: "#f59e0b" },
  deprecated: { icon: <XCircle size={14} />, color: "#ef4444" },
  superseded: { icon: <AlertCircle size={14} />, color: "#8b5cf6" },
};

export function ADRsPanel() {
  const data = useArchitectureStore((s) => s.data);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [editADR, setEditADR] = useState<ADRJSON | undefined>(undefined);
  const [showEditForm, setShowEditForm] = useState(false);
  const [deleteADR, setDeleteADR] = useState<ADRJSON | undefined>(undefined);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
  const pendingAction = useUIStore((s) => s.pendingAction);
  const clearPendingAction = useUIStore((s) => s.clearPendingAction);

  // Initial load action handler
  useEffect(() => {
    if (pendingAction === "create-adr") {
      setEditADR(undefined);
      setShowEditForm(true);
      clearPendingAction();
    }
  }, [pendingAction, clearPendingAction]);

  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);

  const adrs = useMemo(() => {
    return data?.architecture?.adrs ?? [];
  }, [data]);

  const filteredADRs = useMemo(() => {
    let filtered = adrs;

    // Filter by selected node
    if (selectedNodeId) {
      filtered = filtered.filter((a) => a.tags?.includes(selectedNodeId));
    }

    if (!searchQuery.trim()) return filtered;

    const query = searchQuery.toLowerCase();
    return filtered.filter(
      (adr) =>
        adr.id?.toLowerCase().includes(query) ||
        adr.title?.toLowerCase().includes(query) ||
        adr.status?.toLowerCase().includes(query) ||
        adr.context?.toLowerCase().includes(query) ||
        adr.decision?.toLowerCase().includes(query) ||
        adr.consequences?.toLowerCase().includes(query)
    );
  }, [adrs, searchQuery, selectedNodeId]);

  // If no ADRs at all (regardless of filter), show nothing?
  // Wait, original code: if (!adrs.length) return null;
  // We should keep that unless we want to show "No ADRs for selection".
  // Better UX: Show panel but say "No relevant ADRs for selection".
  if (!adrs.length) {
    return null;
  }

  const toggleExpand = (id: string) => {
    setExpandedId(expandedId === id ? null : id);
  };

  return (
    <div className="adrs-panel">
      <h3 className="adrs-title">
        <BookOpen size={18} />
        Architecture Decision Records
        <span className="adrs-count">{adrs.length}</span>
        {isEditMode() && (
          <button
            className="adrs-add-btn"
            onClick={() => {
              setEditADR(undefined);
              setShowEditForm(true);
            }}
            title="Add ADR"
          >
            <Plus size={14} />
          </button>
        )}
      </h3>

      {/* Selection Filter Info */}
      {selectedNodeId && (
        <div
          className="filter-info-banner"
          style={{
            padding: "8px 12px",
            background: "var(--bg-secondary)",
            borderBottom: "1px solid var(--border-color)",
            fontSize: "0.85rem",
            display: "flex",
            alignItems: "center",
            gap: "6px",
            color: "var(--text-secondary)",
          }}
        >
          <div
            style={{ width: 8, height: 8, borderRadius: "50%", background: "var(--primary-color)" }}
          ></div>
          Filtered by selection: <strong>{selectedNodeId}</strong>
        </div>
      )}

      {/* Search */}
      <div className="adrs-search">
        <Input
          placeholder="Search ADRs..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <div className="adrs-list">
        {filteredADRs.map((adr) => {
          const isExpanded = expandedId === adr.id;
          const status = adr.status?.toLowerCase() || "proposed";
          const statusConfig = STATUS_CONFIG[status] || STATUS_CONFIG.proposed;

          return (
            <div key={adr.id} className={`adr-card ${isExpanded ? "expanded" : ""}`}>
              <button className="adr-header" onClick={() => toggleExpand(adr.id)}>
                <span className="adr-id">{adr.id}</span>
                <span className="adr-title-text">{adr.title}</span>
                <span
                  className="adr-status"
                  style={{ "--status-color": statusConfig.color } as React.CSSProperties}
                >
                  {statusConfig.icon}
                  {adr.status || "Proposed"}
                </span>
                {isEditMode() && (
                  <div className="adr-actions">
                    <button
                      className="adr-edit-btn"
                      onClick={(e) => {
                        e.stopPropagation();
                        setEditADR(adr);
                        setShowEditForm(true);
                      }}
                      title="Edit ADR"
                    >
                      <Edit size={14} />
                    </button>
                    <button
                      className="adr-delete-btn"
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteADR(adr);
                        setShowDeleteConfirm(true);
                      }}
                      title="Delete ADR"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                )}
                {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
              </button>

              {isExpanded && (
                <div className="adr-content">
                  {adr.context && (
                    <div className="adr-section">
                      <h4>Context</h4>
                      <p>{adr.context}</p>
                    </div>
                  )}
                  {adr.decision && (
                    <div className="adr-section">
                      <h4>Decision</h4>
                      <p>{adr.decision}</p>
                    </div>
                  )}
                  {adr.consequences && (
                    <div className="adr-section">
                      <h4>Consequences</h4>
                      <p>{adr.consequences}</p>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      <EditADRForm
        isOpen={showEditForm}
        onClose={() => {
          setShowEditForm(false);
          setEditADR(undefined);
        }}
        adr={editADR}
      />
      <ConfirmDialog
        isOpen={showDeleteConfirm}
        onClose={() => {
          setShowDeleteConfirm(false);
          setDeleteADR(undefined);
        }}
        onConfirm={async () => {
          if (deleteADR) {
            await updateArchitecture((arch) => {
              if (!arch.architecture) return arch;
              const adrs = (arch.architecture.adrs || []).filter((a) => a.id !== deleteADR.id);
              return {
                ...arch,
                architecture: {
                  ...arch.architecture,
                  adrs,
                },
              };
            });
          }
        }}
        title="Delete ADR"
        message={`Are you sure you want to delete ADR "${deleteADR?.id}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </div>
  );
}

import { useMemo, useState, useEffect } from "react";
import {
  CheckCircle,
  FileText,
  Lock,
  Gauge,
  Shield,
  AlertCircle,
  Edit,
  Trash2,
  Plus,
  XCircle,
  GitBranch,
} from "lucide-react";
import { useArchitectureStore, useUIStore, useSelectionStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { EditRequirementForm, ConfirmDialog } from "../shared";
import { Input, Button } from "@sruja/ui";
import type { RequirementDump } from "@sruja/shared";
import { useTagNavigation } from "../../hooks/useTagNavigation";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import "./RequirementsPanel.css";

type RequirementType =
  | "functional"
  | "performance"
  | "security"
  | "constraint"
  | "reliability"
  | "all";

const REQUIREMENT_TYPES: {
  type: RequirementType;
  label: string;
  icon: React.ReactNode;
  color: string;
}[] = [
    { type: "all", label: "All", icon: <FileText size={14} />, color: "#667eea" },
    { type: "functional", label: "Functional", icon: <CheckCircle size={14} />, color: "#22c55e" },
    { type: "performance", label: "Performance", icon: <Gauge size={14} />, color: "#f59e0b" },
    { type: "security", label: "Security", icon: <Shield size={14} />, color: "#ef4444" },
    { type: "constraint", label: "Constraint", icon: <Lock size={14} />, color: "#8b5cf6" },
    { type: "reliability", label: "Reliability", icon: <AlertCircle size={14} />, color: "#06b6d4" },
  ];

export function RequirementsPanel() {
  const model = useArchitectureStore((s) => s.likec4Model);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
  const [activeType, setActiveType] = useState<RequirementType>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [editRequirement, setEditRequirement] = useState<RequirementDump | undefined>(undefined);
  const [showEditForm, setShowEditForm] = useState(false);
  const [deleteRequirement, setDeleteRequirement] = useState<RequirementDump | undefined>(
    undefined
  );
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const pendingAction = useUIStore((s) => s.pendingAction);
  const clearPendingAction = useUIStore((s) => s.clearPendingAction);

  // Initial load action handler
  useEffect(() => {
    if (pendingAction === "create-requirement") {
      setEditRequirement(undefined);
      setShowEditForm(true);
      clearPendingAction();
    }
  }, [pendingAction, clearPendingAction]);

  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const { navigateToTaggedElement } = useTagNavigation();
  const [selectedRequirement, setSelectedRequirement] = useState<string | null>(null);

  // Root-level requirements only
  // Deduplicate by requirement ID to prevent showing duplicates
  const requirements = useMemo(() => {
    if (!model) return [];

    // @ts-expect-error: types mismatch
    const reqs = (model.sruja as any)?.requirements || [];

    // Deduplicate by ID to prevent showing duplicates
    return deduplicateRequirements(reqs);
  }, [model]);

  const filteredRequirements = useMemo(() => {
    let filtered = requirements;

    // Filter by selected node - RequirementDump might not have tags for linking to elements directly in standard model, but maybe it does. Assuming tags for now but safely.
    if (selectedNodeId) {
      // @ts-expect-error: types mismatch
      filtered = filtered.filter((r) => r.tags?.includes(selectedNodeId));
    }

    // Filter by type
    if (activeType !== "all") {
      // @ts-expect-error: types mismatch
      filtered = filtered.filter((r) => r.type?.toLowerCase() === activeType);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (r) =>
          r.id?.toLowerCase().includes(query) ||
          r.title?.toLowerCase().includes(query) ||
          r.description?.toLowerCase().includes(query) ||
          // @ts-expect-error: types mismatch
          r.type?.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [requirements, activeType, searchQuery, selectedNodeId]);

  const countByType = useMemo(() => {
    // Count based on CURRENTLY filtered list (by selection), but ignoring type filter
    // Actually, usually counts show total available.
    // If selection is active, counts should arguably reflect the selection subset.
    // Let's filter by selection first for counts.
    let baseList = requirements;
    if (selectedNodeId) {
      // @ts-expect-error: types mismatch
      baseList = baseList.filter((r) => r.tags?.includes(selectedNodeId));
    }

    const counts: Record<string, number> = { all: baseList.length };
    baseList.forEach((r) => {
      // @ts-expect-error: types mismatch
      const type = r.type?.toLowerCase() || "other";
      counts[type] = (counts[type] || 0) + 1;
    });
    return counts;
  }, [requirements, selectedNodeId]);

  // Calculate requirement coverage for traceability
  const requirementCoverage = useMemo(() => {
    const coverage: Record<string, {
      elementIds: string[];
      coverage: number;
      status: "fulfilled" | "partial" | "missing";
    }> = {};

    requirements.forEach((req) => {
      // @ts-expect-error: types mismatch
      const elementIds: string[] = req.tags ?? [];
      const hasLinks = elementIds.length > 0;
      const status: "fulfilled" | "partial" | "missing" =
        hasLinks ? (elementIds.length >= 2 ? "fulfilled" : "partial") : "missing";

      coverage[req.id] = {
        elementIds,
        coverage: hasLinks ? Math.min(100, (elementIds.length / 3) * 100) : 0,
        status,
      };
    });

    return coverage;
  }, [requirements]);

  // Calculate overall coverage
  const overallCoverage = useMemo(() => {
    if (requirements.length === 0) return 0;
    const total = requirements.reduce((sum, req) => {
      const cov = requirementCoverage[req.id];
      return sum + (cov?.coverage ?? 0);
    }, 0);
    return Math.round(total / requirements.length);
  }, [requirements, requirementCoverage]);

  const handleRequirementClick = (req: RequirementDump) => {
    if (selectedRequirement === req.id) {
      setSelectedRequirement(null);
    } else {
      setSelectedRequirement(req.id);
      // Could trigger element highlighting here in the future
    }
  };

  return (
    <div className="requirements-panel">
      <div className="requirements-header">
        <h3 className="requirements-title">
          <FileText size={18} />
          Requirements
          {requirements.length > 0 && (
            <span className="requirements-count">{requirements.length}</span>
          )}
        </h3>
        {requirements.length > 0 && (
          <div className="coverage-summary">
            <GitBranch size={14} />
            <span className="coverage-label">Coverage:</span>
            <span className={`coverage-value ${overallCoverage >= 80 ? "good" : overallCoverage >= 50 ? "medium" : "poor"}`}>
              {overallCoverage}%
            </span>
          </div>
        )}
        {isEditMode() && (
          <Button
            variant="ghost"
            size="sm"
            className="requirements-add-btn"
            onClick={() => {
              setEditRequirement(undefined);
              setShowEditForm(true);
            }}
            title="Add Requirement"
            aria-label="Add Requirement"
          >
            <Plus size={14} />
          </Button>
        )}
      </div>

      {!requirements.length ? (
        <div className="requirements-empty">
          <FileText size={48} className="empty-icon" />
          <p>No requirements defined</p>
          <p className="empty-hint">Add requirements to your architecture DSL to see them here</p>
        </div>
      ) : (
        <>
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
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  background: "var(--primary-color)",
                }}
              ></div>
              Filtered by selection: <strong>{selectedNodeId}</strong>
            </div>
          )}

          {/* Search */}
          <div className="requirements-search">
            <Input
              placeholder="Search requirements..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>

          {/* Type Tabs */}
          <div className="requirements-tabs">
            {REQUIREMENT_TYPES.map(({ type, label, icon, color }) => (
              <Button
                key={type}
                variant={activeType === type ? "primary" : "ghost"}
                size="sm"
                className={`req-tab ${activeType === type ? "active" : ""}`}
                style={{ "--tab-color": color } as React.CSSProperties}
                onClick={() => setActiveType(type)}
              >
                {icon}
                <span>{label}</span>
                {countByType[type] !== undefined && (
                  <span className="tab-count">{countByType[type] || 0}</span>
                )}
              </Button>
            ))}
          </div>

          {/* Requirements List */}
          <div className="requirements-list">
            {filteredRequirements.map((req, index) => {
              // @ts-expect-error: types mismatch
              const typeConfig = REQUIREMENT_TYPES.find((t) => t.type === req.type?.toLowerCase());
              const coverage = requirementCoverage[req.id];
              const isSelected = selectedRequirement === req.id;
              const hasLinks = (coverage?.elementIds.length ?? 0) > 0;
              // Use index as fallback for key if id is missing or duplicate
              const uniqueKey = req.id ? `${req.id}-${index}` : `req-${index}`;
              return (
                <div
                  key={uniqueKey}
                  className={`requirement-card ${isSelected ? "selected" : ""} ${coverage?.status || "missing"}`}
                  style={{ "--req-color": typeConfig?.color || "#667eea" } as React.CSSProperties}
                  onClick={() => handleRequirementClick(req)}
                >
                  <div className="req-header">
                    <span className="req-id">{req.id || `REQ-${index + 1}`}</span>
                    {/* @ts-expect-error: types mismatch */}
                    {req.type && (
                      <span className="req-type-badge">
                        {typeConfig?.icon}
                        {/* @ts-expect-error: types mismatch */}
                        {req.type}
                      </span>
                    )}
                    {coverage && (
                      <div className={`status-badge ${coverage.status}`}>
                        {coverage.status === "fulfilled" ? (
                          <CheckCircle size={12} />
                        ) : coverage.status === "partial" ? (
                          <AlertCircle size={12} />
                        ) : (
                          <XCircle size={12} />
                        )}
                        <span>{coverage.status}</span>
                      </div>
                    )}
                    {isEditMode() && (
                      <div className="req-actions">
                        <Button
                          variant="ghost"
                          size="sm"
                          className="req-edit-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setEditRequirement(req);
                            setShowEditForm(true);
                          }}
                          title="Edit Requirement"
                        >
                          <Edit size={14} />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="req-delete-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setDeleteRequirement(req);
                            setShowDeleteConfirm(true);
                          }}
                          title="Delete Requirement"
                        >
                          <Trash2 size={14} />
                        </Button>
                      </div>
                    )}
                  </div>
                  <p className="req-description">
                    {req.title || req.description || "No description"}
                  </p>
                  {coverage && hasLinks && (
                    <div className="requirement-coverage">
                      <div className="coverage-bar">
                        <div
                          className="coverage-fill"
                          style={{ width: `${coverage.coverage}%` }}
                        />
                      </div>
                      <span className="coverage-text">
                        {coverage.elementIds.length} element{coverage.elementIds.length !== 1 ? "s" : ""}
                      </span>
                    </div>
                  )}
                  {/* @ts-expect-error: types mismatch */}
                  {req.tags && req.tags.length > 0 && (
                    <div className="req-tags">
                      {/* @ts-expect-error: types mismatch */}
                      {req.tags.map((tag) => (
                        <Button
                          key={tag}
                          variant="ghost"
                          size="sm"
                          className="req-tag clickable"
                          onClick={(e) => {
                            e.stopPropagation();
                            navigateToTaggedElement(tag);
                          }}
                          title={`Navigate to ${tag} in diagram`}
                        >
                          {tag}
                        </Button>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}

      <EditRequirementForm
        isOpen={showEditForm}
        onClose={() => {
          setShowEditForm(false);
          setEditRequirement(undefined);
        }}
        requirement={editRequirement}
      />
      <ConfirmDialog
        isOpen={showDeleteConfirm}
        onClose={() => {
          setShowDeleteConfirm(false);
          setDeleteRequirement(undefined);
        }}
        onConfirm={async () => {
          if (deleteRequirement) {
            await updateArchitecture((model) => {
              const sruja = (model as any).sruja || {};
              // @ts-expect-error: types mismatch
              const requirements = (sruja.requirements || []).filter(
                (r: any) => r.id !== deleteRequirement.id
              );
              return {
                ...model,
                sruja: {
                  ...sruja,
                  requirements,
                },
              };
            });
          }
        }}
        title="Delete Requirement"
        message={`Are you sure you want to delete requirement "${deleteRequirement?.id}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </div>
  );
}

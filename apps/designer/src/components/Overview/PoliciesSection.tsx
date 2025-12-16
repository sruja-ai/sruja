import { Shield, Plus, Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useSelectionStore } from "../../stores";
import type { PolicyJSON } from "../../types";

interface PoliciesSectionProps {
  policies: PolicyJSON[] | undefined;
  policyCount: number;
  onAddPolicy: () => void;
  onEditPolicy: (policy: PolicyJSON) => void;
}

export function PoliciesSection({
  policies,
  policyCount,
  onAddPolicy,
  onEditPolicy,
}: PoliciesSectionProps) {
  const isFeatureEnabled = useFeatureFlagsStore((s) => s.isEnabled);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);

  const filteredPolicies = policies?.filter((p) => {
    if (!selectedNodeId) return true;
    return p.tags?.includes(selectedNodeId);
  });

  if (!isFeatureEnabled("policies") || policyCount <= 0) return null;

  // If filtered list is empty but we have policies, should we hide the section or show "None"?
  // If filtering is active, better to show filtering state.
  if (selectedNodeId && (!filteredPolicies || filteredPolicies.length === 0)) {
    // Optional: Hide section if no relevant policies?
    // User request: "apply selected node as filter"
    // If I select a node and it has NO policies, maybe I want to see that.
    // Let's keep the section if we have global policies, but list is empty.
  }

  return (
    <div className="overview-policies-section">
      <div className="policies-section-header">
        <h3 className="policies-section-title">
          <Shield size={16} />
          Policies ({filteredPolicies?.length ?? 0}/{policyCount})
        </h3>
        {isEditMode() && (
          <Button variant="ghost" size="sm" onClick={onAddPolicy} title="Add Policy">
            <Plus size={14} />
          </Button>
        )}
      </div>

      {selectedNodeId && (
        <div
          className="filter-info-banner"
          style={{
            padding: "6px 10px",
            marginBottom: "8px",
            background: "var(--bg-secondary)",
            border: "1px solid var(--border-color)",
            borderRadius: "4px",
            fontSize: "0.8rem",
            display: "flex",
            alignItems: "center",
            gap: "6px",
            color: "var(--text-secondary)",
          }}
        >
          <div
            style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--primary-color)" }}
          ></div>
          Filtered by: <strong>{selectedNodeId}</strong>
        </div>
      )}

      <div className="policies-list">
        {filteredPolicies?.map((policy) => (
          <div key={policy.id} className="policy-card">
            <div className="policy-card-content">
              <span className="policy-id">{policy.id}</span>
              <span className="policy-label">{policy.label || policy.description}</span>
            </div>
            {isEditMode() && (
              <div className="policy-card-actions">
                <button
                  className="policy-edit-btn"
                  onClick={() => onEditPolicy(policy)}
                  title="Edit Policy"
                >
                  <Edit size={12} />
                </button>
              </div>
            )}
          </div>
        ))}
        {filteredPolicies?.length === 0 && (
          <div
            className="empty-message"
            style={{ padding: "8px", color: "var(--text-tertiary)", fontSize: "0.85rem" }}
          >
            No policies linked to this element.
          </div>
        )}
      </div>
    </div>
  );
}

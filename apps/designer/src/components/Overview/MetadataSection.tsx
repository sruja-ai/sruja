import { useMemo } from "react";
import { Plus, Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import type { MetadataEntry } from "../../types";

interface MetadataSectionProps {
  metadata: MetadataEntry[] | undefined;
  onAddMetadata: () => void;
  onEditMetadata: (metadata: MetadataEntry, index: number) => void;
  onDeleteMetadata: (index: number, key: string) => void;
}

export function MetadataSection({
  metadata,
  onAddMetadata,
  onEditMetadata,
  onDeleteMetadata,
}: MetadataSectionProps) {
  const isFeatureEnabled = useFeatureFlagsStore((s) => s.isEnabled);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Group metadata by semantic categories
  const metadataByCategory = useMemo(() => {
    if (!metadata) return {};
    const categories: Record<string, { key: string; value: string }[]> = {
      ownership: [], // team, owner, stakeholders
      status: [], // version, status, scale
      operations: [], // cost, availability, performance
      other: [],
    };
    metadata.forEach((m) => {
      const key = m.key.toLowerCase();
      const value = m.value || m.array?.join(", ") || "";
      if (["team", "owner", "stakeholders"].includes(key)) {
        categories.ownership.push({ key: m.key, value });
      } else if (["version", "status", "scale", "architecture"].includes(key)) {
        categories.status.push({ key: m.key, value });
      } else if (["cost", "availability", "performance"].includes(key)) {
        categories.operations.push({ key: m.key, value });
      } else {
        categories.other.push({ key: m.key, value });
      }
    });
    return categories;
  }, [metadata]);

  if (!isFeatureEnabled("metadata") || !metadata || metadata.length === 0) return null;

  return (
    <div className="overview-metadata-section">
      <div className="metadata-section-header">
        <h3 className="metadata-section-title">Metadata</h3>
        {isEditMode() && (
          <Button variant="ghost" size="sm" onClick={onAddMetadata} title="Add Metadata">
            <Plus size={14} />
          </Button>
        )}
      </div>
      <div className="overview-metadata">
        {Object.entries(metadataByCategory).map(
          ([category, items]) =>
            items.length > 0 && (
              <div key={category} className="metadata-section">
                {items.map((item, i) => {
                  const metadataIndex = metadata.findIndex((m) => m.key === item.key);
                  return (
                    <div key={`${item.key}-${i}`} className="metadata-card metadata-card-editable">
                      <div className="metadata-card-content">
                        <span className="metadata-key">{item.key}</span>
                        <span className="metadata-value">{item.value}</span>
                      </div>
                      {isEditMode() && (
                        <div className="metadata-card-actions">
                          <button
                            className="metadata-edit-btn"
                            onClick={() => onEditMetadata(metadata[metadataIndex], metadataIndex)}
                            title="Edit Metadata"
                          >
                            <Edit size={12} />
                          </button>
                          <button
                            className="metadata-delete-btn"
                            onClick={() => onDeleteMetadata(metadataIndex, item.key)}
                            title="Delete Metadata"
                          >
                            ×
                          </button>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )
        )}
      </div>
    </div>
  );
}

import { Tag, Plus, Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";

interface ConventionsSectionProps {
  conventions: any[] | undefined;
  onAddConvention: () => void;
  onEditConvention: (convention: any, index: number) => void;
  onDeleteConvention: (index: number, key: string) => void;
}

export function ConventionsSection({
  conventions,
  onAddConvention,
  onEditConvention,
  onDeleteConvention,
}: ConventionsSectionProps) {
  const isFeatureEnabled = useFeatureFlagsStore((s) => s.isEnabled);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  if (!isFeatureEnabled("conventions") || !conventions || conventions.length === 0) return null;

  return (
    <div className="overview-conventions-section">
      <div className="conventions-section-header">
        <h3 className="conventions-section-title">
          <Tag size={16} />
          Conventions ({conventions.length})
        </h3>
        {isEditMode() && (
          <Button variant="ghost" size="sm" onClick={onAddConvention} title="Add Convention">
            <Plus size={14} />
          </Button>
        )}
      </div>
      <div className="conventions-list">
        {conventions.map((convention, index) => (
          <div key={`${convention.key}-${index}`} className="convention-card">
            <div className="convention-card-content">
              <span className="convention-key">{convention.key}</span>
              <span className="convention-value">{convention.value}</span>
            </div>
            {isEditMode() && (
              <div className="convention-card-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="convention-edit-btn"
                  onClick={() => onEditConvention(convention, index)}
                  title="Edit Convention"
                >
                  <Edit size={12} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="convention-delete-btn"
                  onClick={() => onDeleteConvention(index, convention.key)}
                  title="Delete Convention"
                >
                  ×
                </Button>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

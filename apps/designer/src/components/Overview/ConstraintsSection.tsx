import { AlertOctagon, Plus, Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";

interface ConstraintsSectionProps {
  constraints: any[] | undefined;
  onAddConstraint: () => void;
  onEditConstraint: (constraint: any, index: number) => void;
  onDeleteConstraint: (index: number, key: string) => void;
}

export function ConstraintsSection({
  constraints,
  onAddConstraint,
  onEditConstraint,
  onDeleteConstraint,
}: ConstraintsSectionProps) {
  const isFeatureEnabled = useFeatureFlagsStore((s) => s.isEnabled);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  if (!isFeatureEnabled("constraints") || !constraints || constraints.length === 0) return null;

  return (
    <div className="overview-constraints-section">
      <div className="constraints-section-header">
        <h3 className="constraints-section-title">
          <AlertOctagon size={16} />
          Constraints ({constraints.length})
        </h3>
        {isEditMode() && (
          <Button variant="ghost" size="sm" onClick={onAddConstraint} title="Add Constraint">
            <Plus size={14} />
          </Button>
        )}
      </div>
      <div className="constraints-list">
        {constraints.map((constraint, index) => (
          <div key={`${constraint.key}-${index}`} className="constraint-card">
            <div className="constraint-card-content">
              <span className="constraint-key">{constraint.key}</span>
              <span className="constraint-value">{constraint.value}</span>
            </div>
            {isEditMode() && (
              <div className="constraint-card-actions">
                <Button
                  variant="ghost"
                  size="sm"
                  className="constraint-edit-btn"
                  onClick={() => onEditConstraint(constraint, index)}
                  title="Edit Constraint"
                >
                  <Edit size={12} />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="constraint-delete-btn"
                  onClick={() => onDeleteConstraint(index, constraint.key)}
                  title="Delete Constraint"
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

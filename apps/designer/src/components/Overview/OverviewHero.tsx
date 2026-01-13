import { Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import type { OverviewJSON, MetadataEntryJSON } from "@sruja/shared";

interface OverviewHeroProps {
  architectureName?: string;
  description?: string;
  overview?: OverviewJSON;
  archMetadata?: MetadataEntryJSON[];
  onEditOverview: () => void;
}

export function OverviewHero({
  architectureName,
  description,
  overview,
  archMetadata,
  onEditOverview,
}: OverviewHeroProps) {
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Use architecture name as primary title, fallback to "Architecture" if not available
  const displayName = architectureName || "Architecture";
  const hasContent = description || overview?.summary;

  return (
    <div className="overview-hero">
      <div className="overview-hero-header">
        <div className="overview-hero-content">
          <h1 className="overview-title">{displayName}</h1>
          {hasContent && (
            <div className="overview-hero-text">
              {description && <p className="overview-description">{description}</p>}
              {overview?.summary && <p className="overview-summary">{overview.summary}</p>}
            </div>
          )}
          {!hasContent && isEditMode() && (
            <div className="overview-hero-empty-hint">
              <p>
                💡 <strong>Tip:</strong> Add a description to explain what this architecture does
                and why it exists.
              </p>
            </div>
          )}
        </div>
        <div className="overview-hero-actions">
          {isEditMode() && (overview || archMetadata) && (
            <Button variant="ghost" size="sm" onClick={onEditOverview} title="Edit Overview">
              <Edit size={16} />
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

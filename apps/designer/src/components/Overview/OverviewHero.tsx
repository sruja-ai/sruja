import { Edit } from "lucide-react";
import { Button } from "@sruja/ui";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import type { OverviewJSON, MetadataEntry } from "../../types";

interface OverviewHeroProps {
  architectureName?: string;
  description?: string;
  overview?: OverviewJSON;
  archMetadata?: MetadataEntry[];
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

  return (
    <div className="overview-hero">
      <div className="overview-hero-header">
        <div>
          <h1 className="overview-title">Sruja Architecture</h1>
          {architectureName && <h2 className="overview-architecture-name">{architectureName}</h2>}
          {description && <p className="overview-description">{description}</p>}
          {overview?.summary && <p className="overview-summary">{overview.summary}</p>}
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

/**
 * GovernanceBadge
 * Shows requirement/ADR counts on diagram nodes
 */

import { FileText, ClipboardCheck } from "lucide-react";

interface GovernanceBadgeProps {
  requirementCount?: number;
  adrCount?: number;
}

export function GovernanceBadge({ requirementCount = 0, adrCount = 0 }: GovernanceBadgeProps) {
  const hasGovernance = requirementCount > 0 || adrCount > 0;

  if (!hasGovernance) return null;

  return (
    <div className="governance-badge">
      {requirementCount > 0 && (
        <span
          className="gov-badge-item req"
          title={`${requirementCount} requirement${requirementCount > 1 ? "s" : ""}`}
        >
          <FileText size={10} />
          <span>{requirementCount}</span>
        </span>
      )}
      {adrCount > 0 && (
        <span className="gov-badge-item adr" title={`${adrCount} ADR${adrCount > 1 ? "s" : ""}`}>
          <ClipboardCheck size={10} />
          <span>{adrCount}</span>
        </span>
      )}
    </div>
  );
}

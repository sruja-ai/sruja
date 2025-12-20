// apps/designer/src/components/shared/BestPracticeTip.tsx
import { Lightbulb, AlertTriangle, CheckCircle2, Info, ExternalLink } from "lucide-react";
import { getPrimaryDocumentationUrl } from "../../utils/documentationService";
import "./BestPracticeTip.css";

export type TipVariant = "tip" | "warning" | "success" | "info";

interface BestPracticeTipProps {
  variant?: TipVariant;
  title?: string;
  children: React.ReactNode;
  show?: boolean;
  /** Optional step ID to show documentation link */
  stepId?: string;
  /** Custom documentation URL (overrides stepId) */
  docUrl?: string;
}

const VARIANT_CONFIG: Record<TipVariant, { icon: React.ReactNode; className: string }> = {
  tip: { icon: <Lightbulb size={16} />, className: "tip-tip" },
  warning: { icon: <AlertTriangle size={16} />, className: "tip-warning" },
  success: { icon: <CheckCircle2 size={16} />, className: "tip-success" },
  info: { icon: <Info size={16} />, className: "tip-info" },
};

/**
 * Best practice tip component with optional documentation link.
 * 
 * Displays contextual tips, warnings, or information to guide users.
 * Can optionally include a link to relevant documentation.
 * 
 * @param props - Tip configuration
 * @param props.variant - Tip variant: "tip", "warning", "success", or "info"
 * @param props.title - Optional title for the tip
 * @param props.children - Tip content
 * @param props.show - Whether to show the tip (default: true)
 * @param props.stepId - Builder step ID to auto-link documentation
 * @param props.docUrl - Custom documentation URL (overrides stepId)
 * @returns Best practice tip component
 * 
 * @example
 * ```tsx
 * <BestPracticeTip variant="tip" stepId="context">
 *   Start with actors — Identify who uses your system
 * </BestPracticeTip>
 * ```
 */
export function BestPracticeTip({
  variant = "tip",
  title,
  children,
  show = true,
  stepId,
  docUrl,
}: BestPracticeTipProps) {
  if (!show) return null;

  const config = VARIANT_CONFIG[variant];
  const documentationUrl = docUrl || (stepId ? getPrimaryDocumentationUrl(stepId) : null);

  return (
    <div className={`best-practice-tip ${config.className}`}>
      <div className="tip-icon">{config.icon}</div>
      <div className="tip-content">
        {title && <strong className="tip-title">{title}</strong>}
        <span className="tip-text">{children}</span>
        {documentationUrl && (
          <a
            href={documentationUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="tip-doc-link"
            onClick={(e) => e.stopPropagation()}
          >
            Learn more <ExternalLink size={12} />
          </a>
        )}
      </div>
    </div>
  );
}

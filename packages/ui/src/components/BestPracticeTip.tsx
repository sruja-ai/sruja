import { Lightbulb, AlertTriangle, CheckCircle2, Info, ExternalLink } from "lucide-react";

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
  /** Function to get documentation URL */
  getDocUrl?: (stepId: string) => string | null;
}

const VARIANT_CONFIG: Record<TipVariant, { icon: React.ReactNode; className: string }> = {
  tip: { icon: <Lightbulb size={16} />, className: "tip-tip" },
  warning: { icon: <AlertTriangle size={16} />, className: "tip-warning" },
  success: { icon: <CheckCircle2 size={16} />, className: "tip-success" },
  info: { icon: <Info size={16} />, className: "tip-info" },
};

export function BestPracticeTip({
  variant = "tip",
  title,
  children,
  show = true,
  stepId,
  docUrl,
  getDocUrl,
}: BestPracticeTipProps) {
  if (!show) return null;

  const config = VARIANT_CONFIG[variant];
  const documentationUrl = docUrl || (stepId && getDocUrl ? getDocUrl(stepId) : null);

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

export type { BestPracticeTipProps };

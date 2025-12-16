import { Lightbulb, AlertTriangle, CheckCircle2, Info } from "lucide-react";
import "./BestPracticeTip.css";

export type TipVariant = "tip" | "warning" | "success" | "info";

interface BestPracticeTipProps {
  variant?: TipVariant;
  title?: string;
  children: React.ReactNode;
  show?: boolean;
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
}: BestPracticeTipProps) {
  if (!show) return null;

  const config = VARIANT_CONFIG[variant];

  return (
    <div className={`best-practice-tip ${config.className}`}>
      <div className="tip-icon">{config.icon}</div>
      <div className="tip-content">
        {title && <strong className="tip-title">{title}</strong>}
        <span className="tip-text">{children}</span>
      </div>
    </div>
  );
}

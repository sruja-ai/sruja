// packages/ui/src/components/Toast.tsx
import { useEffect } from "react";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";
import "./Toast.css";

export type ToastType = "success" | "error" | "info" | "warning";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

interface ToastProps {
  toast: Toast;
  onClose: (id: string) => void;
}

export function ToastComponent({ toast, onClose }: ToastProps) {
  useEffect(() => {
    if (toast.duration !== 0) {
      const timer = setTimeout(() => {
        onClose(toast.id);
      }, toast.duration ?? 5000);
      return () => clearTimeout(timer);
    }
  }, [toast.id, toast.duration, onClose]);

  const icons = {
    success: <CheckCircle size={20} />,
    error: <AlertCircle size={20} />,
    info: <Info size={20} />,
    warning: <AlertTriangle size={20} />,
  };

  const colors = {
    success: "var(--toast-success)",
    error: "var(--toast-error)",
    info: "var(--toast-info)",
    warning: "var(--toast-warning)",
  };

  return (
    <div
      className={`toast toast-${toast.type}`}
      role="alert"
      aria-live="polite"
      style={{
        borderLeftColor: colors[toast.type],
      }}
    >
      <div className="toast-icon">{icons[toast.type]}</div>
      <div className="toast-content">
        <p className="toast-message">{toast.message}</p>
      </div>
      <button
        className="toast-close"
        onClick={() => onClose(toast.id)}
        aria-label="Close notification"
      >
        <X size={16} />
      </button>
    </div>
  );
}

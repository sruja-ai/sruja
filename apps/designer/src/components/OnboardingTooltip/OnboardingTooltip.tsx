import { useState, useEffect } from "react";
import { X, Lightbulb } from "lucide-react";
import { Button } from "@sruja/ui";
import "./OnboardingTooltip.css";

const ONBOARDING_DISMISSED_KEY = "sruja-onboarding-dismissed";

export function OnboardingTooltip() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Check if user has dismissed the tooltip before
    const dismissed = localStorage.getItem(ONBOARDING_DISMISSED_KEY);
    if (!dismissed) {
      // Show after a short delay
      const timer = setTimeout(() => {
        setVisible(true);
      }, 2000); // 2 seconds after page load

      return () => clearTimeout(timer);
    }
  }, []);

  const handleDismiss = () => {
    setVisible(false);
    localStorage.setItem(ONBOARDING_DISMISSED_KEY, "true");
  };

  if (!visible) return null;

  return (
    <div className="onboarding-tooltip">
      <div className="onboarding-tooltip-content">
        <div className="onboarding-tooltip-icon">
          <Lightbulb size={20} />
        </div>
        <div className="onboarding-tooltip-text">
          <strong>💡 Pro tip:</strong> Try editing in the <strong>Code tab</strong> - changes sync
          to the canvas automatically!
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="onboarding-tooltip-close"
          onClick={handleDismiss}
          aria-label="Dismiss"
        >
          <X size={16} />
        </Button>
      </div>
    </div>
  );
}

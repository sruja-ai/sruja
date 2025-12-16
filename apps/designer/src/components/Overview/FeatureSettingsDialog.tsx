import { Button, Dialog } from "@sruja/ui";
import { useFeatureFlagsStore, type FeatureFlags } from "../../stores/featureFlagsStore";
import "../Panels/OverviewPanel.css";

interface FeatureSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function FeatureSettingsDialog({ isOpen, onClose }: FeatureSettingsDialogProps) {
  const flags = useFeatureFlagsStore((s) => s.flags);
  const setFlag = useFeatureFlagsStore((s) => s.setFlag);
  const resetFlags = useFeatureFlagsStore((s) => s.resetFlags);

  const optionalFeatures: Array<{
    key: keyof FeatureFlags;
    label: string;
    description: string;
    complexity: "low" | "medium" | "high";
  }> = [
    {
      key: "policies",
      label: "Policies",
      description: "Architectural rules and constraints",
      complexity: "low",
    },
    {
      key: "metadata",
      label: "Metadata",
      description: "Key-value metadata entries",
      complexity: "low",
    },
    {
      key: "constraints",
      label: "Constraints",
      description: "System constraints and limitations",
      complexity: "low",
    },
    {
      key: "conventions",
      label: "Conventions",
      description: "Coding and naming conventions",
      complexity: "low",
    },
    {
      key: "deployment",
      label: "Deployment",
      description: "Deployment node configurations",
      complexity: "medium",
    },
    {
      key: "contracts",
      label: "Contracts",
      description: "API, event, and data contracts",
      complexity: "high",
    },
    {
      key: "sharedArtifacts",
      label: "Shared Artifacts",
      description: "Reusable components and services",
      complexity: "medium",
    },
    {
      key: "libraries",
      label: "Libraries",
      description: "Code libraries and frameworks",
      complexity: "medium",
    },
    { key: "imports", label: "Imports", description: "External DSL imports", complexity: "medium" },
  ];

  const getComplexityColor = (complexity: string) => {
    switch (complexity) {
      case "low":
        return "#10b981"; // green
      case "medium":
        return "#f59e0b"; // amber
      case "high":
        return "#ef4444"; // red
      default:
        return "#6b7280";
    }
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="Feature Settings"
      size="lg"
      footer={
        <>
          <Button variant="ghost" onClick={resetFlags} type="button">
            Reset to Defaults
          </Button>
          <Button variant="secondary" onClick={onClose} type="button">
            Close
          </Button>
        </>
      }
    >
      <div className="feature-settings">
        <div className="feature-settings-info">
          <p className="feature-settings-description">
            Enable or disable optional editing features. Mandatory features (Requirements, ADRs,
            Scenarios, Flows, Overview) are always enabled.
          </p>
        </div>
        <div className="feature-settings-list">
          {optionalFeatures.map((feature) => (
            <div key={feature.key} className="feature-setting-item">
              <div className="feature-setting-content">
                <div className="feature-setting-header">
                  <label className="feature-setting-label">
                    <input
                      type="checkbox"
                      checked={flags[feature.key] === true}
                      onChange={(e) => setFlag(feature.key, e.target.checked)}
                      disabled={
                        feature.key === "requirements" ||
                        feature.key === "adrs" ||
                        feature.key === "scenarios" ||
                        feature.key === "flows" ||
                        feature.key === "overview"
                      }
                    />
                    <span>{feature.label}</span>
                  </label>
                  <span
                    className="feature-complexity-badge"
                    style={{
                      backgroundColor: getComplexityColor(feature.complexity) + "20",
                      color: getComplexityColor(feature.complexity),
                    }}
                  >
                    {feature.complexity}
                  </span>
                </div>
                <p className="feature-setting-description">{feature.description}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </Dialog>
  );
}

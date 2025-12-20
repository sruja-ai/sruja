// apps/designer/src/components/Overview/FeatureSettingsDialog.tsx
import { Button, Dialog, Switch, Card } from "@sruja/ui";
import { useFeatureFlagsStore, type FeatureFlags } from "../../stores/featureFlagsStore";
import { useGridStore, type GridSize } from "../../stores/gridStore";
import { useUISettingsStore } from "../../stores/uiSettingsStore";
import "../Panels/OverviewPanel.css";

interface FeatureSettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function FeatureSettingsDialog({ isOpen, onClose }: FeatureSettingsDialogProps) {
  const flags = useFeatureFlagsStore((s) => s.flags);
  const setFlag = useFeatureFlagsStore((s) => s.setFlag);
  const resetFlags = useFeatureFlagsStore((s) => s.resetFlags);

  // Grid settings
  const gridEnabled = useGridStore((s) => s.enabled);
  const gridSize = useGridStore((s) => s.size);
  const snapToGrid = useGridStore((s) => s.snapToGrid);
  const setGridEnabled = useGridStore((s) => s.setEnabled);
  const setGridSize = useGridStore((s) => s.setSize);
  const setSnapToGrid = useGridStore((s) => s.setSnapToGrid);

  // MiniMap settings
  const showMinimap = useUISettingsStore((s) => s.showMinimap);
  const setShowMinimap = useUISettingsStore((s) => s.setShowMinimap);

  // Layout engine is now always the consolidated modular engine - no settings needed

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
      { key: "imports", label: "Imports", description: "LikeC4 cross-project imports", complexity: "medium" },
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
      title="Settings"
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
      <div className="feature-settings" style={{ maxHeight: "70vh", overflowY: "auto", paddingRight: "4px" }}>
        <div className="feature-settings-info">
          <p className="feature-settings-description">
            Configure optional editing features and display preferences. Mandatory features are always enabled.
          </p>
        </div>

        {/* Optional Features - Grid Layout */}
        <div className="feature-settings-section">
          <h3 className="feature-settings-section-title">Optional Features</h3>
          <div className="feature-settings-grid">
            {optionalFeatures.map((feature) => (
              <Card key={feature.key} className="feature-setting-card" interactive={false}>
                <div className="feature-setting-card-content">
                  <div className="feature-setting-card-header">
                    <div className="feature-setting-card-title-group">
                      <span className="feature-setting-card-title">{feature.label}</span>
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
                    <Switch
                      checked={flags[feature.key] === true}
                      onChange={(checked) => setFlag(feature.key, checked)}
                    />
                  </div>
                  <p className="feature-setting-card-description">{feature.description}</p>
                </div>
              </Card>
            ))}
          </div>
        </div>

        {/* Grid & Alignment Settings */}
        <div className="feature-settings-section">
          <h3 className="feature-settings-section-title">Grid & Alignment</h3>
          <div className="feature-settings-compact">
            <Card className="feature-setting-card" interactive={false}>
              <div className="feature-setting-card-content">
                <div className="feature-setting-card-header">
                  <div className="feature-setting-card-title-group">
                    <span className="feature-setting-card-title">Show Grid</span>
                  </div>
                  <Switch checked={gridEnabled} onChange={setGridEnabled} />
                </div>
                <p className="feature-setting-card-description">
                  Display a visual grid overlay on the canvas for better alignment
                </p>
              </div>
            </Card>

            <Card className="feature-setting-card" interactive={false}>
              <div className="feature-setting-card-content">
                <div className="feature-setting-card-header">
                  <div className="feature-setting-card-title-group">
                    <span className="feature-setting-card-title">Snap to Grid</span>
                  </div>
                  <Switch checked={snapToGrid} onChange={setSnapToGrid} disabled={!gridEnabled} />
                </div>
                <p className="feature-setting-card-description">
                  Automatically align nodes to grid positions when dragging
                </p>
              </div>
            </Card>

            <Card className="feature-setting-card" interactive={false}>
              <div className="feature-setting-card-content">
                <div className="feature-setting-card-header">
                  <div className="feature-setting-card-title-group">
                    <span className="feature-setting-card-title">Grid Size</span>
                  </div>
                </div>
                <p className="feature-setting-card-description">Size of the grid squares in pixels</p>
                <div className="feature-setting-options">
                  {([8, 16, 32] as GridSize[]).map((size) => (
                    <Button
                      key={size}
                      variant={gridSize === size ? "primary" : "secondary"}
                      onClick={() => setGridSize(size)}
                      disabled={!gridEnabled}
                      className="feature-setting-option-btn"
                    >
                      {size}px
                    </Button>
                  ))}
                </div>
              </div>
            </Card>
          </div>
        </div>

        {/* Navigation Settings */}
        <div className="feature-settings-section">
          <h3 className="feature-settings-section-title">Navigation</h3>
          <div className="feature-settings-compact">
            <Card className="feature-setting-card" interactive={false}>
              <div className="feature-setting-card-content">
                <div className="feature-setting-card-header">
                  <div className="feature-setting-card-title-group">
                    <span className="feature-setting-card-title">Show MiniMap</span>
                  </div>
                  <Switch checked={showMinimap} onChange={setShowMinimap} />
                </div>
                <p className="feature-setting-card-description">
                  Display a minimap overview in the bottom-right corner. Click to jump to different areas of the diagram.
                </p>
              </div>
            </Card>
          </div>
        </div>

        {/* Layout engine is now always the consolidated modular engine - no settings needed */}
      </div>
    </Dialog>
  );
}

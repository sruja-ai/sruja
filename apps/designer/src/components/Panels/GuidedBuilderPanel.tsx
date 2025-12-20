import { useState } from "react";
import { Button } from "@sruja/ui";
import {
  CheckCircle,
  AlertCircle,
  Target,
  Layout,
  FileCode,
  ChevronDown,
  ChevronUp,
  BookOpen,
  Lightbulb,
  Info,
} from "lucide-react";
import { useUIStore } from "../../stores/uiStore";
import { useBuilderProgress } from "../../hooks/useBuilderProgress";
import { BuilderL1Context } from "./Builder/BuilderL1Context";
import { BuilderL2Container } from "./Builder/BuilderL2Container";
import { BuilderL3Component } from "./Builder/BuilderL3Component";
import "./GuidedBuilderPanel.css";

export function GuidedBuilderPanel() {
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const {
    l1Complete,
    l1Progress,
    l2Progress,
    l3Progress,
  } = useBuilderProgress();

  // Need to reconstruct the boolean flags for "Complete" status based on hook data
  // Logic from original file:
  // L2 Complete: systems > 0 && all systems have containers && no missing relations
  // L2 Complete check
  const l2Complete = l2Progress.percentage >= 100;

  // L3 Complete check
  const l3Complete = l3Progress.percentage >= 100;

  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    l1: true,
    l2: true,
    l3: true,
  });
  const [showWelcome, setShowWelcome] = useState(true);

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => ({ ...prev, [section]: !prev[section] }));
  };

  return (
    <div className="guided-panel">
      {showWelcome && (
        <div className="guided-welcome">
          <div className="guided-welcome-header">
            <div className="guided-welcome-icon">
              <BookOpen size={24} />
            </div>
            <div className="guided-welcome-content">
              <h3>Architecture Builder Guide</h3>
              <p>
                Follow this step-by-step guide to build a complete C4 architecture. Start with the
                system context (L1) and work your way down to components (L3).
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              className="guided-welcome-close"
              onClick={() => setShowWelcome(false)}
              aria-label="Close welcome"
            >
              ×
            </Button>
          </div>
          <div className="guided-welcome-tips">
            <div className="guided-tip">
              <Lightbulb size={16} />
              <span>Each level builds upon the previous one. Complete L1 before moving to L2.</span>
            </div>
            <div className="guided-tip">
              <Target size={16} />
              <span>
                Use the checklist items to track your progress. Green checkmarks indicate
                completion.
              </span>
            </div>
            <div className="guided-tip">
              <Info size={16} />
              <span>
                Optional items (marked with info icon) enhance your architecture but aren't
                required.
              </span>
            </div>
          </div>
        </div>
      )}

      {/* L1 Section */}
      <div className="guided-section">
        <div className="guided-section-header" onClick={() => toggleSection("l1")}>
          <div className="guided-section-title-group">
            {expandedSections.l1 ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
            <div className="guided-section-title-content">
              <div className="guided-status-row">
                {l1Complete ? (
                  <CheckCircle size={16} color="#22c55e" />
                ) : (
                  <AlertCircle size={16} color="#f59e0b" />
                )}
                <span className={`guided-status ${l1Complete ? "ok" : "warn"}`}>
                  {l1Complete ? "Complete" : "Incomplete"}
                </span>
                <div className="guided-progress-bar">
                  <div
                    className="guided-progress-fill"
                    style={{ width: `${l1Progress.percentage}%` }}
                  />
                </div>
                <span className="guided-progress-text">
                  {l1Progress.completed}/{l1Progress.total}
                </span>
              </div>
              <div className="guided-header">
                <span>L1: System Context</span>
                <div className="guided-header-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("diagram");
                    }}
                    title="View Diagram"
                  >
                    <Layout size={14} />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("code");
                    }}
                    title="View Code"
                  >
                    <FileCode size={14} />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
        {expandedSections.l1 && <BuilderL1Context />}
      </div>

      {/* L2 Section */}
      <div className="guided-section">
        <div className="guided-section-header" onClick={() => toggleSection("l2")}>
          <div className="guided-section-title-group">
            {expandedSections.l2 ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
            <div className="guided-section-title-content">
              <div className="guided-status-row">
                {l2Complete ? (
                  <CheckCircle size={16} color="#22c55e" />
                ) : (
                  <AlertCircle size={16} color="#f59e0b" />
                )}
                <span className={`guided-status ${l2Complete ? "ok" : "warn"}`}>
                  {l2Complete ? "Complete" : "Incomplete"}
                </span>
                <div className="guided-progress-bar">
                  <div
                    className="guided-progress-fill"
                    style={{ width: `${l2Progress.percentage}%` }}
                  />
                </div>
                <span className="guided-progress-text">
                  {l2Progress.completed}/{l2Progress.total}
                </span>
              </div>
              <div className="guided-header">
                <span>L2: Container View</span>
                <div className="guided-header-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("diagram");
                    }}
                    title="View Diagram"
                  >
                    <Layout size={14} />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("code");
                    }}
                    title="View Code"
                  >
                    <FileCode size={14} />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
        {expandedSections.l2 && <BuilderL2Container />}
      </div>

      {/* L3 Section */}
      <div className="guided-section">
        <div className="guided-section-header" onClick={() => toggleSection("l3")}>
          <div className="guided-section-title-group">
            {expandedSections.l3 ? <ChevronDown size={18} /> : <ChevronUp size={18} />}
            <div className="guided-section-title-content">
              <div className="guided-status-row">
                {l3Complete ? (
                  <CheckCircle size={16} color="#22c55e" />
                ) : (
                  <AlertCircle size={16} color="#f59e0b" />
                )}
                <span className={`guided-status ${l3Complete ? "ok" : "warn"}`}>
                  {l3Complete ? "Complete" : "Incomplete"}
                </span>
                <div className="guided-progress-bar">
                  <div
                    className="guided-progress-fill"
                    style={{ width: `${l3Progress.percentage}%` }}
                  />
                </div>
                <span className="guided-progress-text">
                  {l3Progress.completed}/{l3Progress.total}
                </span>
              </div>
              <div className="guided-header">
                <span>L3: Component View</span>
                <div className="guided-header-actions">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("diagram");
                    }}
                    title="View Diagram"
                  >
                    <Layout size={14} />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setActiveTab("code");
                    }}
                    title="View Code"
                  >
                    <FileCode size={14} />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
        {expandedSections.l3 && <BuilderL3Component />}
      </div>
    </div>
  );
}

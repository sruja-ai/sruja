import { useState, useMemo, useEffect } from "react";
import { Share2, Eye } from "lucide-react";
import { Button } from "@sruja/ui";
import { WizardStepper } from "./WizardStepper";
import type { WizardStep } from "./WizardStepper";
import { GoalsStep } from "./GoalsStep";
import { SystemContextStep } from "./SystemContextStep";
import { ContainersStep } from "./ContainersStep";
import { ComponentsStep } from "./ComponentsStep";
import { FlowsStep } from "./FlowsStep";
import { DslPreview } from "./DslPreview";
import { ValidationPanel } from "./ValidationPanel";
import { SharePanel } from "./SharePanel";
import { DocumentationPanel } from "./DocumentationPanel";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useValidation } from "../../hooks/useValidation";
import { convertModelToDsl } from "../../utils/modelToDsl";
import "./BuilderWizard.css";

export function BuilderWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const [showShare, setShowShare] = useState(false);
  const [showSidebar, setShowSidebar] = useState(false);
  const [hasSidebarPref, setHasSidebarPref] = useState(false);
  const { score } = useValidation();
  const data = useArchitectureStore((state) => state.likec4Model);
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const setDslSource = useArchitectureStore((s) => s.setDslSource);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Generate current DSL for preview
  // Prefer store's dslSource (original DSL) over regenerating from JSON
  // This ensures DSL persists across tab changes
  const [currentDsl, setCurrentDsl] = useState<string>("// No architecture loaded");

  useEffect(() => {
    // Use stored DSL source if available (preserves original DSL)
    if (storeDslSource && storeDslSource.trim().length > 0) {
      setCurrentDsl(storeDslSource);
      return;
    }

    // Fall back to generating from JSON if no stored DSL
    if (!data) {
      setCurrentDsl("// No architecture loaded");
      return;
    }

    const generateDsl = async () => {
      try {
        const dsl = await convertModelToDsl(data);
        // Ensure we always return something meaningful
        if (!dsl || dsl.trim().length === 0) {
          setCurrentDsl("// No DSL content available");
        } else {
          setCurrentDsl(dsl);
        }
      } catch (error) {
        console.error("Error generating DSL:", error);
        setCurrentDsl(
          `// Error generating DSL: ${error instanceof Error ? error.message : "Unknown error"}`
        );
      }
    };

    void generateDsl();
  }, [storeDslSource, data]);

  // Ensure DSL is stored when generated (so it persists across tab changes)
  useEffect(() => {
    if (currentDsl && currentDsl.trim().length > 0 && !currentDsl.startsWith("//")) {
      // Only update if we don't have a stored DSL source or if it's different
      if (!storeDslSource || storeDslSource !== currentDsl) {
        // Only store if it's a valid DSL (not an error message)
        if (!currentDsl.includes("Error") && !currentDsl.includes("No architecture")) {
          setDslSource(currentDsl, null);
        }
      }
    }
  }, [currentDsl, storeDslSource, setDslSource]);

  // Calculate completion status for each step
  const steps: WizardStep[] = useMemo(() => {
    // SrujaModelDump uses flat elements map
    const elements = data?.elements ? Object.values(data.elements) : [];

    // Requirements are in sruja.requirements
    const requirements = data?.sruja?.requirements ?? [];
    // Goals removed as they are not in SrujaModelDump and assume Requirements cover it.

    const systems = elements.filter((e: any) => e.kind === "system");
    const persons = elements.filter(
      (e: any) => e.kind === "person" || e.kind === "actor" || e.kind === "user"
    );
    const allContainers = elements.filter(
      (e: any) =>
        e.kind === "container" ||
        e.kind === "webapp" ||
        e.kind === "mobile" ||
        e.kind === "api" ||
        e.kind === "database" ||
        e.kind === "queue"
    );
    const allComponents = elements.filter((e: any) => e.kind === "component");

    const scenarios = data?.sruja?.scenarios ?? [];

    const hasGoalsOrReqs = requirements.length > 0;
    const hasContext = systems.length > 0 || persons.length > 0;
    const hasContainers = allContainers.length > 0;
    const hasComponents = allComponents.length > 0;
    const hasFlows = scenarios.length > 0;

    return [
      {
        id: "goals",
        label: "Define",
        description: "Goals & requirements",
        isComplete: hasGoalsOrReqs,
        isLocked: false,
      },
      {
        id: "context",
        label: "Context",
        description: "Actors & systems",
        isComplete: hasContext,
        isLocked: false,
      },
      {
        id: "containers",
        label: "Containers",
        description: "Apps & databases",
        isComplete: hasContainers,
        isLocked: !hasContext,
      },
      {
        id: "components",
        label: "Components",
        description: "Internal details",
        isComplete: hasComponents,
        isLocked: !hasContainers,
      },
      {
        id: "flows",
        label: "Flows",
        description: "Scenarios",
        isComplete: hasFlows,
        isLocked: false,
      },
    ];
  }, [data]);

  // Initialize state from local storage on mount (Metadata persistence deprecated/removed for now)
  useEffect(() => {
    // Load step from local storage
    const storedStep = window.localStorage.getItem("playground:wizard:step");
    if (storedStep) {
      const step = parseInt(storedStep, 10);
      if (!isNaN(step) && step >= 0 && step < steps.length) {
        setCurrentStep(step);
      }
    }

    // Load sidebar pref from local storage
    const pref = window.localStorage.getItem("playground:previewSidebar");
    if (pref === "on") {
      setShowSidebar(true);
      setHasSidebarPref(true);
    } else if (pref === "off") {
      setShowSidebar(false);
      setHasSidebarPref(true);
    }
  }, [steps.length]); // Run on mount or when steps change
  // Actually, we only want to load on mount or when we switch projects (loaded data changes significantly)
  // But since we write back to metadata, we need to avoid reading back our own writes in a loop.
  // The simple way: Only read on initial load of a new architecture.
  // We can track the last loaded ID or similar?
  // Limitation: If another user updates the step, we might want to sync it?
  // For now, let's load once per session/project load.
  // We can use a ref to track if we've initialized for the current data signature.

  // Helper to update metadata (now local storage)
  const updateMetadata = (key: string, value: string) => {
    // Map keys to local storage keys if needed, or just use raw
    if (key === "playground:wizard:step") {
      window.localStorage.setItem("playground:wizard:step", value);
    } else if (key === "playground:wizard:sidebar") {
      // Handled by specific sidebar logic usually
    }
    // We don't update store metadata anymore as SrujaModelDump structure differs
  };

  const goToStep = (stepIndex: number) => {
    if (!steps[stepIndex].isLocked) {
      setCurrentStep(stepIndex);
      if (isEditMode()) {
        updateMetadata("playground:wizard:step", stepIndex.toString());
      }
    }
  };

  const nextStep = () => {
    if (currentStep < steps.length - 1) {
      const next = currentStep + 1;
      setCurrentStep(next);
      updateMetadata("playground:wizard:step", next.toString());
    }
  };

  const prevStep = () => {
    if (currentStep > 0) {
      const prev = currentStep - 1;
      setCurrentStep(prev);
      updateMetadata("playground:wizard:step", prev.toString());
    }
  };

  useEffect(() => {
    if (currentStep === 4 && !hasSidebarPref && !showSidebar) {
      setShowSidebar(true);
      // Don't auto-persist this as user preference yet, just local conveniences
    }
  }, [currentStep, hasSidebarPref, showSidebar]);

  return (
    <div className={`builder-wizard ${isEditMode() ? "edit-mode" : "view-mode"}`}>
      {!isEditMode() && (
        <div className="builder-view-mode-banner">
          <Eye size={16} />
          <span>View Mode - This is a read-only guide. Switch to edit mode to make changes.</span>
        </div>
      )}
      <WizardStepper
        steps={steps}
        currentStep={currentStep}
        onStepClick={goToStep}
        extraActions={
          <>
            <Button
              variant={showSidebar ? "primary" : "ghost"}
              size="sm"
              className={`preview-toggle-btn ${showSidebar ? "active" : ""}`}
              onClick={() => {
                setShowSidebar((v) => {
                  const next = !v;
                  if (isEditMode()) {
                    updateMetadata("playground:wizard:sidebar", next.toString());
                  }
                  try {
                    window.localStorage.setItem("playground:previewSidebar", next ? "on" : "off");
                    setHasSidebarPref(true);
                  } catch {}
                  return next;
                });
              }}
              title="Toggle preview sidebar (DSL, Documentation, Validation)"
            >
              {showSidebar ? "Hide Preview & Docs" : "Show Preview & Docs"}
            </Button>
          </>
        }
      />

      <div className="wizard-content">
        <div className="wizard-main">
          {currentStep === 0 && <GoalsStep onNext={nextStep} readOnly={!isEditMode()} />}
          {currentStep === 1 && (
            <SystemContextStep onNext={nextStep} onBack={prevStep} readOnly={!isEditMode()} />
          )}
          {currentStep === 2 && (
            <ContainersStep onNext={nextStep} onBack={prevStep} readOnly={!isEditMode()} />
          )}
          {currentStep === 3 && (
            <ComponentsStep onBack={prevStep} onFinish={nextStep} readOnly={!isEditMode()} />
          )}
          {currentStep === 4 && <FlowsStep onBack={prevStep} readOnly={!isEditMode()} />}
        </div>

        {/* Preview Sidebar (Documentation, Validation, DSL) */}
        {showSidebar && (
          <div className="wizard-sidebar">
            <div className="sidebar-actions">
              <Button variant="ghost" size="sm" onClick={() => setShowSidebar(!showSidebar)}>
                <Eye size={16} />
                {showSidebar ? "Hide Details" : "Show Details"}
              </Button>
              <div
                className={`quality-badge ${score >= 80 ? "good" : score >= 50 ? "avg" : "poor"}`}
              >
                <div className="quality-label">Quality Score</div>
                <div className="quality-value">{score}</div>
              </div>
              <Button variant="outline" size="sm" onClick={() => setShowShare(!showShare)}>
                <Share2 size={14} />
                Share
              </Button>
            </div>
            {/* Documentation Panel - Step-specific help */}
            <DocumentationPanel stepId={steps[currentStep]?.id || "goals"} compact />
            {/* Validation Panel - Quality checks */}
            <ValidationPanel compact />
            {/* DSL Preview - Generated code */}
            <DslPreview dsl={currentDsl} />
          </div>
        )}
      </div>

      <SharePanel isOpen={showShare} onClose={() => setShowShare(false)} />
    </div>
  );
}

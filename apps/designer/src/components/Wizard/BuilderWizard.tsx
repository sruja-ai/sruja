import { useState, useMemo, useEffect } from "react";
import { Share2 } from "lucide-react";
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
import { useArchitectureStore } from "../../stores/architectureStore";
import { convertJsonToDsl } from "../../utils/jsonToDsl";
import "./BuilderWizard.css";

export function BuilderWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const [showShare, setShowShare] = useState(false);
  const [showSidebar, setShowSidebar] = useState(false);
  const [hasSidebarPref, setHasSidebarPref] = useState(false);
  const data = useArchitectureStore((s) => s.data);

  // Generate current DSL for preview
  const currentDsl = useMemo(() => {
    if (!data) return "// No architecture loaded";
    try {
      return convertJsonToDsl(data);
    } catch {
      return "// Error generating DSL";
    }
  }, [data]);

  // Calculate completion status for each step
  const steps: WizardStep[] = useMemo(() => {
    const goals = data?.architecture?.overview?.goals ?? [];
    const requirements = data?.architecture?.requirements ?? [];
    const systems = data?.architecture?.systems ?? [];
    const persons = data?.architecture?.persons ?? [];
    const allContainers = systems.flatMap((s) => s.containers ?? []);
    const allComponents = allContainers.flatMap((c) => c.components ?? []);
    const scenarios = data?.architecture?.scenarios ?? [];

    const hasGoalsOrReqs = goals.length > 0 || requirements.length > 0;
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

  // Initialize state from metadata on mount
  useEffect(() => {
    if (!data?.architecture) return;

    // Load step from metadata
    const stepMeta = data.architecture.archMetadata?.find(
      (m) => m.key === "playground:wizard:step"
    );
    if (stepMeta?.value) {
      const step = parseInt(stepMeta.value, 10);
      if (!isNaN(step) && step >= 0 && step < steps.length) {
        setCurrentStep(step);
      }
    }

    // Load sidebar pref from metadata (overrides local storage if present)
    const sidebarMeta = data.architecture.archMetadata?.find(
      (m) => m.key === "playground:wizard:sidebar"
    );
    if (sidebarMeta?.value) {
      setShowSidebar(sidebarMeta.value === "true");
      setHasSidebarPref(true);
    } else {
      // Fallback to local storage (for first load)
      const pref =
        typeof window !== "undefined"
          ? window.localStorage.getItem("playground:previewSidebar")
          : null;
      if (pref === "on") {
        setShowSidebar(true);
        setHasSidebarPref(true);
      } else if (pref === "off") {
        setShowSidebar(false);
        setHasSidebarPref(true);
      }
    }
  }, [data?.architecture?.archMetadata]); // Only re-run when metadata loads/changes structurally (careful with loops)
  // Actually, we only want to load on mount or when we switch projects (loaded data changes significantly)
  // But since we write back to metadata, we need to avoid reading back our own writes in a loop.
  // The simple way: Only read on initial load of a new architecture.
  // We can track the last loaded ID or similar?
  // Limitation: If another user updates the step, we might want to sync it?
  // For now, let's load once per session/project load.
  // We can use a ref to track if we've initialized for the current data signature.

  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  // Helper to update metadata
  const updateMetadata = (key: string, value: string) => {
    updateArchitecture((arch) => {
      const metadata = arch.architecture.archMetadata || [];
      const index = metadata.findIndex((m) => m.key === key);
      if (index >= 0) {
        const newMeta = [...metadata];
        newMeta[index] = { ...newMeta[index], value };
        return { ...arch, architecture: { ...arch.architecture, archMetadata: newMeta } };
      } else {
        return {
          ...arch,
          architecture: { ...arch.architecture, archMetadata: [...metadata, { key, value }] },
        };
      }
    });
  };

  const goToStep = (stepIndex: number) => {
    if (!steps[stepIndex].isLocked) {
      setCurrentStep(stepIndex);
      updateMetadata("playground:wizard:step", stepIndex.toString());
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
    <div className="builder-wizard">
      <WizardStepper
        steps={steps}
        currentStep={currentStep}
        onStepClick={goToStep}
        extraActions={
          <button
            className={`preview-toggle-btn ${showSidebar ? "active" : ""}`}
            onClick={() => {
              setShowSidebar((v) => {
                const next = !v;
                updateMetadata("playground:wizard:sidebar", next.toString());
                try {
                  window.localStorage.setItem("playground:previewSidebar", next ? "on" : "off");
                  setHasSidebarPref(true);
                } catch {}
                return next;
              });
            }}
          >
            {showSidebar ? "Hide Preview" : "Show Preview"}
          </button>
        }
      />

      <div className="wizard-content">
        <div className="wizard-main">
          {currentStep === 0 && <GoalsStep onNext={nextStep} />}
          {currentStep === 1 && <SystemContextStep onNext={nextStep} onBack={prevStep} />}
          {currentStep === 2 && <ContainersStep onNext={nextStep} onBack={prevStep} />}
          {currentStep === 3 && <ComponentsStep onBack={prevStep} onFinish={nextStep} />}
          {currentStep === 4 && <FlowsStep onBack={prevStep} />}
        </div>

        {/* DSL Preview Sidebar */}
        {showSidebar && (
          <div className="wizard-sidebar">
            <div className="sidebar-actions">
              <button className="share-btn" onClick={() => setShowShare(true)}>
                <Share2 size={14} />
                Share
              </button>
            </div>
            <ValidationPanel compact />
            <DslPreview dsl={currentDsl} />
          </div>
        )}
      </div>

      <SharePanel isOpen={showShare} onClose={() => setShowShare(false)} />
    </div>
  );
}

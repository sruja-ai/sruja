import { useState, useMemo, useEffect } from "react";
import { logger } from "@sruja/shared";
import type { ElementDump } from "@sruja/shared";
import { WizardStepper, type WizardStep } from "./WizardStepper";
import { GoalsStep } from "./GoalsStep";
import { SystemContextStep } from "./SystemContextStep";
import { ContainersStep } from "./ContainersStep";
import { ComponentsStep } from "./ComponentsStep";
import { FlowsStep } from "./FlowsStep";
import { RolesViewsStep } from "./RolesViewsStep";
// import { DslPreview } from "./DslPreview"; // Handled by Right Pane now
// import { ValidationPanel } from "./ValidationPanel"; // Handled by Right Pane now
// import { DocumentationPanel } from "./DocumentationPanel"; // Handled by Right Pane now
import { SharePanel } from "./SharePanel";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useUIStore } from "../../stores"; // Import UI Store
import { convertModelToDsl } from "../../utils/modelToDsl";
import "./BuilderWizard.css";

export function BuilderWizard() {
  const [showShare, setShowShare] = useState(false);

  // Use global store for step state
  const builderStepId = useUIStore((s) => s.builderStep);
  const setBuilderStepId = useUIStore((s) => s.setBuilderStep);

  // const { score } = useValidation();
  const data = useArchitectureStore((state) => state.model);
  const storeDslSource = useArchitectureStore((s) => s.dslSource);
  const sourceType = useArchitectureStore((s) => s.sourceType);
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
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("DSL generation failed", {
          component: "BuilderWizard",
          action: "generateDSL",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        setCurrentDsl(`// Error generating DSL: ${errorMessage}`);
      }
    };

    void generateDsl();
  }, [storeDslSource, data]);

  // Ensure DSL is stored when generated (so it persists across tab changes)
  useEffect(() => {
    // Skip updating if DSL was manually edited (sourceType === "dsl")
    // This prevents a loop between Builder and DSL panel
    if (sourceType === "dsl") {
      return;
    }

    if (currentDsl && currentDsl.trim().length > 0 && !currentDsl.startsWith("//")) {
      // Only update if we don't have a stored DSL source or if it's different
      if (!storeDslSource || storeDslSource !== currentDsl) {
        // Only store if it's a valid DSL (not an error message)
        if (!currentDsl.includes("Error") && !currentDsl.includes("No architecture")) {
          setDslSource(currentDsl, null);
        }
      }
    }
  }, [currentDsl, storeDslSource, setDslSource, sourceType]);

  // Calculate completion status for each step
  const steps: WizardStep[] = useMemo(() => {
    // SrujaModelDump uses flat elements map
    const elements: ElementDump[] = data?.elements ? Object.values(data.elements) : [];

    // Requirements are in sruja.requirements
    const requirements = data?.sruja?.requirements ?? [];
    // Goals removed as they are not in SrujaModelDump and assume Requirements cover it.

    // Type guard functions for element filtering
    const isSystem = (e: ElementDump): boolean => e.kind === "system";
    const isPerson = (e: ElementDump): boolean =>
      e.kind === "person" || e.kind === "actor" || e.kind === "user";
    const isContainer = (e: ElementDump): boolean =>
      e.kind === "container" ||
      e.kind === "webapp" ||
      e.kind === "mobile" ||
      e.kind === "api" ||
      e.kind === "database" ||
      e.kind === "queue";
    const isComponent = (e: ElementDump): boolean => e.kind === "component";

    const systems = elements.filter(isSystem);
    const persons = elements.filter(isPerson);
    const allContainers = elements.filter(isContainer);
    const allComponents = elements.filter(isComponent);

    const scenarios = data?.sruja?.scenarios ?? [];

    const hasGoalsOrReqs = requirements.length > 0;
    const hasContext = systems.length > 0 || persons.length > 0;
    const hasContainers = allContainers.length > 0;
    const hasComponents = allComponents.length > 0;
    const hasFlows = scenarios.length > 0;

    // Check for roles
    const roles = elements.filter((e) => e.kind === "role");
    const hasRoles = roles.length > 0;

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
      {
        id: "roles-views",
        label: "Roles & Views",
        description: "Organizational roles",
        isComplete: hasRoles,
        isLocked: false,
      },
    ];
  }, [data]);

  // Derive numeric index from store ID
  const currentStep = useMemo(() => {
    const idx = steps.findIndex((s) => s.id === builderStepId);
    return idx >= 0 ? idx : 0;
  }, [steps, builderStepId]);

  const goToStep = (stepIndex: number) => {
    if (!steps[stepIndex].isLocked) {
      setBuilderStepId(steps[stepIndex].id);
    }
  };

  const nextStep = () => {
    if (currentStep < steps.length - 1) {
      const next = currentStep + 1;
      setBuilderStepId(steps[next].id);
    }
  };

  const prevStep = () => {
    if (currentStep > 0) {
      const prev = currentStep - 1;
      setBuilderStepId(steps[prev].id);
    }
  };

  return (
    <div className={`builder-wizard ${isEditMode() ? "edit-mode" : "view-mode"}`}>
      {/* 
        Removed "View Mode" banner as it is prominent in split view context.
        The UI itself should look clean.
      */}
      <WizardStepper
        steps={steps}
        currentStep={currentStep}
        onStepClick={goToStep}
        onClose={() => useUIStore.getState().setLeftPaneContent("none")}
        extraActions={
          // Simplified: No preview toggle anymore as it's controlled by Layout
          null
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
          {currentStep === 5 && <RolesViewsStep onBack={prevStep} readOnly={!isEditMode()} />}
        </div>
      </div>

      <SharePanel isOpen={showShare} onClose={() => setShowShare(false)} />
    </div>
  );
}

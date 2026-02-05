import { useState, useMemo, useEffect } from "react";
import { logger } from "@sruja/shared";
import type { ElementDump } from "@sruja/shared";
import { WizardStepper, type WizardStep } from "./WizardStepper";
import { SystemContextStep } from "./SystemContextStep";
import { ContainersStep } from "./ContainersStep";
import { ComponentsStep } from "./ComponentsStep";
import { GoalsStep } from "../non-core/wizard/GoalsStep";
import { FlowsStep } from "../non-core/wizard/FlowsStep";
import { RolesViewsStep } from "../non-core/wizard/RolesViewsStep";
// import { DslPreview } from "./DslPreview"; // Handled by Right Pane now
// import { ValidationPanel } from "./ValidationPanel"; // Handled by Right Pane now
// import { DocumentationPanel } from "./DocumentationPanel"; // Handled by Right Pane now
import { SharePanel } from "../non-core/wizard/SharePanel";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import { useUIStore } from "../../stores"; // Import UI Store
import { convertModelToDsl } from "../../utils/modelToDsl";
import { studioScope } from "../../config/studioScope";
import "./BuilderWizard.css";

export function BuilderWizard() {
  const [showShare, setShowShare] = useState(false);
  const [quickStartMode, setQuickStartMode] = useState<boolean | null>(null);

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

    const baseSteps: WizardStep[] = [
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
    ];

    if (studioScope.builderFlows) {
      baseSteps.push({
        id: "flows",
        label: "Flows",
        description: "Scenarios (optional)",
        isComplete: hasFlows,
        isLocked: false,
        isOptional: true,
      });
    }

    if (studioScope.builderRoles) {
      baseSteps.push({
        id: "roles-views",
        label: "Roles & Views",
        description: "Role perspectives (optional)",
        isComplete: hasRoles,
        isLocked: false,
        isOptional: true,
      });
    }

    if (studioScope.builderGoals) {
      baseSteps.push({
        id: "goals",
        label: "Define",
        description: "Goals & requirements (optional)",
        isComplete: hasGoalsOrReqs,
        isLocked: false,
        isOptional: true,
      });
    }

    return baseSteps;
  }, [data]);

  // Check if we should show welcome/quick start option
  // Show if: no elements exist yet AND user hasn't chosen a mode
  const shouldShowWelcome = useMemo(() => {
    const elements = data?.elements ? Object.values(data.elements) : [];
    return elements.length === 0 && quickStartMode === null;
  }, [data?.elements, quickStartMode]);

  // Derive numeric index from store ID
  const currentStep = useMemo(() => {
    const idx = steps.findIndex((s) => s.id === builderStepId);
    // Default to first step (Context) if not found
    return idx >= 0 ? idx : 0;
  }, [steps, builderStepId]);

  const activeStepId = steps[currentStep]?.id;

  const goToStep = (stepIndex: number) => {
    // Allow jumping to any step, but show a warning if prerequisites aren't met
    const step = steps[stepIndex];
    if (step.isLocked) {
      // Still allow navigation but user should know prerequisites aren't met
      // This gives flexibility while maintaining guidance
    }
    setBuilderStepId(step.id);
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

  // Handle quick start mode selection
  const handleQuickStart = () => {
    setQuickStartMode(true);
    setBuilderStepId("context"); // Start with context (C4 Level 1)
  };

  const handleGuidedMode = () => {
    setQuickStartMode(false);
    setBuilderStepId("context"); // Start with context (C4 Level 1)
  };

  // Show welcome screen for new users
  if (shouldShowWelcome) {
    return (
      <div className={`builder-wizard ${isEditMode() ? "edit-mode" : "view-mode"}`}>
        <div className="welcome-step">
          <div className="welcome-header">
            <h2>Build Your Architecture</h2>
            <p>Choose how you'd like to get started</p>
          </div>

          <div className="welcome-options">
            <div className="welcome-card">
              <h3>🚀 Quick Start</h3>
              <p>Build your architecture visually and iterate as you go.</p>
              <ul>
                <li>✓ Start with C4 architecture (Context → Containers → Components)</li>
                <li>✓ See your diagram as you build</li>
                {studioScope.builderGoals && <li>✓ Add goals & requirements at the end</li>}
                {studioScope.builderGoals && <li>✓ Tag requirements to architecture elements</li>}
              </ul>
              <button className="btn-primary" onClick={handleQuickStart}>
                Start Building
              </button>
            </div>

            <div className="welcome-card">
              <h3>📚 Guided Mode</h3>
              <p>Step-by-step wizard following C4 model structure.</p>
              <ul>
                <li>✓ Follow C4 model (Context → Containers → Components)</li>
                {studioScope.builderFlows && <li>✓ Add flows and scenarios (optional)</li>}
                {studioScope.builderRoles && <li>✓ Add roles and views (optional)</li>}
                {studioScope.builderGoals && (
                  <li>✓ Formalize with goals & requirements (optional)</li>
                )}
              </ul>
              <button className="btn-secondary" onClick={handleGuidedMode}>
                Start Guided Mode
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

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
        onClose={() => useUIStore.getState().setActiveEditor(null)}
        extraActions={
          // Simplified: No preview toggle anymore as it's controlled by Layout
          null
        }
      />

      <div className="wizard-content">
        <div className="wizard-main">
          {activeStepId === "context" && (
            <SystemContextStep onNext={nextStep} onBack={prevStep} readOnly={!isEditMode()} />
          )}
          {activeStepId === "containers" && (
            <ContainersStep onNext={nextStep} onBack={prevStep} readOnly={!isEditMode()} />
          )}
          {activeStepId === "components" && (
            <ComponentsStep onBack={prevStep} onFinish={nextStep} readOnly={!isEditMode()} />
          )}
          {studioScope.builderFlows && activeStepId === "flows" && (
            <FlowsStep onBack={prevStep} onNext={nextStep} readOnly={!isEditMode()} />
          )}
          {studioScope.builderRoles && activeStepId === "roles-views" && (
            <RolesViewsStep onBack={prevStep} onNext={nextStep} readOnly={!isEditMode()} />
          )}
          {studioScope.builderGoals && activeStepId === "goals" && (
            <GoalsStep onNext={nextStep} onBack={prevStep} readOnly={!isEditMode()} />
          )}
        </div>
      </div>

      {studioScope.builderShare && (
        <SharePanel isOpen={showShare} onClose={() => setShowShare(false)} />
      )}
    </div>
  );
}

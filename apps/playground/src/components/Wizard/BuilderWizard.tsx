import { useState, useMemo } from "react";
import { WizardStepper } from "./WizardStepper";
import type { WizardStep } from "./WizardStepper";
import { GoalsStep } from "./GoalsStep";
import { SystemContextStep } from "./SystemContextStep";
import { ContainersStep } from "./ContainersStep";
import { ComponentsStep } from "./ComponentsStep";
import { FlowsStep } from "./FlowsStep";
import { DslPreview } from "./DslPreview";
import { useArchitectureStore } from "../../stores/architectureStore";
import { convertJsonToDsl } from "../../utils/jsonToDsl";
import "./BuilderWizard.css";

export function BuilderWizard() {
  const [currentStep, setCurrentStep] = useState(0);
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

  const goToStep = (stepIndex: number) => {
    if (!steps[stepIndex].isLocked) {
      setCurrentStep(stepIndex);
    }
  };

  const nextStep = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const prevStep = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  return (
    <div className="builder-wizard">
      <WizardStepper steps={steps} currentStep={currentStep} onStepClick={goToStep} />

      <div className="wizard-content">
        <div className="wizard-main">
          {currentStep === 0 && <GoalsStep onNext={nextStep} />}
          {currentStep === 1 && <SystemContextStep onNext={nextStep} onBack={prevStep} />}
          {currentStep === 2 && <ContainersStep onNext={nextStep} onBack={prevStep} />}
          {currentStep === 3 && <ComponentsStep onBack={prevStep} onFinish={nextStep} />}
          {currentStep === 4 && <FlowsStep onBack={prevStep} />}
        </div>

        {/* DSL Preview Sidebar */}
        <div className="wizard-sidebar">
          <DslPreview dsl={currentDsl} />
        </div>
      </div>
    </div>
  );
}

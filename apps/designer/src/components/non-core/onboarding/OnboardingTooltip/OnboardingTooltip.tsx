import { useState, useEffect, useCallback } from "react";
import { X, ArrowRight, ArrowLeft, CheckCircle, Box, Database, ArrowRightLeft } from "lucide-react";
import { Button } from "@sruja/ui";
import { useUIStore } from "../../../../stores";
import "./OnboardingTooltip.css";

const ONBOARDING_COMPLETED_KEY = "sruja-onboarding-completed";

interface TourStep {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  action?: string;
}

const TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    title: "Welcome to Sruja! 🎉",
    description: "Let's build your first architecture diagram in under 2 minutes.",
    icon: <CheckCircle size={24} />,
  },
  {
    id: "add-system",
    title: "Step 1: Add a System",
    description:
      "Click '+' or type in the Code tab to add your first system. Try: web = system \"My App\"",
    icon: <Box size={24} />,
    action: "Add a system element",
  },
  {
    id: "add-database",
    title: "Step 2: Add a Database",
    description: 'Now add a database: db = database "My Database"',
    icon: <Database size={24} />,
    action: "Add a database element",
  },
  {
    id: "connect",
    title: "Step 3: Connect Them",
    description: 'Draw a connection: web -> db "reads from"',
    icon: <ArrowRightLeft size={24} />,
    action: "Connect elements",
  },
  {
    id: "done",
    title: "You're all set! 🚀",
    description:
      "Explore the Diagram tab to see your architecture. Switch to Code to edit anytime!",
    icon: <CheckCircle size={24} />,
  },
];

export function OnboardingTooltip() {
  const [step, setStep] = useState(0);
  const [visible, setVisible] = useState(false);
  const beginnerMode = useUIStore((s) => s.beginnerMode);

  useEffect(() => {
    // Only show tour for beginner mode users who haven't completed it
    const completed = localStorage.getItem(ONBOARDING_COMPLETED_KEY);
    if (!completed && beginnerMode) {
      const timer = setTimeout(() => {
        setVisible(true);
      }, 1500);
      return () => clearTimeout(timer);
    }
  }, [beginnerMode]);

  const handleNext = useCallback(() => {
    if (step < TOUR_STEPS.length - 1) {
      setStep(step + 1);
    } else {
      handleComplete();
    }
  }, [step]);

  const handleBack = useCallback(() => {
    if (step > 0) {
      setStep(step - 1);
    }
  }, [step]);

  const handleComplete = useCallback(() => {
    setVisible(false);
    localStorage.setItem(ONBOARDING_COMPLETED_KEY, "true");
  }, []);

  const handleSkip = useCallback(() => {
    setVisible(false);
    localStorage.setItem(ONBOARDING_COMPLETED_KEY, "skipped");
  }, []);

  if (!visible) return null;

  const currentStep = TOUR_STEPS[step];
  const isFirstStep = step === 0;
  const isLastStep = step === TOUR_STEPS.length - 1;

  return (
    <div className="onboarding-tour">
      <div className="onboarding-tour-content">
        <button className="onboarding-tour-close" onClick={handleSkip} aria-label="Skip tour">
          <X size={18} />
        </button>

        <div className="onboarding-tour-icon">{currentStep.icon}</div>

        <div className="onboarding-tour-body">
          <h3 className="onboarding-tour-title">{currentStep.title}</h3>
          <p className="onboarding-tour-description">{currentStep.description}</p>
        </div>

        <div className="onboarding-tour-progress">
          {TOUR_STEPS.map((_, idx) => (
            <span
              key={idx}
              className={`progress-dot ${idx === step ? "active" : ""} ${idx < step ? "completed" : ""}`}
            />
          ))}
        </div>

        <div className="onboarding-tour-actions">
          {!isFirstStep && (
            <Button variant="ghost" size="sm" onClick={handleBack}>
              <ArrowLeft size={16} />
              Back
            </Button>
          )}
          <Button variant="primary" size="sm" onClick={handleNext}>
            {isLastStep ? "Get Started" : "Next"}
            {!isLastStep && <ArrowRight size={16} />}
          </Button>
        </div>
      </div>
    </div>
  );
}

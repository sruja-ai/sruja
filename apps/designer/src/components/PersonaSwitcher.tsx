// apps/designer/src/components/PersonaSwitcher.tsx
import { useEffect, useState } from "react";
import { Users, Code, Server, Shield, TrendingUp, Activity, type LucideIcon } from "lucide-react";
import { Button } from "@sruja/ui";
import "./PersonaSwitcher.css";

export type Persona = "product" | "architect" | "devops" | "security" | "cto" | "sre";

interface PersonaConfig {
  id: Persona;
  label: string;
  icon: LucideIcon;
  description: string;
}

const PERSONAS: PersonaConfig[] = [
  {
    id: "product",
    label: "Product",
    icon: Users,
    description: "Feature library, user stories, requirements coverage",
  },
  {
    id: "architect",
    label: "Architect",
    icon: Code,
    description: "ADRs, policies, governance, anti-patterns",
  },
  {
    id: "devops",
    label: "DevOps",
    icon: Server,
    description: "Infrastructure, capacity, cost, deployments",
  },
  {
    id: "security",
    label: "Security",
    icon: Shield,
    description: "Trust boundaries, compliance, data flows",
  },
  {
    id: "cto",
    label: "CTO",
    icon: TrendingUp,
    description: "Health scores, risks, technical debt",
  },
  {
    id: "sre",
    label: "SRE",
    icon: Activity,
    description: "SLOs, error budgets, reliability",
  },
];

const STORAGE_KEY = "sruja-persona-selection";

interface PersonaSwitcherProps {
  selectedPersona: Persona;
  onPersonaChange: (persona: Persona) => void;
  className?: string;
}

export function PersonaSwitcher({
  selectedPersona,
  onPersonaChange,
  className = "",
}: PersonaSwitcherProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Load persona from localStorage on mount
  useEffect(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && PERSONAS.some((p) => p.id === saved)) {
      onPersonaChange(saved as Persona);
    }
  }, [onPersonaChange]);

  // Save persona to localStorage when changed
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, selectedPersona);
  }, [selectedPersona]);

  const currentPersona = PERSONAS.find((p) => p.id === selectedPersona) || PERSONAS[1]; // Default to architect

  const handlePersonaClick = (persona: Persona) => {
    onPersonaChange(persona);
    setIsExpanded(false);
  };

  return (
    <div className={`persona-switcher ${className}`}>
      <div className="persona-switcher-main">
        <Button
          variant="ghost"
          size="sm"
          className="persona-switcher-button"
          onClick={() => setIsExpanded(!isExpanded)}
          aria-label={`Current persona: ${currentPersona.label}`}
          aria-expanded={isExpanded}
          title={`Switch persona view (${currentPersona.description})`}
        >
          <currentPersona.icon size={16} />
          <span className="persona-label">{currentPersona.label}</span>
        </Button>
      </div>

      {isExpanded && (
        <>
          <div
            className="persona-switcher-overlay"
            onClick={() => setIsExpanded(false)}
            aria-hidden="true"
          />
          <div className="persona-switcher-menu">
            <div className="persona-switcher-menu-header">
              <span className="persona-switcher-menu-title">Switch Persona View</span>
              <span className="persona-switcher-menu-subtitle">
                Each persona sees the same architecture through a different lens
              </span>
            </div>
            <div className="persona-switcher-options">
              {PERSONAS.map((persona) => {
                const Icon = persona.icon;
                const isSelected = persona.id === selectedPersona;
                return (
                  <button
                    key={persona.id}
                    className={`persona-option ${isSelected ? "selected" : ""}`}
                    onClick={() => handlePersonaClick(persona.id)}
                    aria-label={`Switch to ${persona.label} view`}
                    aria-pressed={isSelected}
                  >
                    <Icon size={18} className="persona-option-icon" />
                    <div className="persona-option-content">
                      <span className="persona-option-label">{persona.label}</span>
                      <span className="persona-option-description">{persona.description}</span>
                    </div>
                    {isSelected && (
                      <span className="persona-option-check" aria-hidden="true">
                        ✓
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

import { ArrowRight, Plus } from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import "./WizardSteps.css";

interface FlowsStepProps {
    onBack: () => void;
    readOnly?: boolean;
}

export function FlowsStep({ onBack, readOnly = false }: FlowsStepProps) {
    const data = useArchitectureStore((s) => s.likec4Model);
    // scenarios are in sruja.scenarios
    const scenarios = data?.sruja?.scenarios ?? [];

    return (
        <div className="wizard-step-content">
            <div className="step-header">
                <div className="step-icon">
                    <ArrowRight size={24} />
                </div>
                <div className="step-header-content">
                    <h2>Define Flows</h2>
                    <p>Describe how your system behaves with scenarios.</p>
                </div>
            </div>
            <div className="step-section">
                <div className="items-list">
                    {scenarios.map((s, i) => (
                        <div key={i} className="item-card">
                            <span className="item-text">{s.title || "Untitled Scenario"}</span>
                        </div>
                    ))}
                    {scenarios.length === 0 && <div className="empty-state">No scenarios defined yet.</div>}
                </div>

                {!readOnly && (
                    <div className="add-form">
                        <Button variant="secondary" disabled>
                            <Plus size={16} />
                            Add Scenario (Coming Soon)
                        </Button>
                    </div>
                )}
            </div>
            <div className="step-navigation">
                <Button variant="ghost" onClick={onBack}>
                    ← Back
                </Button>
            </div>
        </div>
    );
}

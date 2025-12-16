import { useMemo, useState } from "react";
import { Target, Info, Shield, Play } from "lucide-react"; // Needed for action buttons icons
import { useArchitectureStore, useUIStore } from "../../stores";
import { useFeatureFlagsStore } from "../../stores/featureFlagsStore";
import {
  EditRequirementForm,
  EditADRForm,
  EditScenarioForm,
  EditFlowForm,
  EditOverviewForm,
  EditPolicyForm,
  EditMetadataForm,
  EditConstraintForm,
  EditConventionForm,
} from "../shared";
import type {
  RequirementJSON,
  ADRJSON,
  ScenarioJSON,
  FlowJSON,
  PolicyJSON,
  MetadataEntry,
  ConstraintJSON,
  ConventionJSON,
} from "../../types";
import "./OverviewPanel.css";

// Import extracted components
import { OverviewHero } from "../../components/Overview/OverviewHero";
import { StatsRow } from "../../components/Overview/StatsRow";
import { PoliciesSection } from "../../components/Overview/PoliciesSection";
import { ConstraintsSection } from "../../components/Overview/ConstraintsSection";
import { ConventionsSection } from "../../components/Overview/ConventionsSection";
import { GoalsSection } from "../../components/Overview/GoalsSection";
import { MetadataSection } from "../../components/Overview/MetadataSection";

export function OverviewPanel() {
  const data = useArchitectureStore((s) => s.data);
  const isFeatureEnabled = useFeatureFlagsStore((s) => s.isEnabled);
  const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);

  // Dialog states
  const [editRequirement, setEditRequirement] = useState<RequirementJSON | undefined>(undefined);
  const [editADR, setEditADR] = useState<ADRJSON | undefined>(undefined);
  const [editScenario, setEditScenario] = useState<ScenarioJSON | undefined>(undefined);
  const [editFlow, setEditFlow] = useState<FlowJSON | undefined>(undefined);
  const [showRequirementForm, setShowRequirementForm] = useState(false);
  const [showADRForm, setShowADRForm] = useState(false);
  const [showScenarioForm, setShowScenarioForm] = useState(false);
  const [showFlowForm, setShowFlowForm] = useState(false);
  const [showOverviewForm, setShowOverviewForm] = useState(false);
  const [showPolicyForm, setShowPolicyForm] = useState(false);
  const [editPolicy, setEditPolicy] = useState<PolicyJSON | undefined>(undefined);
  const [showMetadataForm, setShowMetadataForm] = useState(false);
  const [editMetadata, setEditMetadata] = useState<
    { metadata: MetadataEntry; index: number } | undefined
  >(undefined);
  const [showConstraintForm, setShowConstraintForm] = useState(false);
  const [editConstraint, setEditConstraint] = useState<
    { constraint: ConstraintJSON; index: number } | undefined
  >(undefined);
  const [showConventionForm, setShowConventionForm] = useState(false);
  const [editConvention, setEditConvention] = useState<
    { convention: ConventionJSON; index: number } | undefined
  >(undefined);

  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
  const setPendingAction = useUIStore((s) => s.setPendingAction);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  const overview = data?.architecture?.overview;
  const archMetadata = data?.architecture?.archMetadata;
  const architectureName = data?.architecture?.name || data?.metadata?.name;
  const description = data?.architecture?.description;

  // Calculate counts for quick stats
  const stats = useMemo(() => {
    if (!data?.architecture) return null;
    const arch = data.architecture;
    return {
      systems: arch.systems?.length ?? 0,
      persons: arch.persons?.length ?? 0,
      requirements: arch.requirements?.length ?? 0,
      adrs: arch.adrs?.length ?? 0,
      policies: arch.policies?.length ?? 0,
      flows: arch.flows?.length ?? 0,
    };
  }, [data]);

  if (!data) {
    return null;
  }

  // Delete handlers
  const handleDeleteMetadata = async (index: number, key: string) => {
    if (confirm(`Delete metadata "${key}"?`)) {
      await updateArchitecture((arch) => {
        if (!arch.architecture) return arch;
        const metadataList = (arch.architecture.archMetadata || []).filter(
          (_, idx) => idx !== index
        );
        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            archMetadata: metadataList,
          },
        };
      });
    }
  };

  const handleDeleteConstraint = async (index: number, key: string) => {
    if (confirm(`Delete constraint "${key}"?`)) {
      await updateArchitecture((arch) => {
        if (!arch.architecture) return arch;
        const constraints = (arch.architecture.constraints || []).filter((_, idx) => idx !== index);
        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            constraints,
          },
        };
      });
    }
  };

  const handleDeleteConvention = async (index: number, key: string) => {
    if (confirm(`Delete convention "${key}"?`)) {
      await updateArchitecture((arch) => {
        if (!arch.architecture) return arch;
        const conventions = (arch.architecture.conventions || []).filter((_, idx) => idx !== index);
        return {
          ...arch,
          architecture: {
            ...arch.architecture,
            conventions,
          },
        };
      });
    }
  };

  return (
    <div className="overview-panel">
      <OverviewHero
        architectureName={architectureName}
        description={description}
        overview={overview}
        archMetadata={archMetadata}
        onEditOverview={() => setShowOverviewForm(true)}
      />

      <StatsRow
        stats={stats}
        onAddRequirement={() => {
          setPendingAction("create-requirement");
          setActiveTab("details");
        }}
        onAddADR={() => {
          setPendingAction("create-adr");
          setActiveTab("details");
        }}
      />

      {/* Editing Actions */}
      {isEditMode() && (
        <div className="overview-actions">
          <div className="action-group">
            <h4 className="action-group-title">Add New</h4>
            <div className="action-buttons">
              <button
                className="action-btn"
                onClick={() => {
                  setEditRequirement(undefined);
                  setShowRequirementForm(true);
                }}
              >
                <Target size={16} />
                <span>Requirement</span>
              </button>
              <button
                className="action-btn"
                onClick={() => {
                  setEditADR(undefined);
                  setShowADRForm(true);
                }}
              >
                <Info size={16} />
                <span>ADR</span>
              </button>
              {isFeatureEnabled("policies") && (
                <button
                  className="action-btn"
                  onClick={() => {
                    setEditPolicy(undefined);
                    setShowPolicyForm(true);
                  }}
                >
                  <Shield size={16} />
                  <span>Policy</span>
                </button>
              )}
              <button
                className="action-btn"
                onClick={() => {
                  setEditScenario(undefined);
                  setShowScenarioForm(true);
                }}
              >
                <Play size={16} />
                <span>Scenario</span>
              </button>
              <button
                className="action-btn"
                onClick={() => {
                  setEditFlow(undefined);
                  setShowFlowForm(true);
                }}
              >
                <Play size={16} />
                <span>Flow</span>
              </button>
            </div>
          </div>
        </div>
      )}

      <MetadataSection
        metadata={archMetadata}
        onAddMetadata={() => {
          setEditMetadata(undefined);
          setShowMetadataForm(true);
        }}
        onEditMetadata={(meta, index) => {
          setEditMetadata({ metadata: meta, index });
          setShowMetadataForm(true);
        }}
        onDeleteMetadata={handleDeleteMetadata}
      />

      <GoalsSection overview={overview} />

      <PoliciesSection
        policies={data?.architecture?.policies}
        policyCount={stats?.policies || 0}
        onAddPolicy={() => {
          setEditPolicy(undefined);
          setShowPolicyForm(true);
        }}
        onEditPolicy={(policy) => {
          setEditPolicy(policy);
          setShowPolicyForm(true);
        }}
      />

      <ConstraintsSection
        constraints={data?.architecture?.constraints}
        onAddConstraint={() => {
          setEditConstraint(undefined);
          setShowConstraintForm(true);
        }}
        onEditConstraint={(constraint, index) => {
          setEditConstraint({ constraint, index });
          setShowConstraintForm(true);
        }}
        onDeleteConstraint={handleDeleteConstraint}
      />

      <ConventionsSection
        conventions={data?.architecture?.conventions}
        onAddConvention={() => {
          setEditConvention(undefined);
          setShowConventionForm(true);
        }}
        onEditConvention={(convention, index) => {
          setEditConvention({ convention, index });
          setShowConventionForm(true);
        }}
        onDeleteConvention={handleDeleteConvention}
      />

      {/* Edit Forms - Kept at top level for state management simplicity */}
      <EditRequirementForm
        isOpen={showRequirementForm}
        onClose={() => {
          setShowRequirementForm(false);
          setEditRequirement(undefined);
        }}
        requirement={editRequirement}
      />
      <EditADRForm
        isOpen={showADRForm}
        onClose={() => {
          setShowADRForm(false);
          setEditADR(undefined);
        }}
        adr={editADR}
      />
      <EditPolicyForm
        isOpen={showPolicyForm}
        onClose={() => {
          setShowPolicyForm(false);
          setEditPolicy(undefined);
        }}
        policy={editPolicy}
      />
      <EditScenarioForm
        isOpen={showScenarioForm}
        onClose={() => {
          setShowScenarioForm(false);
          setEditScenario(undefined);
        }}
        scenario={editScenario}
      />
      <EditFlowForm
        isOpen={showFlowForm}
        onClose={() => {
          setShowFlowForm(false);
          setEditFlow(undefined);
        }}
        flow={editFlow}
      />
      <EditOverviewForm isOpen={showOverviewForm} onClose={() => setShowOverviewForm(false)} />
      <EditMetadataForm
        isOpen={showMetadataForm}
        onClose={() => {
          setShowMetadataForm(false);
          setEditMetadata(undefined);
        }}
        metadata={editMetadata?.metadata}
        metadataIndex={editMetadata?.index}
      />
      <EditConstraintForm
        isOpen={showConstraintForm}
        onClose={() => {
          setShowConstraintForm(false);
          setEditConstraint(undefined);
        }}
        constraint={editConstraint?.constraint}
        constraintIndex={editConstraint?.index}
      />
      <EditConventionForm
        isOpen={showConventionForm}
        onClose={() => {
          setShowConventionForm(false);
          setEditConvention(undefined);
        }}
        convention={editConvention?.convention}
        conventionIndex={editConvention?.index}
      />
    </div>
  );
}

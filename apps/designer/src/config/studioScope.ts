// Centralized scope gates for hiding non-core UI.
// Flip VITE_ENABLE_NON_CORE_UI=true to temporarily re-enable extras.

const NON_CORE_UI_ENABLED = import.meta.env.VITE_ENABLE_NON_CORE_UI === "true";

export const studioScope = {
  // Core: always on
  builder: true,
  dslEditor: true,
  export: true,
  docs: true,
  review: true,
  diffs: true,
  drillDowns: true,

  // Non-core: gated behind a single flag for easy removal later
  examples: NON_CORE_UI_ENABLED,
  share: NON_CORE_UI_ENABLED,
  commandPalette: NON_CORE_UI_ENABLED,
  shortcutsModal: NON_CORE_UI_ENABLED,
  onboarding: NON_CORE_UI_ENABLED,
  qualityScoreCard: NON_CORE_UI_ENABLED,
  governanceScore: NON_CORE_UI_ENABLED,
  reviewGovernanceWidget: NON_CORE_UI_ENABLED,
  reviewPolicyEnforcement: NON_CORE_UI_ENABLED,
  inspectorGovernance: NON_CORE_UI_ENABLED,
  verificationsAction: NON_CORE_UI_ENABLED,
  builderFlows: true,
  builderRoles: true,
  builderGoals: true,
  builderShare: NON_CORE_UI_ENABLED,
} as const;

export type StudioScopeKey = keyof typeof studioScope;

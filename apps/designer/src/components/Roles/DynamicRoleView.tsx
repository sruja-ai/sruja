import { useState, useEffect, useMemo } from "react";
import {
  Paper,
  Text,
  Group,
  Stack,
  Grid,
  Badge,
  ThemeIcon,
  Button,
  ScrollArea,
} from "@mantine/core";
import { Button as UIButton } from "@sruja/ui";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { useArchitectureStore, useUIStore, useViewStore } from "../../stores";
import { logger } from "@sruja/shared";
import type { ParsedView } from "@sruja/shared";
import type { Role } from "../RoleSwitcher";
import { ArrowRight, Layout, AlertTriangle, Zap, Users } from "lucide-react";
import "./RoleView.css";

// Role-specific components
import { AntiPatternDetector, ADRManager } from "../Architect";
import { GovernanceScore, PolicyEnforcement } from "../non-core/governance";
import { SLOManager } from "../SRE";
import { HealthScore, RiskAssessment, TechnicalDebt } from "../CTO";
import { CostEstimation } from "../DevOps";
import { DataFlowScanner, TrustBoundaries } from "../Security";
import { RequirementTraceabilityView, RequirementsCoverage } from "../Product";

interface DynamicRoleViewProps {
  role?: Role; // Optional now - will be selected within component
  title?: string;
  description?: string;
}

export function DynamicRoleView({ role: initialRole }: DynamicRoleViewProps = {}) {
  const archModel = getArchitectureModel();
  const model = useArchitectureStore((s) => s.model);
  const setChaosEnabled = useArchitectureStore((s) => s.setChaosEnabled);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const setActiveView = useViewStore((s) => s.setActiveView);

  // Helper function for default role descriptions
  function getDefaultRoleDescription(role: Role): string {
    switch (role) {
      case "product":
        return "Feature library, user stories, requirements coverage";
      case "devops":
        return "Infrastructure, capacity, cost, deployments";
      case "security":
        return "Trust boundaries, compliance, data flows";
      case "cto":
        return "Health scores, risks, technical debt";
      case "sre":
        return "SLOs, error budgets, reliability";
      case "architect":
      default:
        return "Design and govern system structure";
    }
  }

  // Discover available roles
  const availableRoles = archModel.discoverRoles();
  const nodes = archModel.getNodes();

  // Initialize with first available role from DSL, or fallback to "architect" if none
  const [selectedRole, setSelectedRole] = useState<Role>(
    initialRole || (availableRoles.length > 0 ? availableRoles[0] : "architect")
  );
  const [views, setViews] = useState<ParsedView[]>([]);

  // Build role tabs data - fully dynamic from DSL
  const roleTabs = useMemo(() => {
    // Only show roles that are defined in the DSL
    return availableRoles.map((roleId) => {
      const element = nodes.get(roleId);
      return {
        id: roleId,
        label: element?.title || roleId.charAt(0).toUpperCase() + roleId.slice(1),
        description: element?.description || getDefaultRoleDescription(roleId as Role),
      };
    });
  }, [availableRoles, nodes]);

  // Set initial role if not provided - use first available role from DSL
  // Only run once on mount or when roleTabs changes (not when selectedRole changes)
  useEffect(() => {
    if (!initialRole && roleTabs.length > 0) {
      // Only set initial role if current selectedRole is not in the available roles
      const isCurrentRoleValid = roleTabs.some((tab) => tab.id === selectedRole);
      if (!isCurrentRoleValid) {
        const firstRole = roleTabs[0].id as Role;
        setSelectedRole(firstRole);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialRole, roleTabs]);

  // Get role title and description
  const roleElement = nodes.get(selectedRole);
  const roleTitle =
    roleElement?.title || selectedRole.charAt(0).toUpperCase() + selectedRole.slice(1);
  const roleDescription = roleElement?.description || getDefaultRoleDescription(selectedRole);

  // Refresh views when model or selected role changes
  useEffect(() => {
    const updateViews = () => {
      const roleViews = archModel.getViewsByRole(selectedRole);
      // Debug logging
      logger.info(`[RoleView] Role: ${selectedRole}, Found ${roleViews.length} views`);
      if (roleViews.length === 0) {
        const allViews = archModel.getViews();
        logger.info(`[RoleView] All views: ${Object.keys(allViews).join(", ")}`);
        logger.info(
          `[RoleView] Sample view tags: ${Object.values(allViews)
            .slice(0, 3)
            .map((v) => ({ id: v.id, tags: v.tags }))
            .join(", ")}}`
        );
      }
      setViews(roleViews);
    };

    updateViews();
    // Subscribe to changes
    const unsubscribe = archModel.subscribe(selectedRole, updateViews);
    return unsubscribe;
  }, [selectedRole, model]);

  const handleLaunchView = (viewId: string, tags: string[] | null | undefined = []) => {
    // Navigate to diagram tab
    setActiveTab("diagram");
    // Set the active view ID in the store to load the specific view
    setActiveView(viewId);

    // Auto-enable scenarios based on tags
    const safeTags = tags || [];
    if (safeTags.includes("#scenario:failure")) {
      setChaosEnabled(true);
    } else {
      setChaosEnabled(false);
    }
  };

  // Show empty state if no roles defined in DSL
  if (roleTabs.length === 0) {
    return (
      <div className="role-view">
        <div className="role-view-header">
          <h2 className="role-view-title">Roles</h2>
          <p className="role-view-description">
            No roles defined yet. Create roles in the Builder tab to get started.
          </p>
        </div>
        <div className="role-view-content">
          <Paper withBorder p="xl" radius="md" bg="var(--mantine-color-gray-0)">
            <Stack align="center" gap="xs">
              <ThemeIcon size={48} radius="xl" color="gray" variant="light">
                <Users size={24} />
              </ThemeIcon>
              <Text fw={600}>No Roles Defined</Text>
              <Text size="sm" c="dimmed" ta="center" maw={400}>
                Define roles in your DSL using the <code>role</code> keyword, or create them in the
                Builder tab.
              </Text>
              <Text size="xs" c="dimmed" ta="center" maw={400} mt="md">
                Example:{" "}
                <code>
                  devops = role "DevOps Team" {"{"} description "Infrastructure team" {"}"}
                </code>
              </Text>
            </Stack>
          </Paper>
        </div>
      </div>
    );
  }

  return (
    <div className="role-view">
      {/* Role Sub-Tabs - Dynamic from DSL */}
      <div className="role-tabs-container">
        <div className="role-tabs" role="tablist" aria-label="Role tabs">
          {roleTabs.map((roleTab) => {
            const isActive = roleTab.id === selectedRole;
            return (
              <UIButton
                key={roleTab.id}
                variant={isActive ? "secondary" : "ghost"}
                size="sm"
                className={`role-tab ${isActive ? "active" : ""}`}
                onClick={() => setSelectedRole(roleTab.id as Role)}
                role="tab"
                aria-selected={isActive}
                id={`role-tab-${roleTab.id}`}
                aria-controls={`role-tabpanel-${roleTab.id}`}
                title={roleTab.description}
              >
                <div className="role-tab-content">
                  <Users size={16} />
                  <span>{roleTab.label}</span>
                </div>
              </UIButton>
            );
          })}
        </div>
      </div>

      {/* Role Content */}
      <div className="role-view-content-wrapper">
        <div className="role-view-header">
          <h2 className="role-view-title">{roleTitle} View</h2>
          <p className="role-view-description">{roleDescription}</p>
        </div>

        <ScrollArea className="role-view-content" style={{ height: "calc(100% - 140px)" }}>
          <Stack gap="xl" p="md">
            {/* Scenarios Section */}
            {views.length > 0 ? (
              <div>
                <Text fw={600} size="lg" mb="sm">
                  Recommended Views & Scenarios
                </Text>
                <Grid>
                  {views.map((view) => {
                    const isFailureScenario = view.tags?.includes("#scenario:failure");
                    const isCapacityScenario = view.tags?.includes("#scenario:capacity");

                    return (
                      <Grid.Col span={4} key={view.id}>
                        <Paper
                          withBorder
                          p="md"
                          radius="md"
                          className="view-card"
                          style={{ cursor: "pointer", height: "100%" }}
                          onClick={() => handleLaunchView(view.id, view.tags)}
                        >
                          <Stack justify="space-between" h="100%">
                            <div>
                              <Group justify="space-between" mb="xs">
                                <ThemeIcon
                                  color={
                                    isFailureScenario ? "red" : isCapacityScenario ? "blue" : "gray"
                                  }
                                  variant="light"
                                  size="lg"
                                >
                                  {isFailureScenario ? (
                                    <AlertTriangle size={18} />
                                  ) : isCapacityScenario ? (
                                    <Zap size={18} />
                                  ) : (
                                    <Layout size={18} />
                                  )}
                                </ThemeIcon>
                                {isFailureScenario && <Badge color="error">Chaos Mode</Badge>}
                              </Group>
                              <Text fw={600} lineClamp={1}>
                                {view.title || view.id}
                              </Text>
                              <Text size="sm" c="dimmed" lineClamp={2} mt={4}>
                                {view.description || "Interactive architecture view."}
                              </Text>
                            </div>
                            <Button
                              rightSection={<ArrowRight size={14} />}
                              variant="light"
                              mt="md"
                              color={isFailureScenario ? "red" : "blue"}
                            >
                              Launch Scenario
                            </Button>
                          </Stack>
                        </Paper>
                      </Grid.Col>
                    );
                  })}
                </Grid>
              </div>
            ) : (
              <Paper withBorder p="xl" radius="md" bg="var(--mantine-color-gray-0)">
                <Stack align="center" gap="xs">
                  <ThemeIcon size={48} radius="xl" color="gray" variant="light">
                    <Layout size={24} />
                  </ThemeIcon>
                  <Text fw={600}>No Role-specific Views Found</Text>
                  <Text size="sm" c="dimmed" ta="center" maw={400}>
                    Add tags like <code>{selectedRole.toLowerCase()}</code> to your views
                    (referencing a role element) to make them appear here automatically.
                  </Text>
                </Stack>
              </Paper>
            )}

            {/* Role-Specific Tools Section */}
            <div>
              <Text fw={600} size="lg" mb="sm">
                {selectedRole.charAt(0).toUpperCase() + selectedRole.slice(1)} Insights
              </Text>
              <Stack gap="md">
                {selectedRole === "architect" && (
                  <>
                    <GovernanceScore />
                    <PolicyEnforcement />
                    <AntiPatternDetector />
                    <ADRManager />
                  </>
                )}
                {selectedRole === "sre" && (
                  <>
                    <SLOManager />
                  </>
                )}
                {selectedRole === "cto" && (
                  <>
                    <HealthScore />
                    <RiskAssessment />
                    <TechnicalDebt />
                  </>
                )}
                {selectedRole === "devops" && (
                  <>
                    <CostEstimation />
                  </>
                )}
                {selectedRole === "security" && (
                  <>
                    <DataFlowScanner />
                    <TrustBoundaries />
                  </>
                )}
                {selectedRole === "product" && (
                  <>
                    <RequirementTraceabilityView />
                    <RequirementsCoverage />
                  </>
                )}
              </Stack>
            </div>
          </Stack>
        </ScrollArea>
      </div>
    </div>
  );
}

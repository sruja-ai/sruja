import { useMemo } from "react";
import { ShieldCheck, AlertTriangle } from "lucide-react";
import { Paper, Stack, Group, Text } from "@mantine/core";
import { useArchitectureStore } from "../../stores";
import { GovernanceWidget } from "../non-core/governance/GovernanceWidget";
import { PolicyEnforcement } from "../non-core/governance/PolicyEnforcement";
import { AntiPatternDetector } from "../Architect";
import { ProjectInspector } from "../Panels/Inspector/ProjectInspector";
import "./BestPracticesView.css";
import { studioScope } from "../../config/studioScope";

/**
 * BestPracticesView
 *
 * A focused, “staff engineer” review surface:
 * - Architecture health score + rule deductions (WASM)
 * - Anti-pattern detection
 * - Policy enforcement (if policies exist)
 *
 * Intentionally read-only: it surfaces issues and deep-links into the DSL editor.
 */
export function BestPracticesView() {
  const model = useArchitectureStore((s) => s.model);

  const hasModel = !!model;
  const hasPolicies = useMemo(() => {
    const policies = model?.sruja?.policies;
    return Array.isArray(policies) && policies.length > 0;
  }, [model]);

  if (!hasModel) {
    return (
      <div className="best-practices-view">
        <Paper withBorder p="xl" radius="md">
          <Group gap="sm" align="center">
            <ShieldCheck size={22} />
            <div>
              <Text fw={700}>Best Practices Review</Text>
              <Text size="sm" c="dimmed">
                Load or build an architecture to see quality checks and recommendations.
              </Text>
            </div>
          </Group>
        </Paper>
      </div>
    );
  }

  return (
    <div className="best-practices-view">
      <Stack gap="md">
        <Paper withBorder p="md" radius="md">
          <Group gap="sm" align="center">
            <ShieldCheck size={20} />
            <div>
              <Text fw={700}>Best Practices Review</Text>
              <Text size="sm" c="dimmed">
                Quality signals, rule violations, and concrete fixes.
              </Text>
            </div>
          </Group>
        </Paper>

        {/* Reuse the existing project overview block (score + stats) */}
        <ProjectInspector />

        {/* Detailed rule deductions with deep-links into the DSL editor */}
        {studioScope.reviewGovernanceWidget && <GovernanceWidget />}

        <AntiPatternDetector />

        {studioScope.reviewPolicyEnforcement &&
          (hasPolicies ? (
            <PolicyEnforcement />
          ) : (
            <Paper withBorder p="md" radius="md">
              <Group gap="sm" align="flex-start">
                <AlertTriangle size={18} />
                <div>
                  <Text fw={600}>Policy enforcement is currently inactive</Text>
                  <Text size="sm" c="dimmed">
                    No policies found in this project. Add policies to enable automated enforcement
                    checks.
                  </Text>
                </div>
              </Group>
            </Paper>
          ))}
      </Stack>
    </div>
  );
}

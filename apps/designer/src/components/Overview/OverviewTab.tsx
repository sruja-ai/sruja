import { useMemo } from "react";
import { Layers, Users, FileText, ArrowRight, Layout, Code, Hammer } from "lucide-react";
import { Paper, Text, Stack, Group, Grid } from "@mantine/core";
import { useArchitectureStore, useUIStore } from "../../stores";
import type { Element } from "@sruja/shared";
import { OverviewHero } from "./OverviewHero";
import { StatsRow } from "./StatsRow";
import { MetadataSection } from "./MetadataSection";
import { GovernanceWidget } from "./GovernanceWidget";
import "./OverviewTab.css";
import "../Panels/styles/overview-hero.css";
import "../Panels/styles/overview-stats.css";
import "../Panels/styles/overview-metadata.css";
import "../Panels/styles/overview-context.css";

export function OverviewTab() {
  const model = useArchitectureStore((s) => s.model);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const sruja = model?.sruja;

  // Calculate stats
  const stats = useMemo(() => {
    if (!model) return null;
    const elements: Element[] = Object.values(model.elements || {});
    const isSystem = (e: Element): boolean => e.kind === "system";
    const isPerson = (e: Element): boolean => e.kind === "person";

    return {
      systems: elements.filter(isSystem).length,
      persons: elements.filter(isPerson).length,
      requirements: sruja?.requirements?.length ?? 0,
      adrs: sruja?.adrs?.length ?? 0,
      policies: sruja?.policies?.length ?? 0,
      flows: sruja?.flows?.length ?? 0,
    };
  }, [model, sruja]);

  // Get architecture metadata
  const architectureName = model?._metadata?.name || "Architecture";
  const archMetadataRaw = model?._metadata
    ? (model._metadata as { archMetadata?: Record<string, string> }).archMetadata
    : undefined;
  const archMetadata = archMetadataRaw
    ? Object.entries(archMetadataRaw).map(([key, value]) => ({ key, value }))
    : undefined;

  if (!model) {
    return (
      <div className="overview-tab-empty">
        <Paper withBorder p="xl" radius="md">
          <Stack align="center" gap="md">
            <Layers size={48} style={{ opacity: 0.3 }} />
            <Text fw={600} size="lg">
              No Architecture Loaded
            </Text>
            <Text size="sm" c="dimmed" ta="center">
              Load an example, import a .sruja file, or start building in the Builder tab.
            </Text>
          </Stack>
        </Paper>
      </div>
    );
  }

  return (
    <div className="overview-tab">
      <div className="overview-tab-content">
        {/* Hero Section with Governance */}
        <section className="overview-section overview-hero-section">
          <Grid gutter={{ base: "md", md: "lg" }}>
            <Grid.Col span={{ base: 12, lg: 8 }}>
              <OverviewHero
                architectureName={architectureName}
                description={undefined}
                overview={undefined}
                archMetadata={archMetadata}
                onEditOverview={() => {}}
              />
            </Grid.Col>
            <Grid.Col span={{ base: 12, lg: 4 }}>
              <div className="governance-widget-wrapper">
                <GovernanceWidget />
              </div>
            </Grid.Col>
          </Grid>
        </section>

        {/* Stats Section */}
        <section className="overview-section overview-stats-section">
          <StatsRow
            stats={stats}
            onAddRequirement={() => {
              setActiveTab("details");
            }}
            onAddADR={() => {
              setActiveTab("details");
            }}
          />
        </section>

        {/* Quick Navigation Section */}
        <section className="overview-section overview-quick-nav">
          <div className="overview-section-header">
            <Text fw={700} size="lg" className="overview-section-title">
              Quick Navigation
            </Text>
            <Text size="sm" c="dimmed" className="overview-section-subtitle">
              Access key features and views
            </Text>
          </div>
          <div className="overview-nav-cards-container">
            {/* Primary Actions */}
            <Paper
              withBorder
              p="sm"
              radius="md"
              className="overview-nav-card overview-nav-card-primary"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab("builder")}
            >
              <Group gap="xs" align="center" wrap="nowrap">
                <div className="nav-card-icon-wrapper nav-card-icon-primary">
                  <Hammer size={18} />
                </div>
                <div className="nav-card-content">
                  <Text fw={600} size="sm" className="nav-card-title">
                    Builder
                  </Text>
                  <Text size="xs" c="dimmed" lineClamp={1} className="nav-card-description">
                    Step-by-step design guide
                  </Text>
                </div>
                <ArrowRight size={16} className="nav-card-arrow" />
              </Group>
            </Paper>
            <Paper
              withBorder
              p="sm"
              radius="md"
              className="overview-nav-card overview-nav-card-primary"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab("diagram")}
            >
              <Group gap="xs" align="center" wrap="nowrap">
                <div className="nav-card-icon-wrapper nav-card-icon-primary">
                  <Layout size={18} />
                </div>
                <div className="nav-card-content">
                  <Text fw={600} size="sm" className="nav-card-title">
                    Diagram
                  </Text>
                  <Text size="xs" c="dimmed" lineClamp={1} className="nav-card-description">
                    Visual architecture diagram
                  </Text>
                </div>
                <ArrowRight size={16} className="nav-card-arrow" />
              </Group>
            </Paper>
            <Paper
              withBorder
              p="sm"
              radius="md"
              className="overview-nav-card overview-nav-card-primary"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab("code")}
            >
              <Group gap="xs" align="center" wrap="nowrap">
                <div className="nav-card-icon-wrapper nav-card-icon-primary">
                  <Code size={18} />
                </div>
                <div className="nav-card-content">
                  <Text fw={600} size="sm" className="nav-card-title">
                    Code
                  </Text>
                  <Text size="xs" c="dimmed" lineClamp={1} className="nav-card-description">
                    View and edit DSL code
                  </Text>
                </div>
                <ArrowRight size={16} className="nav-card-arrow" />
              </Group>
            </Paper>
            {/* Secondary Actions */}
            <Paper
              withBorder
              p="sm"
              radius="md"
              className="overview-nav-card"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab("details")}
            >
              <Group gap="xs" align="center" wrap="nowrap">
                <div className="nav-card-icon-wrapper">
                  <FileText size={18} />
                </div>
                <div className="nav-card-content">
                  <Text fw={600} size="sm" className="nav-card-title">
                    Details
                  </Text>
                  <Text size="xs" c="dimmed" lineClamp={1} className="nav-card-description">
                    Requirements, ADRs, flows
                  </Text>
                </div>
                <ArrowRight size={16} className="nav-card-arrow" />
              </Group>
            </Paper>
            <Paper
              withBorder
              p="sm"
              radius="md"
              className="overview-nav-card"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab("roles")}
            >
              <Group gap="xs" align="center" wrap="nowrap">
                <div className="nav-card-icon-wrapper">
                  <Users size={18} />
                </div>
                <div className="nav-card-content">
                  <Text fw={600} size="sm" className="nav-card-title">
                    Roles
                  </Text>
                  <Text size="xs" c="dimmed" lineClamp={1} className="nav-card-description">
                    Role-based perspectives
                  </Text>
                </div>
                <ArrowRight size={16} className="nav-card-arrow" />
              </Group>
            </Paper>
          </div>
        </section>

        {/* Metadata Section */}
        {archMetadata && archMetadata.length > 0 && (
          <section className="overview-section overview-metadata-section">
            <MetadataSection
              metadata={archMetadata}
              onAddMetadata={() => {}}
              onEditMetadata={() => {}}
              onDeleteMetadata={() => {}}
            />
          </section>
        )}
      </div>
    </div>
  );
}

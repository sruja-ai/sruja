import { useState, useEffect } from "react";
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
import { getArchitectureModel } from "../../models/ArchitectureModel";
import { useArchitectureStore, useUIStore } from "../../stores";
import type { Persona } from "../PersonaSwitcher";
import { ArrowRight, Layout, Play, AlertTriangle, Zap } from "lucide-react";
import "./PersonaView.css";

interface DynamicPersonaViewProps {
  persona: Persona;
  title: string;
  description: string;
}

export function DynamicPersonaView({ persona, title, description }: DynamicPersonaViewProps) {
  const [views, setViews] = useState<any[]>([]);
  const model = useArchitectureStore((s) => s.model);
  const setChaosEnabled = useArchitectureStore((s) => s.setChaosEnabled);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  // Refresh views when model changes
  useEffect(() => {
    const archModel = getArchitectureModel();
    const updateViews = () => {
      const personaViews = archModel.getViewsByPersona(persona);
      setViews(personaViews);
    };

    updateViews();
    // Subscribe to changes
    const unsubscribe = archModel.subscribe(persona, updateViews);
    return unsubscribe;
  }, [persona, model]);

  const handleLaunchView = (_viewId: string, tags: string[] = []) => {
    // Navigate to diagram tab
    setActiveTab("diagram");
    // Ideally we should also set the View ID in the store to load this specific view
    // For now, we'll just activate the tab functionality as a placeholder for navigation logic
    // TODO: Implement setViewId action in store
    // Launching persona view

    // Auto-enable scenarios based on tags
    if (tags.includes("#scenario:failure")) {
      setChaosEnabled(true);
    } else {
      setChaosEnabled(false);
    }
  };

  return (
    <div className="persona-view">
      <div className="persona-view-header">
        <h2 className="persona-view-title">{title}</h2>
        <p className="persona-view-description">{description}</p>
      </div>

      <ScrollArea className="persona-view-content" style={{ height: "calc(100% - 80px)" }}>
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
                              {isFailureScenario && (
                                <Badge color="red" variant="dot">
                                  Chaos Mode
                                </Badge>
                              )}
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
                            fullWidth
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
                <Text fw={600}>No Persona-specific Views Found</Text>
                <Text size="sm" c="dimmed" ta="center" maw={400}>
                  Add tags like <code>#persona:{persona.toLowerCase()}</code> to your views to make
                  them appear here automatically.
                </Text>
              </Stack>
            </Paper>
          )}

          {/* Default Tools Section (Hardcoded Fallback) */}
          <div>
            <Text fw={600} size="lg" mb="sm">
              Standard Tools
            </Text>
            <Grid>
              <Grid.Col span={6}>
                <Paper withBorder p="md" radius="md">
                  <Group>
                    <ThemeIcon size="lg" variant="light" color="blue">
                      <Play size={18} />
                    </ThemeIcon>
                    <div>
                      <Text fw={500}>Explore Full Diagram</Text>
                      <Text size="xs" c="dimmed">
                        Navigate the complete system model
                      </Text>
                    </div>
                  </Group>
                </Paper>
              </Grid.Col>
            </Grid>
          </div>
        </Stack>
      </ScrollArea>
    </div>
  );
}

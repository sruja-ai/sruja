import { useEffect, useState, useMemo } from "react";
import { Shield, AlertTriangle, XCircle, RefreshCw, ChevronDown, ChevronUp } from "lucide-react";
import { Button } from "@sruja/ui";
import { Paper, Text, Stack, Group, Badge, Collapse } from "@mantine/core";
import { useArchitectureStore } from "../../stores/architectureStore";
import { useToastStore } from "../../stores/toastStore";
import { useUIStore } from "../../stores/uiStore";
import { useSelectionStore } from "../../stores/viewStore";
import { getWasmApi, logger } from "@sruja/shared";
import "./GovernanceWidget.css";

// Interface for ScoreCard returned by WASM
interface Deduction {
  Rule: string;
  Points: number;
  Message: string;
  Target: string;
  Severity: "Critical" | "Warning" | "Info";
}

interface CategoryScores {
  Structural: number;
  Documentation: number;
  Traceability: number;
  Complexity: number;
  Standardization: number;
}

interface ScoreCard {
  Score: number;
  Grade: string;
  Categories: CategoryScores;
  Deductions: Deduction[];
}

export function GovernanceWidget() {
  const dslSource = useArchitectureStore((s) => s.dslSource);
  const [scoreCard, setScoreCard] = useState<ScoreCard | null>(null);
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const showToast = useToastStore((s) => s.showToast);

  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const setCodeTab = useUIStore((s) => s.setCodeTab);
  const setTargetLine = useUIStore((s) => s.setTargetLine);
  const selectNode = useSelectionStore((s) => s.selectNode);

  const handleTargetClick = (target: string) => {
    if (!target) return;

    // Parse "filename:line" pattern, e.g. "input.sruja:46"
    const parts = target.split(":");
    if (parts.length >= 2) {
      const line = parseInt(parts[1], 10);
      if (!isNaN(line)) {
        setTargetLine(line);
        setCodeTab("dsl");
        setActiveTab("code");
        return;
      }
    }

    // Fallback: If no line but just ID (e.g. "web.api"), select it
    selectNode(target, "navigation");
    setCodeTab("dsl");
    setActiveTab("code");
  };

  const calculateScore = async () => {
    if (!dslSource) {
      setScoreCard(null);
      return;
    }

    setLoading(true);
    try {
      const api = await getWasmApi();
      if (!api) {
        setScoreCard(null);
        return;
      }

      // Validate DSL by attempting to parse it first
      try {
        await api.dslToModel(dslSource);
      } catch (parseError) {
        setScoreCard(null);
        if (loading === false) {
          showToast("DSL syntax is invalid. Please fix errors before calculating score.", "error");
        }
        return;
      }

      // DSL is valid, proceed with score calculation
      const result = await api.calculateArchitectureScore(dslSource);
      setScoreCard(result as unknown as ScoreCard);
    } catch (error) {
      setScoreCard(null);
      if (loading === false) {
        showToast("Could not calculate architecture score.", "error");
      }
      if (process.env.NODE_ENV === "development") {
        logger.debug("Score calculation failed", {
          component: "GovernanceWidget",
          action: "calculateScore",
          error: error instanceof Error ? error.message : String(error),
        });
      }
    } finally {
      setLoading(false);
    }
  };

  // Calculate score on mount and when DSL changes
  useEffect(() => {
    calculateScore();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dslSource]);

  const groupedDeductions = useMemo(() => {
    if (!scoreCard) return { Critical: [], Warning: [], Info: [] };

    return scoreCard.Deductions.reduce(
      (acc, d) => {
        const severity = d.Severity || "Info";
        if (!acc[severity]) acc[severity] = [];
        acc[severity].push(d);
        return acc;
      },
      { Critical: [], Warning: [], Info: [] } as Record<string, Deduction[]>
    );
  }, [scoreCard]);

  const getHealthColor = (grade: string) => {
    switch (grade) {
      case "A":
        return "var(--mantine-color-green-filled)";
      case "B":
        return "var(--mantine-color-blue-filled)";
      case "C":
        return "var(--mantine-color-yellow-filled)";
      case "D":
        return "var(--mantine-color-red-filled)";
      default:
        return "var(--mantine-color-gray-filled)";
    }
  };

  if (loading && !scoreCard) {
    return (
      <Paper withBorder p="md" radius="md">
        <Text size="sm" c="dimmed">
          Calculating Architecture Score...
        </Text>
      </Paper>
    );
  }

  if (!scoreCard) {
    return null;
  }

  const criticalCount = groupedDeductions.Critical.length;
  const warningCount = groupedDeductions.Warning.length;
  const infoCount = groupedDeductions.Info.length;
  const totalIssues = criticalCount + warningCount + infoCount;

  const handleCardClick = (e: React.MouseEvent) => {
    // Don't toggle if clicking on buttons or interactive elements
    const target = e.target as HTMLElement;
    if (
      target.closest("button") ||
      target.closest("code") ||
      target.closest(".governance-deduction-item")
    ) {
      return;
    }
    if (totalIssues > 0) {
      setExpanded(!expanded);
    }
  };

  return (
    <Paper
      withBorder
      p="md"
      radius="md"
      className={`governance-widget ${totalIssues > 0 ? "governance-widget-clickable" : ""} ${expanded ? "governance-widget-expanded" : ""}`}
      onClick={handleCardClick}
      style={{ cursor: totalIssues > 0 ? "pointer" : "default" }}
    >
      <Stack gap="sm">
        {/* Header */}
        <Group justify="space-between" align="center">
          <Group gap="xs">
            <Shield size={20} className="governance-icon" />
            <Text fw={600} size="md">
              Architecture Health
            </Text>
            <Badge
              size="lg"
              style={{
                backgroundColor: getHealthColor(scoreCard.Grade),
                color: "#fff",
                fontWeight: "bold",
              }}
            >
              {scoreCard.Grade}
            </Badge>
          </Group>
          <Group gap="xs">
            {totalIssues > 0 && (
              <Badge
                variant="light"
                color={criticalCount > 0 ? "red" : warningCount > 0 ? "yellow" : "blue"}
                className="governance-issues-badge"
              >
                {totalIssues} {totalIssues === 1 ? "issue" : "issues"}
              </Badge>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={(e) => {
                e.stopPropagation();
                calculateScore();
              }}
              disabled={loading}
              title="Refresh score"
            >
              <RefreshCw size={14} className={loading ? "spin" : ""} />
            </Button>
            {totalIssues > 0 && (
              <div className="governance-expand-indicator">
                {expanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
              </div>
            )}
          </Group>
        </Group>

        {/* Score Display */}
        <Group gap="md">
          <div
            className="governance-score-ring"
            style={{ borderColor: getHealthColor(scoreCard.Grade) }}
          >
            <Text fw={700} size="xl">
              {scoreCard.Score}
            </Text>
            <Text size="xs" c="dimmed">
              /100
            </Text>
          </div>
          <Stack gap="xs" style={{ flex: 1 }}>
            <Text size="sm" fw={500}>
              Architecture Health Index (AHI)
            </Text>
            <Group gap="md">
              <div>
                <Text size="xs" c="dimmed">
                  Structural
                </Text>
                <Text size="sm" fw={600}>
                  {scoreCard.Categories.Structural}
                </Text>
              </div>
              <div>
                <Text size="xs" c="dimmed">
                  Documentation
                </Text>
                <Text size="sm" fw={600}>
                  {scoreCard.Categories.Documentation}
                </Text>
              </div>
              <div>
                <Text size="xs" c="dimmed">
                  Traceability
                </Text>
                <Text size="sm" fw={600}>
                  {scoreCard.Categories.Traceability}
                </Text>
              </div>
            </Group>
          </Stack>
        </Group>

        {/* Expandable Details - Show All Issues */}
        {totalIssues > 0 && (
          <Collapse in={expanded}>
            <Stack gap="md" mt="md" className="governance-issues-list">
              {/* Critical Issues */}
              {criticalCount > 0 && (
                <div className="governance-deduction-section critical">
                  <Group gap="xs" mb="sm">
                    <XCircle size={18} color="var(--mantine-color-red-filled)" />
                    <Text fw={700} size="sm" c="red">
                      Critical Issues ({criticalCount})
                    </Text>
                  </Group>
                  <Stack gap="xs">
                    {groupedDeductions.Critical.map((d, i) => (
                      <div
                        key={i}
                        className="governance-deduction-item"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <Group gap="xs" align="flex-start" mb={4}>
                          <Text size="xs" fw={600} c="red" style={{ minWidth: "60px" }}>
                            -{d.Points}
                          </Text>
                          <div style={{ flex: 1 }}>
                            <Text size="xs" fw={600} mb={2}>
                              {d.Rule}
                            </Text>
                            <Text size="xs" c="dimmed" mb={d.Target ? 4 : 0}>
                              {d.Message}
                            </Text>
                            {d.Target && (
                              <Text
                                size="xs"
                                component="code"
                                className="governance-target-link"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleTargetClick(d.Target);
                                }}
                              >
                                {d.Target}
                              </Text>
                            )}
                          </div>
                        </Group>
                      </div>
                    ))}
                  </Stack>
                </div>
              )}

              {/* Warnings */}
              {warningCount > 0 && (
                <div className="governance-deduction-section warning">
                  <Group gap="xs" mb="sm">
                    <AlertTriangle size={18} color="var(--mantine-color-yellow-filled)" />
                    <Text fw={700} size="sm" c="yellow">
                      Warnings ({warningCount})
                    </Text>
                  </Group>
                  <Stack gap="xs">
                    {groupedDeductions.Warning.map((d, i) => (
                      <div
                        key={i}
                        className="governance-deduction-item"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <Group gap="xs" align="flex-start" mb={4}>
                          <Text size="xs" fw={600} c="yellow" style={{ minWidth: "60px" }}>
                            -{d.Points}
                          </Text>
                          <div style={{ flex: 1 }}>
                            <Text size="xs" fw={600} mb={2}>
                              {d.Rule}
                            </Text>
                            <Text size="xs" c="dimmed" mb={d.Target ? 4 : 0}>
                              {d.Message}
                            </Text>
                            {d.Target && (
                              <Text
                                size="xs"
                                component="code"
                                className="governance-target-link"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleTargetClick(d.Target);
                                }}
                              >
                                {d.Target}
                              </Text>
                            )}
                          </div>
                        </Group>
                      </div>
                    ))}
                  </Stack>
                </div>
              )}

              {/* Info Issues */}
              {infoCount > 0 && (
                <div className="governance-deduction-section info">
                  <Group gap="xs" mb="sm">
                    <AlertTriangle size={18} color="var(--mantine-color-blue-filled)" />
                    <Text fw={700} size="sm" c="blue">
                      Info ({infoCount})
                    </Text>
                  </Group>
                  <Stack gap="xs">
                    {groupedDeductions.Info.map((d, i) => (
                      <div
                        key={i}
                        className="governance-deduction-item"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <Group gap="xs" align="flex-start" mb={4}>
                          <Text size="xs" fw={600} c="blue" style={{ minWidth: "60px" }}>
                            -{d.Points}
                          </Text>
                          <div style={{ flex: 1 }}>
                            <Text size="xs" fw={600} mb={2}>
                              {d.Rule}
                            </Text>
                            <Text size="xs" c="dimmed" mb={d.Target ? 4 : 0}>
                              {d.Message}
                            </Text>
                            {d.Target && (
                              <Text
                                size="xs"
                                component="code"
                                className="governance-target-link"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleTargetClick(d.Target);
                                }}
                              >
                                {d.Target}
                              </Text>
                            )}
                          </div>
                        </Group>
                      </div>
                    ))}
                  </Stack>
                </div>
              )}
            </Stack>
          </Collapse>
        )}
      </Stack>
    </Paper>
  );
}

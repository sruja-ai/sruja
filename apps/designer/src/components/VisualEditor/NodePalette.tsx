// apps/designer/src/components/VisualEditor/NodePalette.tsx
import { Paper, Stack, Text, Button, Group } from "@mantine/core";
import { useVisualEditorStore, type SrujaNodeType } from "../../stores/visualEditorStore";
import { useTheme } from "@sruja/ui";
import { User, Box, Container, Package, Database, MessageSquare } from "lucide-react";

interface NodeTypeInfo {
  type: SrujaNodeType;
  label: string;
  icon: React.ComponentType<{ size?: number }>;
  description: string;
}

const NODE_TYPES: NodeTypeInfo[] = [
  {
    type: "person",
    label: "Person",
    icon: User,
    description: "External user or actor",
  },
  {
    type: "system",
    label: "System",
    icon: Box,
    description: "Software system",
  },
  {
    type: "container",
    label: "Container",
    icon: Container,
    description: "Deployment unit",
  },
  {
    type: "component",
    label: "Component",
    icon: Package,
    description: "Component within a container",
  },
  {
    type: "datastore",
    label: "Data Store",
    icon: Database,
    description: "Database or storage",
  },
  {
    type: "queue",
    label: "Queue",
    icon: MessageSquare,
    description: "Message queue or stream",
  },
];

export function NodePalette() {
  const { selectedNodeType, setSelectedNodeType, activeTool } = useVisualEditorStore();
  const { mode } = useTheme();
  const isDark =
    mode === "dark" ||
    (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  return (
    <Paper
      shadow="md"
      p="md"
      radius="md"
      withBorder
      style={{
        backgroundColor: isDark ? "rgba(30, 30, 30, 0.95)" : "rgba(255, 255, 255, 0.95)",
        backdropFilter: "blur(8px)",
      }}
    >
      <Stack gap="xs">
        <Text fw={700} size="sm">
          Create Node
        </Text>
        <Text size="xs" c="dimmed" style={{ wordBreak: "break-word" }}>
          Select a type, then click canvas to create
        </Text>
        <Stack gap={4}>
          {NODE_TYPES.map((nodeType) => {
            const Icon = nodeType.icon;
            const isSelected = activeTool === "create-node" && selectedNodeType === nodeType.type;

            return (
              <Button
                key={nodeType.type}
                variant={isSelected ? "filled" : "light"}
                color={isSelected ? "blue" : "gray"}
                size="sm"
                leftSection={<Icon size={16} />}
                onClick={() => setSelectedNodeType(nodeType.type)}
                style={{
                  justifyContent: "flex-start",
                }}
                title={nodeType.description}
              >
                <Group gap="xs" justify="space-between" style={{ width: "100%" }}>
                  <Text size="sm">{nodeType.label}</Text>
                </Group>
              </Button>
            );
          })}
        </Stack>
      </Stack>
    </Paper>
  );
}

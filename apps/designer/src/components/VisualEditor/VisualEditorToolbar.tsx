// apps/designer/src/components/VisualEditor/VisualEditorToolbar.tsx
import { Group, Button, Tooltip, Divider } from "@mantine/core";
import { useVisualEditorStore } from "../../stores/visualEditorStore";
import { MousePointer2, Plus, Link2, Layout } from "lucide-react";

export function VisualEditorToolbar() {
  const { activeTool, setActiveTool, isManualMode, setManualMode } = useVisualEditorStore();

  return (
    <Group gap="xs">
      <Tooltip label="Select and move nodes" position="bottom">
        <Button
          variant={activeTool === "select" || activeTool === null ? "filled" : "light"}
          color={activeTool === "select" || activeTool === null ? "blue" : "gray"}
          size="sm"
          leftSection={<MousePointer2 size={16} />}
          onClick={() => setActiveTool("select")}
        >
          Select
        </Button>
      </Tooltip>
      <Tooltip label="Create new nodes" position="bottom">
        <Button
          variant={activeTool === "create-node" ? "filled" : "light"}
          color={activeTool === "create-node" ? "blue" : "gray"}
          size="sm"
          leftSection={<Plus size={16} />}
          onClick={() => setActiveTool("create-node")}
        >
          Create
        </Button>
      </Tooltip>
      <Tooltip label="Connect nodes" position="bottom">
        <Button
          variant={activeTool === "connect" ? "filled" : "light"}
          color={activeTool === "connect" ? "blue" : "gray"}
          size="sm"
          leftSection={<Link2 size={16} />}
          onClick={() => setActiveTool("connect")}
        >
          Connect
        </Button>
      </Tooltip>

      <Divider orientation="vertical" />

      <Tooltip
        label={isManualMode ? "Disable Manual Layout" : "Enable Manual Layout"}
        position="bottom"
      >
        <Button
          variant={isManualMode ? "filled" : "light"}
          color={isManualMode ? "orange" : "gray"}
          size="sm"
          leftSection={<Layout size={16} />}
          onClick={() => setManualMode(!isManualMode)}
        >
          Manual Mode
        </Button>
      </Tooltip>
    </Group>
  );
}

import {
  Paper,
  Text,
  Stack,
  Slider,
  Group,
  ThemeIcon,
  Badge,
  RingProgress,
  Center,
} from "@mantine/core";
import { Users, AlertTriangle } from "lucide-react";
import { useArchitectureStore } from "../../stores";

export function CapacityController() {
  const capacityState = useArchitectureStore((s) => s.capacityState);
  const setCapacityLoad = useArchitectureStore((s) => s.setCapacityLoad);

  const loadColor =
    capacityState.userLoad > 200 ? "red" : capacityState.userLoad > 120 ? "orange" : "blue";

  return (
    <Paper shadow="md" p="md" radius="md" withBorder w={320}>
      <Stack gap="sm">
        <Group justify="space-between">
          <Group gap="xs">
            <ThemeIcon variant="light" color={loadColor}>
              <Users size={18} />
            </ThemeIcon>
            <Text fw={600} size="sm">
              Traffic Simulation
            </Text>
          </Group>
          {capacityState.userLoad > 100 && (
            <Badge color={loadColor} variant="filled">
              +{capacityState.userLoad - 100}% Load
            </Badge>
          )}
        </Group>

        <Group align="flex-start" mt="xs">
          <RingProgress
            size={80}
            roundCaps
            thickness={8}
            sections={[{ value: Math.min(capacityState.userLoad / 3, 100), color: loadColor }]}
            label={
              <Center>
                <Stack gap={0} align="center">
                  <Text fw={700} size="xs">
                    {capacityState.userLoad}%
                  </Text>
                  <Text size="xs" c="dimmed" style={{ fontSize: 9 }}>
                    NOMINAL
                  </Text>
                </Stack>
              </Center>
            }
          />
          <Stack gap={4} style={{ flex: 1 }}>
            <Text size="xs" fw={500}>
              Estimated Requests/Sec
            </Text>
            <Text fw={700} size="xl">
              {(2500 * (capacityState.userLoad / 100)).toLocaleString()}
            </Text>
            <Text size="xs" c="dimmed">
              Global edge traffic
            </Text>
          </Stack>
        </Group>

        <Stack gap={2} mt="xs">
          <Group justify="space-between">
            <Text size="xs" fw={500}>
              User Load Multiplier
            </Text>
            <Text size="xs" c="dimmed">
              {capacityState.userLoad}%
            </Text>
          </Group>
          <Slider
            value={capacityState.userLoad}
            onChange={setCapacityLoad}
            min={0}
            max={300}
            step={10}
            color={loadColor}
            marks={[
              { value: 100, label: "1x" },
              { value: 200, label: "2x" },
              { value: 300, label: "3x" },
            ]}
            mb="lg"
          />
        </Stack>

        {capacityState.userLoad > 150 && (
          <Paper withBorder p="xs" bg="var(--mantine-color-red-0)">
            <Group gap="xs">
              <AlertTriangle size={16} color="var(--mantine-color-red-7)" />
              <Text size="xs" c="red.9">
                High load may degrade latency in non-scaled services.
              </Text>
            </Group>
          </Paper>
        )}
      </Stack>
    </Paper>
  );
}

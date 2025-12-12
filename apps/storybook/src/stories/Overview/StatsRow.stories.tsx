
import type { Meta, StoryObj } from '@storybook/react';
import { StatsRow } from '../../../../apps/playground/src/components/Overview/StatsRow';

const meta = {
    title: 'Overview/StatsRow',
    component: StatsRow,
    parameters: {
        layout: 'padded',
    },
    tags: ['autodocs'],
    argTypes: {
    },
} satisfies Meta<typeof StatsRow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: {
        stats: {
            systems: 5,
            persons: 3,
            containers: 12,
            components: 45,
            requirements: 20,
            adrs: 15,
            policies: 8,
            flows: 30,
        }
    },
};

export const Empty: Story = {
    args: {
        stats: {
            systems: 0,
            persons: 0,
            containers: 0,
            components: 0,
            requirements: 0,
            adrs: 0,
            policies: 0,
            flows: 0,
        }
    },
};

export const PartialStats: Story = {
    args: {
        stats: {
            systems: 1,
            persons: 1,
            containers: 0,
            components: 0,
            requirements: 0,
            adrs: 0,
            policies: 0,
            flows: 0,
        }
    },
};

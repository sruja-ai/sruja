
import type { Meta, StoryObj } from '@storybook/react';
import { GoalsSection } from '../../../../apps/playground/src/components/Overview/GoalsSection';
import { fn } from '@storybook/test';

const meta = {
    title: 'Overview/GoalsSection',
    component: GoalsSection,
    parameters: {
        layout: 'padded',
    },
    tags: ['autodocs'],
    args: {
        onEdit: fn(),
    }
} satisfies Meta<typeof GoalsSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: {
        goals: [
            'Scale to 1 million daily users',
            'Reduce latency to < 100ms',
            'Ensure 99.99% availability'
        ],
        nonGoals: [
            'Support legacy browsers (IE11)',
            'Offline capability for version 1'
        ],
        context: 'We are rebuilding the legacy monolith into microservices.',
        risks: [
            'Migration could take longer than expected',
            'Data consistency during transition'
        ]
    },
};

export const OnlyGoals: Story = {
    args: {
        goals: [
            'Launch MVP by Q3',
        ],
        nonGoals: [],
        context: '',
        risks: []
    },
};

export const Empty: Story = {
    args: {
        goals: [],
        nonGoals: [],
        context: '',
        risks: []
    },
};

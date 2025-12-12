
import type { Meta, StoryObj } from '@storybook/react';
import { PoliciesSection } from '../../../../apps/playground/src/components/Overview/PoliciesSection';
import { fn } from '@storybook/test';

const meta = {
    title: 'Overview/PoliciesSection',
    component: PoliciesSection,
    parameters: {
        layout: 'padded',
    },
    tags: ['autodocs'],
    args: {
        onEdit: fn(),
    }
} satisfies Meta<typeof PoliciesSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: {
        policies: [
            { id: 'pol-1', title: 'Data Privacy', description: 'All PII must be encrypted at rest and in transit.', category: 'Security' },
            { id: 'pol-2', title: 'Logging', description: 'All services must log structured JSON to the central aggregator.', category: 'Operations' },
        ]
    },
};

export const Empty: Story = {
    args: {
        policies: []
    },
};

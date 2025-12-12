
import type { Meta, StoryObj } from '@storybook/react';
import { ConstraintsSection } from '../../../../apps/playground/src/components/Overview/ConstraintsSection';
import { fn } from '@storybook/test';

const meta = {
    title: 'Overview/ConstraintsSection',
    component: ConstraintsSection,
    parameters: {
        layout: 'padded',
    },
    tags: ['autodocs'],
    args: {
        onEdit: fn(),
    }
} satisfies Meta<typeof ConstraintsSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: {
        constraints: [
            { id: 'con-1', title: 'Budget', description: 'Monthly cloud spend must not exceed $5000.', category: 'Financial' },
            { id: 'con-2', title: 'Compliance', description: 'Must be GDPR compliant.', category: 'Legal' },
        ]
    },
};

export const Empty: Story = {
    args: {
        constraints: []
    },
};

// apps/storybook/src/stories/Switch.stories.tsx
import { useState } from 'react'
import { Switch } from '../../../../packages/ui/src/components/Switch'
import { Card } from '../../../../packages/ui/src/components/Card'
import type { Meta, StoryObj } from '@storybook/react'

const meta: Meta<typeof Switch> = {
  title: 'Components/Switch',
  component: Switch,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Toggle switch component for boolean settings and preferences. Used in Sruja Studio for toggling features, view options, and configuration settings.',
      },
    },
  },
  argTypes: {
    checked: {
      control: { type: 'boolean' },
      description: 'Whether the switch is checked',
    },
    disabled: {
      control: { type: 'boolean' },
      description: 'Disables the switch',
    },
    label: {
      control: { type: 'text' },
      description: 'Label text displayed next to the switch',
    },
  },
}

export default meta
type Story = StoryObj<typeof Switch>

export const Playground: Story = {
  render: function PlaygroundComponent() {
    const [checked, setChecked] = useState(false)
    return <Switch checked={checked} onChange={setChecked} label="Enable feature" />
  },
}

export const Basic: Story = {
  render: function BasicComponent() {
    const [checked, setChecked] = useState(false)
    return <Switch checked={checked} onChange={setChecked} label="Auto-save architecture" />
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic switch for toggling a single setting.',
      },
    },
  },
}

export const SettingsPanel: Story = {
  render: function SettingsPanelComponent() {
    const [autoSave, setAutoSave] = useState(true)
    const [showGrid, setShowGrid] = useState(false)
    const [darkMode, setDarkMode] = useState(false)
    const [animations, setAnimations] = useState(true)
    const [validation, setValidation] = useState(true)

    return (
      <Card title="Settings" subtitle="Application preferences">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <Switch
            checked={autoSave}
            onChange={setAutoSave}
            label="Auto-save architecture"
          />
          <Switch
            checked={showGrid}
            onChange={setShowGrid}
            label="Show grid in diagram"
          />
          <Switch
            checked={darkMode}
            onChange={setDarkMode}
            label="Dark mode"
          />
          <Switch
            checked={animations}
            onChange={setAnimations}
            label="Enable animations"
          />
          <Switch
            checked={validation}
            onChange={setValidation}
            label="Real-time validation"
          />
        </div>
      </Card>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'Multiple switches in a settings panel for configuring Studio preferences.',
      },
    },
  },
}

export const States: Story = {
  render: function StatesComponent() {
    const [checked1, setChecked1] = useState(false)
    const [checked2, setChecked2] = useState(true)
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
        <Switch checked={checked1} onChange={setChecked1} label="Unchecked" />
        <Switch checked={checked2} onChange={setChecked2} label="Checked" />
        <Switch checked={false} onChange={() => {}} label="Disabled (unchecked)" disabled />
        <Switch checked={true} onChange={() => {}} label="Disabled (checked)" disabled />
      </div>
    )
  },
  parameters: {
    docs: {
      description: {
        story: 'All switch states: unchecked, checked, and disabled variants.',
      },
    },
  },
}

export const Showcase: Story = {
  render: function ShowcaseComponent() {
    const [a, setA] = useState(true)
    const [b, setB] = useState(false)
    const [c, setC] = useState(true)
    return (
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-md">
        <div className="space-y-3">
          <Switch checked={a} onChange={setA} label="Auto-save" />
          <Switch checked={b} onChange={setB} label="Show grid" />
          <Switch checked={c} onChange={setC} label="Enable animations" />
        </div>
      </div>
    )
  },
}

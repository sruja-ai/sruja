// apps/storybook/src/stories/DSL/Requirements.stories.tsx
// Stories demonstrating DSL requirements

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Requirements',
  parameters: {
    docs: {
      description: {
        component: 'Requirements define what the system must do or constraints it must satisfy.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicRequirements = `model {
  requirement R1 functional "System must handle user authentication"
  requirement R2 performance "API response time must be < 100ms"
  requirement R3 security "All data must be encrypted at rest"
}`

const requirementsWithDetails = `model {
  requirement R1 functional "System must handle user authentication" {
    description "Users must be able to log in with email and password"
    priority "high"
    status "in-progress"
  }
  
  requirement R2 performance "API response time must be < 100ms" {
    description "95th percentile response time should be under 100ms"
    priority "high"
    status "accepted"
  }
  
  requirement R3 constraint "Must use PostgreSQL for data persistence" {
    description "Database choice is constrained by existing infrastructure"
    priority "medium"
    status "accepted"
  }
}`

const requirementTypes = `model {
  requirement R1 functional "User can create account"
  requirement R2 nonfunctional "System must be available 99.9% of the time"
  requirement R3 performance "Page load time < 2 seconds"
  requirement R4 security "All API calls must be authenticated"
  requirement R5 compliance "Must comply with GDPR regulations"
  requirement R6 scalability "Must support 1M concurrent users"
}`

export const Basic: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {basicRequirements}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic requirements with different types.',
      },
    },
  },
}

export const WithDetails: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {requirementsWithDetails}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Requirements with description, priority, and status.',
      },
    },
  },
}

export const RequirementTypes: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {requirementTypes}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different requirement types: functional, nonfunctional, performance, security, compliance, scalability.',
      },
    },
  },
}

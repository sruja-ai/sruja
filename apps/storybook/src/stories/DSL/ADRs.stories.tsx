// apps/storybook/src/stories/DSL/ADRs.stories.tsx
// Stories demonstrating DSL Architecture Decision Records (ADRs)

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/ADRs',
  parameters: {
    docs: {
      description: {
        component: 'Architecture Decision Records document important architectural decisions and their rationale.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicADR = `model {
  adr ADR001 "Use microservices architecture" {
    status "Accepted"
    context "Need to scale independently"
    decision "Adopt microservices"
    consequences "Better scalability, increased complexity"
  }
}`

const adrWithDetails = `model {
  adr ADR001 "Use PostgreSQL for data persistence" {
    status "Accepted"
    context "Need ACID transactions and strong consistency"
    decision "Use PostgreSQL as primary database"
    consequences "Strong consistency, SQL complexity, good ecosystem support"
    date "2024-01-15"
    author "Engineering Team"
  }
}`

const adrStatuses = `model {
  adr ADR001 "Proposed: Use GraphQL" {
    status "Proposed"
    context "Need flexible API queries"
    decision "Evaluate GraphQL"
    consequences "More flexible queries, additional complexity"
  }
  
  adr ADR002 "Accepted: Use REST API" {
    status "Accepted"
    context "Team familiar with REST"
    decision "Use REST API"
    consequences "Faster development, standard patterns"
  }
  
  adr ADR003 "Deprecated: Monolithic architecture" {
    status "Deprecated"
    context "Legacy system"
    decision "Migrate to microservices"
    consequences "Better scalability"
  }
}`

export const Basic: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {basicADR}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic ADR with required fields.',
      },
    },
  },
}

export const WithDetails: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {adrWithDetails}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'ADR with date and author information.',
      },
    },
  },
}

export const ADRStatuses: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {adrStatuses}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'ADRs with different statuses: Proposed, Accepted, Deprecated.',
      },
    },
  },
}

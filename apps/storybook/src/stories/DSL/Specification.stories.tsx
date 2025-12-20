// apps/storybook/src/stories/DSL/Specification.stories.tsx
// Stories demonstrating DSL specification block

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Specification',
  parameters: {
    docs: {
      description: {
        component: 'The specification block defines element types and tags that can be used in the model.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicSpecification = `specification {
  element person
  element system
  element container
  element component
  element database
}`

const specificationWithTags = `specification {
  element person
  element system
  element container
  element component
  element database
  element queue
  
  tag external
  tag deprecated
  tag critical
}`

const fullSpecification = `specification {
  element person
  element system
  element container
  element component
  element database
  element queue
  element topic
  element cache
  element filesystem
  element deployment
  
  tag external
  tag deprecated
  tag critical
  tag frontend
  tag backend
  tag infrastructure
}`

export const Basic: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {basicSpecification}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic specification with common element types.',
      },
    },
  },
}

export const WithTags: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {specificationWithTags}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Specification with custom tags for categorization.',
      },
    },
  },
}

export const Full: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {fullSpecification}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete specification with all element types and tags.',
      },
    },
  },
}

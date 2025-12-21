// apps/storybook/src/stories/DSL/Views.stories.tsx
// Stories demonstrating DSL views block

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Views',
  parameters: {
    docs: {
      description: {
        component: 'The views block defines different perspectives of the architecture model.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicView = `views {
  view index {
    title "System Overview"
    include *
  }
}`

const multipleViews = `views {
  view landscape {
    title "System Landscape"
    include *
  }
  
  view containers of ecommerce {
    title "E-Commerce Containers"
    include ecommerce.*
  }
  
  view components of ecommerce.api {
    title "API Components"
    include ecommerce.api.*
  }
}`

const viewsWithExclude = `views {
  view developer of analyticsPlatform {
    title "Developer View"
    include analyticsPlatform.api
    include analyticsPlatform.graphqlApi
    include analyticsPlatform.queryService
    exclude analyticsPlatform.dashboard
    exclude analyticsPlatform.adminPanel
  }
  
  view product of analyticsPlatform {
    title "Product View"
    include analyticsPlatform.dashboard
    include analyticsPlatform.adminPanel
    include analyticsPlatform.api
    exclude analyticsPlatform.dataIngestion
  }
}`

const viewsWithTags = `views {
  view frontend {
    title "Frontend Components"
    tags ["frontend", "ui"]
    include *
  }
  
  view backend {
    title "Backend Services"
    tags ["backend", "api"]
    include *
  }
}`

export const Basic: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {basicView}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic view that includes all elements.',
      },
    },
  },
}

export const MultipleViews: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {multipleViews}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Multiple views showing different levels of detail (landscape, containers, components).',
      },
    },
  },
}

export const WithExclude: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {viewsWithExclude}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Views with include and exclude patterns for fine-grained control.',
      },
    },
  },
}

export const WithTags: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {viewsWithTags}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Views with tags for categorization and filtering.',
      },
    },
  },
}

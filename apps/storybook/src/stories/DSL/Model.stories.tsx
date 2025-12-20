// apps/storybook/src/stories/DSL/Model.stories.tsx
// Stories demonstrating DSL model block with advanced features

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Model',
  parameters: {
    docs: {
      description: {
        component: 'The model block contains all architecture elements, relationships, and Sruja extensions (requirements, policies, ADRs, scenarios, flows).',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicModel = `model {
  customer = person "End User"
  admin = person "Administrator"
  
  ecommerce = system "E-Commerce Platform" {
    webApp = container "Web Application" {
      technology "React"
    }
    api = container "API Service" {
      technology "Go"
    }
    db = database "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  customer -> ecommerce.webApp "uses"
  ecommerce.webApp -> ecommerce.api "calls"
  ecommerce.api -> ecommerce.db "reads and writes to"
}`

const modelWithMetadata = `model {
  customer = person "End User" {
    description "Primary user of the system"
    tags ["external"]
  }
  
  ecommerce = system "E-Commerce Platform" {
    description "Main e-commerce application"
    technology "Microservices"
    tags ["core", "backend"]
    
    metadata {
      team "Platform Team"
      owner "engineering@example.com"
      version "2.0"
    }
    
    webApp = container "Web Application" {
      technology "React, TypeScript"
      description "Customer-facing web interface"
    }
  }
  
  customer -> ecommerce.webApp "browses and purchases"
}`

const modelWithSrujaExtensions = `model {
  customer = person "End User"
  admin = person "Administrator"
  
  ecommerce = system "E-Commerce Platform" {
    webApp = container "Web Application"
    api = container "API Service"
    db = database "PostgreSQL Database"
  }
  
  customer -> ecommerce.webApp "uses"
  ecommerce.webApp -> ecommerce.api "calls"
  ecommerce.api -> ecommerce.db "reads and writes to"
  
  // Requirements
  requirement R1 functional "System must handle 10k concurrent users"
  requirement R2 constraint "Must use PostgreSQL for data persistence"
  requirement R3 performance "API response time must be < 100ms"
  
  // Policies
  policy P1 security "All API endpoints must use HTTPS"
  policy P2 compliance "GDPR compliance required for user data"
  
  // ADRs
  adr ADR001 "Adopt microservices architecture" {
    status "Accepted"
    context "Need to scale independently"
    decision "Adopt microservices"
    consequences "Better scalability, increased complexity"
  }
  
  // Scenarios
  scenario S1 "User purchases product" {
    step "User adds item to cart"
    step "User proceeds to checkout"
    step "Payment is processed"
    step "Order confirmation is sent"
  }
  
  // Flows
  flow F1 "Order Processing Flow" {
    step "Receive order" from customer to ecommerce.webApp
    step "Validate order" from ecommerce.webApp to ecommerce.api
    step "Process payment" from ecommerce.api to paymentGateway
    step "Store order" from ecommerce.api to ecommerce.db
  }
}`

const modelWithBidirectionalRelations = `model {
  customer = person "End User"
  ecommerce = system "E-Commerce Platform" {
    api = container "API Service"
  }
  
  // Bidirectional relationship
  customer <-> ecommerce.api "interacts with"
  
  // Back arrow
  ecommerce.api <- customer "receives requests from"
}`

export const Basic: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px', fontSize: '14px' }}>
      {basicModel}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic model with nested elements and relationships.',
      },
    },
  },
}

export const WithMetadata: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px', fontSize: '14px' }}>
      {modelWithMetadata}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Model with descriptions, technologies, tags, and metadata blocks.',
      },
    },
  },
}

export const WithSrujaExtensions: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px', fontSize: '14px', maxHeight: '600px', overflow: 'auto' }}>
      {modelWithSrujaExtensions}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Model with Sruja extensions: requirements, policies, ADRs, scenarios, and flows.',
      },
    },
  },
}

export const BidirectionalRelations: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px', fontSize: '14px' }}>
      {modelWithBidirectionalRelations}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Model demonstrating bidirectional (<->) and back arrows (<-).',
      },
    },
  },
}

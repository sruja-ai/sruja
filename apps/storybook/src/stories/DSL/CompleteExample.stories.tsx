// apps/storybook/src/stories/DSL/CompleteExample.stories.tsx
// Complete DSL example with all advanced features

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Complete Example',
  parameters: {
    docs: {
      description: {
        component: 'A complete DSL example demonstrating all advanced features: specification, model with extensions, and views.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const completeExample = `specification {
  element person
  element system
  element container
  element component
  element database
  element queue
}

model {
  // Actors
  customer = person "Customer" {
    description "End user who browses and purchases products"
  }
  
  admin = person "Administrator" {
    description "Platform administrator managing products and orders"
  }
  
  // Main system
  ecommerce = system "E-Commerce Platform" {
    description "Comprehensive platform for online retail"
    
    webApp = container "Web Application" {
      description "React-based storefront"
      technology "React, TypeScript"
      tags ["frontend", "ui"]
    }
    
    api = container "REST API" {
      description "Core business logic API"
      technology "Go, Gin"
      tags ["backend", "api"]
    }
    
    db = database "PostgreSQL" {
      description "Primary transactional database"
      technology "PostgreSQL 15"
      tags ["database", "persistence"]
    }
    
    cache = database "Redis" {
      description "Session and cache store"
      technology "Redis 7"
      tags ["database", "cache"]
    }
    
    events = queue "Event Queue" {
      description "Async event processing"
      technology "RabbitMQ"
      tags ["queue", "messaging"]
    }
  }
  
  // External systems
  payments = system "Payment Gateway" {
    description "External payment processor"
    tags ["external"]
  }
  
  shipping = system "Shipping Provider" {
    description "External shipping integration"
    tags ["external"]
  }
  
  // Relationships
  customer -> ecommerce.webApp "browses and purchases"
  admin -> ecommerce.webApp "manages products"
  
  ecommerce.webApp -> ecommerce.api "API calls"
  ecommerce.api -> ecommerce.db "reads/writes"
  ecommerce.api -> ecommerce.cache "caches"
  ecommerce.api -> ecommerce.events "publishes events"
  ecommerce.api -> payments "processes payments"
  ecommerce.api -> shipping "creates shipments"
  
  // Sruja Extensions
  requirement R1 functional "Persist user data"
  requirement R2 constraint "Expose public API"
  requirement R3 performance "Fast checkout flow"
  requirement R4 security "Secure payment processing"
  
  policy P1 security "All API endpoints must use HTTPS"
  policy P2 compliance "GDPR compliance required for user data"
  policy P3 reliability "99.9% uptime SLA"
  
  adr ADR001 "Adopt Stripe for payments" {
    status "Accepted"
    context "Need payment processing"
    decision "Use Stripe API"
    consequences "Easy integration, good documentation"
  }
  
  adr ADR002 "Use PostgreSQL for strong consistency" {
    status "Accepted"
    context "Need ACID transactions"
    decision "Use PostgreSQL"
    consequences "Strong consistency, SQL complexity"
  }
  
  scenario S1 "User purchases product" {
    step "User adds item to cart"
    step "User proceeds to checkout"
    step "Payment is processed"
    step "Order confirmation is sent"
  }
  
  flow F1 "Order Processing Flow" {
    step "Receive order" from customer to ecommerce.webApp
    step "Validate order" from ecommerce.webApp to ecommerce.api
    step "Process payment" from ecommerce.api to payments
    step "Store order" from ecommerce.api to ecommerce.db
    step "Send confirmation" from ecommerce.api to customer
  }
}

views {
  view landscape {
    title "E-Commerce System Landscape"
    include *
  }
  
  view containers of ecommerce {
    title "E-Commerce Containers"
    include ecommerce.*
  }
  
  view backend of ecommerce {
    title "Backend Services"
    include ecommerce.api
    include ecommerce.db
    include ecommerce.cache
    include ecommerce.events
    exclude ecommerce.webApp
  }
}`

export const FullExample: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px', fontSize: '13px', maxHeight: '700px', overflow: 'auto' }}>
      {completeExample}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete DSL example showing specification, model with all features, and multiple views.',
      },
    },
  },
}

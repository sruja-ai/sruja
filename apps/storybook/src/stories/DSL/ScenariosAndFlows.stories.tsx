// apps/storybook/src/stories/DSL/Scenarios and Flows
// Stories demonstrating DSL scenarios and flows

import type { Meta, StoryObj } from '@storybook/react'
import React from 'react'

const meta: Meta = {
  title: 'DSL/Scenarios and Flows',
  parameters: {
    docs: {
      description: {
        component: 'Scenarios and flows describe user interactions and system behaviors.',
      },
    },
  },
}

export default meta
type Story = StoryObj

const basicScenario = `model {
  scenario S1 "User purchases product" {
    step "User adds item to cart"
    step "User proceeds to checkout"
    step "Payment is processed"
    step "Order confirmation is sent"
  }
}`

const scenarioWithSteps = `model {
  scenario S1 "User registration flow" {
    description "New user creates an account"
    step "User enters email and password"
    step "System validates input"
    step "System creates user account"
    step "System sends verification email"
    step "User verifies email"
  }
}`

const flowWithRelations = `model {
  customer = person "Customer"
  ecommerce = system "E-Commerce" {
    webApp = container "Web App"
    api = container "API"
    db = database "Database"
  }
  
  flow F1 "Order Processing Flow" {
    description "Complete order processing workflow"
    step "Receive order" from customer to ecommerce.webApp
    step "Validate order" from ecommerce.webApp to ecommerce.api
    step "Process payment" from ecommerce.api to paymentGateway
    step "Store order" from ecommerce.api to ecommerce.db
    step "Send confirmation" from ecommerce.api to customer
  }
}`

const multipleScenarios = `model {
  scenario S1 "User login" {
    step "User enters credentials"
    step "System validates credentials"
    step "System creates session"
    step "User is redirected to dashboard"
  }
  
  scenario S2 "User logout" {
    step "User clicks logout"
    step "System invalidates session"
    step "User is redirected to login"
  }
  
  scenario S3 "Password reset" {
    step "User requests password reset"
    step "System sends reset email"
    step "User clicks reset link"
    step "User sets new password"
    step "System updates password"
  }
}`

export const BasicScenario: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {basicScenario}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Basic scenario with simple steps.',
      },
    },
  },
}

export const ScenarioWithDescription: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {scenarioWithSteps}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Scenario with description and detailed steps.',
      },
    },
  },
}

export const FlowWithRelations: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {flowWithRelations}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Flow with steps that reference elements and relationships.',
      },
    },
  },
}

export const MultipleScenarios: Story = {
  render: () => (
    <div style={{ padding: '1rem', fontFamily: 'monospace', whiteSpace: 'pre-wrap', background: '#f5f5f5', borderRadius: '4px' }}>
      {multipleScenarios}
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Multiple scenarios describing different user flows.',
      },
    },
  },
}

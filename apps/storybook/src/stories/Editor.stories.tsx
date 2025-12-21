// apps/storybook/src/stories/Editor.stories.tsx
import { useState } from "react";
import { SrujaMonacoEditor } from "../../../../packages/ui/src/components/SrujaMonacoEditor";
import type { Meta, StoryObj } from "@storybook/react";

const meta: Meta<typeof SrujaMonacoEditor> = {
  title: "Components/Editor",
  component: SrujaMonacoEditor,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "Monaco-based code editor with Sruja language support. Provides syntax highlighting, autocomplete, and validation for architecture-as-code DSL. Used in Sruja Studio for editing architecture definitions.",
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof SrujaMonacoEditor>;

const SIMPLE_EXAMPLE = `specification {
  element person
  element system
  element container
  element database
}

model {
  user = person "End User"
  
  app = system "My App" {
    web = container "Web Server" {
      technology "Node.js"
    }
    db = database "Database" {
      technology "PostgreSQL"
    }
  }
  
  user -> app.web "Visits"
  app.web -> app.db "Reads/Writes"
}

views {
  view index {
    title "System Overview"
    include *
  }
}`;

const ECOMMERCE_EXAMPLE = `specification {
  element person
  element system
  element container
  element database
}

model {
  customer = person "Customer"
  admin = person "Administrator"
  
  ecommerce = system "E-Commerce Platform" {
    webApp = container "Web Application" {
      technology "Next.js"
    }
    api = container "API Gateway" {
      technology "Express.js"
    }
    payment = container "Payment Service" {
      technology "Go"
    }
    inventory = container "Inventory Service" {
      technology "Python"
    }
    
    userDB = database "User Database" {
      technology "PostgreSQL"
    }
    productDB = database "Product Database" {
      technology "MongoDB"
    }
    orderDB = database "Order Database" {
      technology "PostgreSQL"
    }
  }
  
  customer -> ecommerce.webApp "Browses"
  customer -> ecommerce.webApp "Purchases"
  ecommerce.webApp -> ecommerce.api "Requests"
  ecommerce.api -> ecommerce.payment "Processes"
  ecommerce.api -> ecommerce.inventory "Checks"
  ecommerce.payment -> ecommerce.orderDB "Stores"
  ecommerce.inventory -> ecommerce.productDB "Queries"
  admin -> ecommerce.api "Manages"
}

views {
  view landscape {
    title "E-Commerce Landscape"
    include *
  }
  
  view containers of ecommerce {
    title "E-Commerce Containers"
    include ecommerce.*
  }
}`;

export const Basic: Story = {
  render: function BasicComponent() {
    const [value, setValue] = useState(SIMPLE_EXAMPLE);
    return (
      <div
        style={{
          height: "400px",
          border: "1px solid var(--color-border)",
          borderRadius: 8,
          overflow: "hidden",
        }}
      >
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: "Basic Sruja editor with simple architecture definition.",
      },
    },
  },
};

export const ComplexArchitecture: Story = {
  render: function ComplexArchitectureComponent() {
    const [value, setValue] = useState(ECOMMERCE_EXAMPLE);
    return (
      <div
        style={{
          height: "500px",
          border: "1px solid var(--color-border)",
          borderRadius: 8,
          overflow: "hidden",
        }}
      >
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: "Editor with complex e-commerce platform architecture definition.",
      },
    },
  },
};

export const SplitView: Story = {
  render: function SplitViewComponent() {
    const [value, setValue] = useState(SIMPLE_EXAMPLE);
    return (
      <div
        style={{
          display: "flex",
          height: "500px",
          border: "1px solid var(--color-border)",
          borderRadius: 8,
          overflow: "hidden",
        }}
      >
        <div style={{ width: "50%", borderRight: "1px solid var(--color-border)" }}>
          <div
            style={{
              padding: 8,
              borderBottom: "1px solid var(--color-border)",
              fontSize: 12,
              fontWeight: 600,
              backgroundColor: "var(--color-surface)",
            }}
          >
            Editor
          </div>
          <div style={{ height: "calc(100% - 40px)" }}>
            <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
          </div>
        </div>
        <div
          style={{
            width: "50%",
            backgroundColor: "var(--color-surface)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--color-text-secondary)",
          }}
        >
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 8 }}>Diagram Preview</div>
            <div style={{ fontSize: 14 }}>Interactive viewer would appear here</div>
          </div>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story:
          "Split view layout with editor on left and diagram preview on right, as used in Studio.",
      },
    },
  },
};

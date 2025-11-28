// Auto-generated from examples/manifest.json - DO NOT EDIT MANUALLY
// Run: go run scripts/generate-playground-examples.go

import type { PlaygroundExample } from './types';

export const EXAMPLES: PlaygroundExample[] = [
  {
    name: "Quick Start",
    code: `architecture "Simple Example" {
  person User "End User"
  
  system API "API Service" {
    container WebApp "Web Application" {
      technology "React"
    }
    
    container Database "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  User -> WebApp "Uses"
  WebApp -> Database "Reads/Writes"
  
  requirement R1 functional "Must handle 10k concurrent users"
  requirement R2 constraint "Must use PostgreSQL"
  
  adr ADR001 "Use microservices architecture for scalability"
}`
  },
  {
    name: "Basic Example",
    code: `architecture "Basic Example" {
  person User "User"
  system SoftwareSystem "Software System" {
    container WebApp "Web Application" {
    }
    container Database "Database" {
    }
  }

  User -> WebApp "Uses"
  WebApp -> Database "Reads/Writes"
}`
  },
  {
    name: "Person & Relations",
    code: `architecture "Person Example" {
  person Customer "Customer"

  system Order "Order System" {
    container API "Order API" {
      component Checkout "Checkout" {}
    }
  }

  Customer -> API "Uses"
  API -> Checkout "Calls"
}`
  },
  {
    name: "Components",
    code: `architecture "Components Relations Example" {
  system Shop "Shop" {
    container Web "Web App" {
      component Cart "Cart" {
      }
      component Auth "Auth" {
      }
    }
    Cart -> Auth uses "Uses"
  }
}`
  },
  {
    name: "Person to Component",
    code: `architecture "Person to Component Example" {
  person User "User"
  system App "App" {
    container Web "Web" {
      component Checkout "Checkout" {
      }
    }
  }
  User -> Checkout uses "Uses"
}`
  },
  {
    name: "Queues & DataStore",
    code: `architecture "Queues and DataStore Example" {
  system Service "Service" {
    container API "API" {
    }
    queue MQ "Message Queue"
    datastore DB "Database"
  }
  API -> MQ publishes "Publishes messages"
  API -> DB reads "Reads data"
  API -> DB writes "Writes data"
}`
  },
  {
    name: "Tags",
    code: `architecture "Tags Showcase Example" {
  system App "Application" {
    container API "HTTP API" {
      tags ["http", "public"]
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
  }
  API -> DB reads "Reads data"
  API -> DB writes "Writes data"
  App -> MQ publishes "Publishes events"
}`
  },
  {
    name: "Requirements",
    code: `architecture "Requirements Implements Example" {
  system API "API" {
  }
  system App "Application" {
    container DB "Database" {
    }
  }
  API -> DB reads "Reads data"
  App -> API uses "Uses"
  requirement R1 functional "Persist data"
  requirement R2 constraint "Expose API"
}`
  },
  {
    name: "Integration Verbs",
    code: `architecture "Integration Verbs and Elements Example" {
  person User "User"
  system App "Application" {
    container API "API" {
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
  }
  User -> API uses "Uses"
  API -> DB reads "Reads data"
  API -> DB writes "Writes data"
  App -> MQ publishes "Publishes events"
}`
  },
  {
    name: "External Systems",
    code: `architecture "External OK Example" {
  system App "Application" {
    container API "API" {
    }
  }
  App -> API uses "Uses"
}`
  },
  {
    name: "Full Features",
    code: `architecture "Full Features Example" {
  person User "User"
  system System "My System" {
    container API "API Service" {
    }
  }
  User -> System "Uses"
  System -> API "Calls"

  requirement R1 functional "Must handle 10k RPS"
  requirement R2 constraint "Must be written in Go"

  adr ADR001 "Use Microservices"
}`
  },
  {
    name: "E‑Commerce",
    code: `architecture "Full MVP Example" {
  person Customer "Customer"

  system App "Application" {
    container API "HTTP API" {
      tags ["http", "public"]
    }
    datastore DB "PostgreSQL"
    queue MQ "Events"
    container Web "Web App" {
      technology "React"
      component Cart "Cart" {
      }
      component Auth "Auth" {
      }
      Cart -> Auth uses "Uses"
    }
  }

  Customer -> API uses "Uses"
  API -> DB reads "Reads data"
  API -> DB writes "Writes data"
  App -> MQ publishes "Publishes events"

  requirement R1 functional "Persist user data"
  requirement R2 constraint "Expose public API"
  requirement R3 performance "Fast checkout"

  adr ADR001 "Adopt Stripe for payments"
}`
  },
  {
    name: "C4 Complete",
    code: `architecture "C4 Complete Example" {

  person User "End User"
  system WebApp "Web Application" {
    container API "API Service"
    container DB "Database"
  }

  // New Deployment View
  deployment Prod "Production" {
    node AWS "AWS" {
      node USEast1 "US-East-1" {
         infrastructure LB "Load Balancer"
         containerInstance API
      }
      node RDS "RDS" {
         containerInstance DB
      }
    }
  }

  // Login Flow Relations (dynamic view converted to relations)
  User -> API "Credentials"
  API -> DB "Validate User"
}`
  },
  {
    name: "Top-Level System",
    code: `// Example: Top-level system without architecture wrapper
system API "API Service" {
  container WebApp "Web Application" {
    component Controller "Controller"
  }
  datastore DB "Database"
}`
  },
  {
    name: "Top-Level Container",
    code: `// Example: Top-level container without architecture wrapper
container WebApp "Web Application" {
  component Frontend "Frontend"
  component Backend "Backend"
}`
  },
  {
    name: "Top-Level Component",
    code: `// Example: Top-level component without architecture wrapper
component AuthService "Authentication Service" {
  metadata {
    team: "Security"
    critical: "true"
  }
}`
  },
  {
    name: "Metadata",
    code: `architecture "Metadata Showcase Example" {
  system BillingAPI {
    metadata {
      team: "Payments"
      tier: "critical"
      rate_limit_per_ip: "50/s"
      error_budget: "99.5%"
      cloud: "aws"
      lambda_memory: "512MB"
      tracing: "enabled"
    }
    container API {
      metadata {
        rate_limit: "100/s"
        auth: "jwt"
        upstream: "BillingAPI"
      }
    }
  }
  
  datastore InvoiceDB {
    metadata {
      engine: "PostgreSQL"
      version: "14"
      encryption_at_rest: "AES-256"
      compliance: "PCI-DSS"
    }
  }
  
  queue Events {
    metadata {
      provider: "RabbitMQ"
      durable: "true"
      prefetch: "10"
    }
  }
  
  person Customer {
    metadata {
      persona: "end-user"
      access_level: "standard"
    }
  }
}`
  },
  {
    name: "Imports",
    code: `architecture "Import Main Example" {
  system Shared "Shared" {
    datastore SharedDB "Shared DB"
  }
  system App "App" {
    container API "API" {
    }
  }
  App -> Shared uses "Uses"
  API -> SharedDB reads "Reads data"
}`
  },
];

# Sruja DSL Cheatsheet & Quick Reference

A comprehensive quick reference guide for Sruja DSL syntax, patterns, and best practices. Perfect for learners working through challenges and experienced users needing a syntax refresher.

## 🚀 Quick Start Syntax

### Basic Architecture Structure
```sruja
architecture "System Name" {
  person User "Description"
  
  system SystemName {
    container ContainerName "Description" {
      technology "Technology Stack"
    }
    component ComponentName "Description"
    datastore DataStore "Database Description"
    queue QueueName "Message Queue"
  }
  
  User -> SystemName "Uses"
  SystemName.ContainerName -> SystemName.DataStore "Reads/Writes"
}
```

## 📋 Core Elements Reference

### Persons (Users)
```sruja
person Customer "Online shopper who browses and purchases products"
person Admin "System administrator who manages the platform"
person ExternalAPI "External system that integrates with our API"
```

### Systems
```sruja
system ECommerce "Online shopping platform" {
  // Contains containers, components, datastores
}

system ExternalService "Third-party payment processor" {
  tags ["external"]
}
```

### Containers
```sruja
container WebApp "React single-page application" {
  technology "React, TypeScript, TailwindCSS"
}

container APIGateway "API Gateway and load balancer" {
  technology "Kong, NGINX"
  tags ["gateway", "infrastructure"]
}
```

### Components
```sruja
component AuthService "Handles user authentication and authorization" {
  technology "Node.js, JWT"
}

component OrderProcessor "Processes incoming orders and manages inventory" {
  technology "Java, Spring Boot"
}
```

### Datastores
```sruja
datastore UserDB "Stores user profiles and authentication data" {
  technology "PostgreSQL 14"
  tags ["primary", "sensitive"]
}

datastore Cache "Redis cache for session and frequently accessed data" {
  technology "Redis 7.0"
  tags ["cache", "session"]
}
```

### Queues
```sruja
queue OrderQueue "Handles order processing events" {
  technology "RabbitMQ, AMQP"
}

queue NotificationQueue "Manages email and SMS notifications" {
  technology "AWS SQS"
  tags ["async", "notification"]
}
```

## 🔗 Relations Syntax

### Basic Relations
```sruja
// Person to System
User -> SystemName "Uses"

// System to System
Frontend -> Backend "Calls"

// Container to Container (within same system)
SystemName.WebApp -> SystemName.API "Makes API calls"

// Cross-system container relations
FrontendSystem.WebApp -> BackendSystem.API "Sends requests"
```

### Relation with Technology
```sruja
WebApp -> API "Makes REST calls" {
  technology "HTTPS/JSON"
  tags ["synchronous"]
}
```

### Bidirectional Relations
```sruja
// Explicit both directions
WebApp -> API "Sends requests"
API -> WebApp "Returns responses"

// Single relation with description
WebApp <> API "Two-way communication"
```

## 🏗️ Deployment Architecture

### Basic Deployment
```sruja
deployment Production "Production Environment" {
  node AWS "Amazon Web Services" {
    infrastructure ALB "Application Load Balancer"
    containerInstance WebApp
    containerInstance API
  }
}
```

### Multi-Node Deployment
```sruja
deployment Staging "Staging Environment" {
  node Kubernetes "K8s Cluster" {
    node FrontendPods "Frontend Pods" {
      containerInstance WebApp
    }
    
    node BackendPods "Backend Pods" {
      containerInstance API
      containerInstance Worker
    }
    
    node Database "Database Tier" {
      infrastructure PostgreSQL "Primary Database"
      containerInstance MainDB
    }
  }
}
```

### Geographic Deployment
```sruja
deployment Global "Global Deployment" {
  node US "US-East Region" {
    infrastructure CDN "CloudFront CDN"
    containerInstance WebApp
  }
  
  node EU "EU-West Region" {
    infrastructure CDN "CloudFront CDN"
    containerInstance WebApp
  }
  
  node Asia "Asia-Pacific Region" {
    infrastructure CDN "CloudFront CDN"
    containerInstance WebApp
  }
}
```

## 🏷️ Metadata and Tags

### Adding Metadata
```sruja
system ECommerce {
  metadata {
    owner "Platform Team"
    criticality "high"
    compliance "PCI-DSS"
    sla "99.9%"
  }
  
  container WebApp {
    metadata {
      team "Frontend Team"
      repository "github.com/company/webapp"
      deployment "kubernetes"
    }
  }
}
```

### Using Tags
```sruja
container PaymentService {
  technology "Stripe API"
  tags ["external", "payment", "critical"]
}

datastore UserData {
  technology "PostgreSQL"
  tags ["sensitive", "encrypted", "pii"]
}
```

## 📊 Requirements and ADRs

### Requirements
```sruja
requirement R001 functional "System must handle 10,000 concurrent users"
requirement R002 nonfunctional "Response time must be under 200ms"
requirement R003 constraint "Must use PostgreSQL as primary database"
requirement R004 security "All API endpoints must be authenticated"
```

### Architecture Decision Records (ADRs)
```sruja
adr ADR001 "Use microservices architecture for scalability" {
  status "accepted"
  date "2024-01-15"
  context "Growing team and feature complexity"
  decision "Adopt microservices with service mesh"
  consequences "Increased operational complexity, better team autonomy"
}

adr ADR002 "Choose PostgreSQL over MySQL" {
  status "accepted"
  date "2024-01-20"
  context "Need for complex queries and JSON support"
  decision "Use PostgreSQL with JSONB columns"
  consequences "Better query capabilities, team needs training"
}
```

## 🔄 Common Patterns

### API Gateway Pattern
```sruja
architecture "API Gateway Pattern" {
  person Client "API Consumer"
  
  system GatewayLayer {
    container APIGateway "API Gateway" {
      technology "Kong"
    }
  }
  
  system BackendServices {
    container UserService "User Management"
    container OrderService "Order Processing"
    container PaymentService "Payment Processing"
  }
  
  Client -> GatewayLayer.APIGateway "Makes requests"
  APIGateway -> UserService "Routes requests"
  APIGateway -> OrderService "Routes requests"
  APIGateway -> PaymentService "Routes requests"
}
```

### Event-Driven Architecture
```sruja
architecture "Event-Driven System" {
  person User "Application User"
  
  system EventPlatform {
    container API "REST API"
    queue EventBus "Central Event Bus"
    container OrderService "Order Handler"
    container InventoryService "Inventory Handler"
    container NotificationService "Notification Handler"
  }
  
  User -> API "Creates order"
  API -> EventBus "Publishes order.created"
  EventBus -> OrderService "Delivers event"
  EventBus -> InventoryService "Delivers event"
  EventBus -> NotificationService "Delivers event"
}
```

### Microservices Pattern
```sruja
architecture "Microservices Architecture" {
  person User "End User"
  
  system Frontend {
    container WebApp "Single Page Application"
  }
  
  system Backend {
    container UserAPI "User Management Service"
    container OrderAPI "Order Management Service"
    container ProductAPI "Product Catalog Service"
    container PaymentAPI "Payment Processing Service"
    
    datastore UserDB "User Database"
    datastore OrderDB "Order Database"
    datastore ProductDB "Product Database"
  }
  
  User -> WebApp "Uses"
  WebApp -> UserAPI "Authenticates"
  WebApp -> OrderAPI "Places orders"
  WebApp -> ProductAPI "Browses products"
  WebApp -> PaymentAPI "Processes payment"
  
  UserAPI -> UserDB "Stores user data"
  OrderAPI -> OrderDB "Stores orders"
  ProductAPI -> ProductDB "Stores products"
}
```

## 🎯 Validation and Best Practices

### Common Validation Rules
```sruja
// ✅ Good: Clear, descriptive names
person Customer "Online customer who purchases products"

// ❌ Bad: Vague or generic names
person User "User"
```

### Best Practices
1. **Use descriptive names**: Be specific about what each element does
2. **Add technology details**: Specify the actual technology stack
3. **Include metadata**: Add team ownership, SLA requirements
4. **Use tags consistently**: Create a tagging standard for your organization
5. **Keep relations focused**: Each relation should have a clear purpose

### Architecture Validation Checklist
- [ ] All persons have meaningful interactions
- [ ] Systems have clear boundaries
- [ ] Containers specify technology
- [ ] Relations have descriptive labels
- [ ] External systems are properly tagged
- [ ] Deployment matches actual infrastructure
- [ ] Security boundaries are clear

## 🔧 Quick Fixes for Common Issues

### Missing Relations
```sruja
// Problem: Components exist but aren't connected
system App {
  container WebApp "Frontend"
  container API "Backend"
}

// Solution: Add meaningful relations
WebApp -> API "Makes API calls"
```

### Syntax Errors
```sruja
// Problem: Missing quotes or braces
container WebApp Web Application  // ❌ Missing quotes

// Solution: Proper syntax
container WebApp "Web Application"  // ✅ Correct
```

### Hierarchy Issues
```sruja
// Problem: Wrong nesting
system App {
  container API "API Service"
  component Database "PostgreSQL"  // ❌ Component should be datastore
}

// Solution: Correct element types
system App {
  container API "API Service"
  datastore Database "PostgreSQL"  // ✅ Correct type
}
```

## 📚 Learning Path Integration

### Beginner Focus Areas
- Basic syntax and structure
- Person → System → Container hierarchy
- Simple relations with descriptions
- Understanding element types

### Intermediate Focus Areas
- Deployment architecture
- External system integration
- Metadata and tagging
- Complex relations

### Advanced Focus Areas
- Multi-system architectures
- Event-driven patterns
- Security architecture
- Performance optimization

## 🚀 Pro Tips

### Efficiency Shortcuts
- Use consistent naming conventions
- Create reusable component templates
- Leverage tags for filtering and organization
- Add metadata early in the design process

### Team Collaboration
- Establish team tagging standards
- Use metadata for ownership and SLAs
- Document architectural decisions with ADRs
- Version control your architecture files

### Integration with Development
- Generate documentation from Sruja files
- Use in CI/CD for architecture validation
- Create architecture review checklists
- Link to actual code repositories

## 📖 Related Resources

- **Sruja Language Specification**: Complete syntax reference
- **Architecture Patterns Guide**: Common design patterns
- **Best Practices Document**: Organizational standards
- **Challenge Library**: Hands-on practice problems

---

*This cheatsheet covers the essential Sruja DSL syntax. For complete language specification and advanced features, refer to the official documentation.*
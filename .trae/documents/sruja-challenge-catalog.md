# Sruja DSL Challenge Catalog

A comprehensive collection of practical challenges designed to teach Sruja DSL through architecture fixing exercises. Each challenge presents a broken architecture that learners must fix by writing actual Sruja code.

## Challenge Categories

### 🟢 Beginner Challenges (Foundation)

#### 1. Missing Relations Challenge
**Difficulty**: Beginner | **Topic**: Relations | **Est. Time**: 5-10 min

**Problem**: A simple e-commerce system has components but no connections between them.

**Broken Architecture**:
```sruja
architecture "Shop" {
  person Customer "Online Shopper"
  
  system WebShop {
    container WebApp "Web Application"
    container API "API Service"
    datastore DB "PostgreSQL Database"
  }
  
  // TODO: Add relations to connect the system
}
```

**Issues to Fix**:
- Missing relation: Customer -> WebApp
- Missing relation: WebApp -> API
- Missing relation: API -> DB

**Learning Objectives**:
- Understand basic relation syntax (`->`)
- Learn to connect persons to systems
- Practice component-to-component relations

**Solution**:
```sruja
architecture "Shop" {
  person Customer "Online Shopper"
  
  system WebShop {
    container WebApp "Web Application"
    container API "API Service"
    datastore DB "PostgreSQL Database"
  }
  
  Customer -> WebApp "Uses"
  Customer -> WebShop.WebApp "Browses"
  WebApp -> API "Calls"
  API -> DB "Reads/Writes"
}
```

---

#### 2. Incomplete System Challenge
**Difficulty**: Beginner | **Topic**: Components | **Est. Time**: 8-12 min

**Problem**: A microservices architecture is missing key components.

**Broken Architecture**:
```sruja
architecture "E-Commerce Platform" {
  person User "Customer"
  
  system Frontend {
    container WebApp "React Frontend"
    // TODO: Add missing API Gateway
  }
  
  system Backend {
    // TODO: Add missing services
  }
  
  User -> WebApp "Uses"
  // TODO: Add missing relations
}
```

**Issues to Fix**:
- Missing API Gateway component
- Missing Order Service, Product Service, User Service
- Missing relations between components

**Learning Objectives**:
- Understand container vs component hierarchy
- Practice creating multiple containers
- Learn service-to-service communication

---

#### 3. Syntax Error Challenge
**Difficulty**: Beginner | **Topic**: Validation | **Est. Time**: 5-8 min

**Problem**: The architecture has syntax errors preventing parsing.

**Broken Architecture**:
```sruja
architecture "Broken System" {
  person User "End User"
  
  system App {
    container WebApp "Web Application"
    container API "API Service"  // Missing closing brace
    
    datastore DB "Database"
  }
  
  User -> WebApp "Uses"
  WebApp -> API "Calls"  // Missing quotes
  API -> DB "Reads/Writes"
}
```

**Issues to Fix**:
- Missing closing brace for API container
- Missing quotes around relation description
- Potential indentation issues

**Learning Objectives**:
- Understand Sruja syntax rules
- Learn to read error messages
- Practice proper code formatting

---

### 🟡 Intermediate Challenges (Application)

#### 4. Deployment Architecture Challenge
**Difficulty**: Intermediate | **Topic**: Deployment | **Est. Time**: 15-20 min

**Problem**: A working system needs proper deployment architecture.

**Broken Architecture**:
```sruja
architecture "E-Commerce Platform" {
  person Customer "Online Shopper"
  
  system WebShop {
    container WebApp "React Frontend"
    container API "Node.js API"
    datastore DB "PostgreSQL Database"
    queue OrderQueue "Redis Queue"
  }
  
  Customer -> WebApp "Uses"
  WebApp -> API "Calls"
  API -> DB "Reads/Writes"
  API -> OrderQueue "Publishes"
  
  // TODO: Add deployment architecture
}
```

**Issues to Fix**:
- Missing deployment nodes and infrastructure
- No environment separation (prod/staging)
- Missing deployment relations

**Learning Objectives**:
- Understand deployment modeling
- Learn node and infrastructure concepts
- Practice production-ready architecture

**Solution Preview**:
```sruja
deployment Production "Production Environment" {
  node AWS "AWS Cloud" {
    infrastructure CloudFront "CDN"
    infrastructure ALB "Application Load Balancer"
    
    node ECS "ECS Cluster" {
      containerInstance WebApp
      containerInstance API
    }
    
    node RDS "RDS Instance" {
      containerInstance DB
    }
    
    node ElastiCache "ElastiCache" {
      infrastructure OrderQueue
    }
  }
}
```

---

#### 5. External Integration Challenge
**Difficulty**: Intermediate | **Topic**: External Systems | **Est. Time**: 12-18 min

**Problem**: A payment system needs external service integration.

**Broken Architecture**:
```sruja
architecture "Payment System" {
  person User "Customer"
  
  system PaymentGateway {
    container API "Payment API"
    container WebApp "Payment Frontend"
    datastore Transactions "Transaction DB"
  }
  
  // TODO: Add external payment processors
  // TODO: Add fraud detection service
  // TODO: Add proper external relations
  
  User -> WebApp "Makes Payment"
  WebApp -> API "Processes"
  API -> Transactions "Stores"
}
```

**Issues to Fix**:
- Missing external payment processors (Stripe, PayPal)
- Missing fraud detection integration
- Need external system boundaries

**Learning Objectives**:
- Understand external system modeling
- Learn secure integration patterns
- Practice third-party service design

---

#### 6. Performance Optimization Challenge
**Difficulty**: Intermediate | **Topic**: Scalability | **Est. Time**: 18-25 min

**Problem**: A social media platform architecture has performance bottlenecks.

**Broken Architecture**:
```sruja
architecture "Social Platform" {
  person User "Social User"
  
  system Platform {
    container WebApp "Web Frontend"
    container API "Monolithic API"  // TODO: Should be microservices
    datastore MainDB "PostgreSQL"
    
    // TODO: Add caching layer
    // TODO: Add CDN
    // TODO: Add read replicas
  }
  
  User -> WebApp "Uses"
  WebApp -> API "Calls"
  API -> MainDB "Reads/Writes"
}
```

**Issues to Fix**:
- Monolithic API should be split into microservices
- Missing caching layer (Redis)
- No CDN for static content
- Missing read replica databases

**Learning Objectives**:
- Understand scalability patterns
- Learn caching strategies
- Practice microservice decomposition

---

### 🔴 Advanced Challenges (Mastery)

#### 7. Multi-System Architecture Challenge
**Difficulty**: Advanced | **Topic**: System Design | **Est. Time**: 25-35 min

**Problem**: A complex enterprise system with multiple interconnected systems.

**Broken Architecture**:
```sruja
architecture "Enterprise Platform" {
  person Employee "Company Employee"
  person Customer "External Customer"
  
  // TODO: Multiple systems need proper integration
  system CRM {
    container API "CRM API"
    datastore CustomerData "Customer DB"
  }
  
  system ERP {
    container API "ERP API"
    datastore FinanceData "Finance DB"
  }
  
  system ECommerce {
    container WebApp "Online Store"
    container API "Commerce API"
    datastore Orders "Order DB"
  }
  
  // Missing integrations between systems
  // Missing event-driven architecture
  // Missing API gateway pattern
}
```

**Issues to Fix**:
- Missing system-to-system integrations
- No event-driven communication
- Missing API gateway for external access
- Need proper data synchronization

**Learning Objectives**:
- Understand enterprise integration patterns
- Learn event-driven architecture
- Practice API gateway design

---

#### 8. Security Architecture Challenge
**Difficulty**: Advanced | **Topic**: Security | **Est. Time**: 20-30 min

**Problem**: A financial system needs comprehensive security layers.

**Broken Architecture**:
```sruja
architecture "Banking System" {
  person Customer "Bank Customer"
  
  system BankingPlatform {
    container WebApp "Online Banking"
    container API "Banking API"
    datastore Accounts "Account DB"
    
    // TODO: Add security infrastructure
    // TODO: Add authentication service
    // TODO: Add audit logging
    // TODO: Add encryption service
  }
  
  Customer -> WebApp "Accesses"
  WebApp -> API "Requests"
  API -> Accounts "Manages"
}
```

**Issues to Fix**:
- Missing authentication/authorization service
- No audit logging for compliance
- Missing encryption for sensitive data
- No network security boundaries

**Learning Objectives**:
- Understand security architecture patterns
- Learn compliance requirements
- Practice defense-in-depth design

---

#### 9. Event-Driven Architecture Challenge
**Difficulty**: Advanced | **Topic**: Event-Driven | **Est. Time**: 30-40 min

**Problem**: An e-commerce platform needs event-driven architecture for scalability.

**Broken Architecture**:
```sruja
architecture "Event-Driven E-Commerce" {
  person Customer "Online Shopper"
  
  system Platform {
    container WebApp "Frontend"
    container API "REST API"  // TODO: Should be event-driven
    datastore MainDB "Database"
    
    // TODO: Add message broker
    // TODO: Add event handlers
    // TODO: Add saga pattern for transactions
  }
  
  // TODO: Add order processing workflow
  // TODO: Add inventory management events
  // TODO: Add payment processing events
  
  Customer -> WebApp "Places Order"
  WebApp -> API "Sends Request"
  API -> MainDB "Stores Data"
}
```

**Issues to Fix**:
- Replace REST API with event-driven architecture
- Add message broker (Kafka/RabbitMQ)
- Implement saga pattern for distributed transactions
- Add event handlers for different domains

**Learning Objectives**:
- Understand event-driven architecture
- Learn saga pattern implementation
- Practice asynchronous communication

---

## Challenge Implementation Guide

### Validation System

Each challenge includes automated validation checks:

```typescript
interface ChallengeCheck {
  type: 'noErrors' | 'relationExists' | 'componentExists' | 'deploymentValid'
  source?: string
  target?: string
  message: string
  hint?: string
}
```

**Example Validation Checks**:
```json
{
  "checks": [
    { "type": "noErrors", "message": "DSL parsed successfully" },
    { "type": "relationExists", "source": "Customer", "target": "WebApp", "message": "Customer should connect to WebApp" },
    { "type": "componentExists", "component": "API", "message": "API Gateway component is required" }
  ]
}
```

### Progressive Hints System

Challenges provide progressive hints to guide learners:

1. **Level 1**: Conceptual hint about what's missing
2. **Level 2**: Syntax example showing the correct pattern
3. **Level 3**: Partial solution with blanks to fill
4. **Level 4**: Complete solution with explanation

### Difficulty Assessment

**Beginner**: Focus on syntax, basic relations, simple fixes
**Intermediate**: Multiple components, deployment, external integration
**Advanced**: Multi-system, security, event-driven, performance optimization

## Learning Path Integration

### Module 1: Foundation (Beginner Challenges)
- Challenge 1: Missing Relations
- Challenge 2: Incomplete System
- Challenge 3: Syntax Error

### Module 2: Application (Intermediate Challenges)
- Challenge 4: Deployment Architecture
- Challenge 5: External Integration
- Challenge 6: Performance Optimization

### Module 3: Mastery (Advanced Challenges)
- Challenge 7: Multi-System Architecture
- Challenge 8: Security Architecture
- Challenge 9: Event-Driven Architecture

### Module 4: Real-World Scenarios
- E-commerce Platform Refactoring
- Microservices Migration
- Cloud-Native Transformation

## Assessment Criteria

### Code Quality Metrics
- **Syntax Correctness**: No parsing errors
- **Relation Completeness**: All required connections present
- **Architecture Validity**: Follows best practices
- **Performance Optimization**: Efficient component placement

### Learning Outcomes
- **Understanding**: Can
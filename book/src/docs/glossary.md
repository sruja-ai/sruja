---
title: "Glossary"
weight: 100
summary: "Definitions of key terms and concepts used in Sruja."
---

# Glossary

Quick reference for technical terms and concepts used throughout Sruja documentation.

## A

### ADR (Architecture Decision Record)

A document that captures an important architectural decision made along with its context and consequences. In Sruja, ADRs are defined using the `adr` keyword and can be linked to specific architecture elements.

**Example:**

```sruja
ADR001 = adr "Use Microservices" {
  status "Accepted"
  context "Need to scale independently"
  decision "Adopt microservices architecture"
  consequences "Increased complexity but better scalability"
}
```

### Architecture-as-Code

The practice of defining software architecture using code (text-based DSL) instead of static diagrams. This enables version control, validation, and automated documentation generation.

## C

### C4 Model

A hierarchical model for visualizing software architecture, created by Simon Brown. It consists of four levels:

- **Level 1 (Context):** System context diagram showing the system and its users
- **Level 2 (Container):** Container diagram showing high-level technical building blocks
- **Level 3 (Component):** Component diagram showing internal structure of containers
- **Level 4 (Code):** Code-level diagram (typically managed by IDEs, not Sruja)

Sruja is based on the C4 model and automatically generates C4-compliant diagrams.

### Component

A structural element within a container that represents a major building block. Components are optional and provide additional detail when needed.

**Example:**

```sruja
App = system "App" {
  API = container "API" {
    Auth = component "Authentication"
    Payment = component "Payment Processing"
  }
}
```

### Container

A deployable unit within a system. In C4 terminology, a container is NOT a Docker container, but rather any separately deployable unit like:

- Web applications
- Mobile apps
- Server-side applications
- Databases
- File systems

**Example:**

```sruja
App = system "E-commerce" {
  Web = container "React App"
  API = container "Node.js API"
  DB = database "PostgreSQL"
}
```

## D

### Database

A type of container that represents a data store. In Sruja, databases are defined using the `database` keyword.

**Example:**

```sruja
DB = database "PostgreSQL"
```

**Note:** Sruja also supports `datastore` as an alias, but `database` is the recommended term.

### DSL (Domain-Specific Language)

A programming language specialized for a particular domain. Sruja DSL is a text-based language specifically designed for defining software architecture.

## E

### Element

Any architectural construct in Sruja: person, system, container, component, database, queue, etc. Elements are the building blocks of your architecture model.

## K

### Kind

A type definition for elements. Kinds can be imported from the standard library (`import { * } from 'sruja.ai/stdlib'`) or declared manually for custom types.

**Example:**

```sruja
// Using stdlib (recommended)
import { * } from 'sruja.ai/stdlib'

// Or declaring manually
microservice = kind "Microservice"
```

## M

### Metadata

Additional information attached to elements, such as team ownership, tier, or custom tags. Metadata helps with governance and organization.

**Example:**

```sruja
App = system "My App" {
  metadata {
    team "Platform"
    tier "critical"
  }
}
```

## P

### Person

An actor or user of the system. Persons are defined at the context level and represent external users or roles.

**Example:**

```sruja
User = person "Customer"
Admin = person "Administrator"
```

## Q

### Queue

A message queue or event stream used for asynchronous communication between containers.

**Example:**

```sruja
EventQueue = queue "Kafka Topic"
```

## R

### Relation

A connection between two elements, showing how they interact. Relations are defined using the `->` operator.

**Example:**

```sruja
<!-- partial -->
User -> App.Web "Visits"
App.Web -> App.API "Calls"
```

### Requirement

A functional or non-functional requirement that can be linked to architecture elements using tags. Requirements help trace business needs to technical implementation.

**Example:**

```sruja
R001 = requirement "User Authentication" {
  description "Users must be able to log in securely"
  tags ["App.Web", "App.API"]
}
```

## S

### Scenario

A sequence of interactions that describe how users or systems interact to accomplish a goal. Scenarios help document user flows and system behavior.

**Example:**

```sruja
<!-- partial -->
scenario Checkout "Checkout Flow" {
  User -> App.Web "adds items to cart"
  App.Web -> App.API "validates cart"
  App.API -> App.DB "stores order"
}
```

### System

The highest-level element in C4 Level 1. A system represents a software system that delivers value to its users.

**Example:**

```sruja
ECommerce = system "E-commerce Platform" {
  description "Online shopping platform"
}
```

### stdlib (Standard Library)

The Sruja standard library that provides common element kinds (person, system, container, database, etc.). Importing from stdlib is the recommended way to use standard types.

**Example:**

```sruja
import { * } from 'sruja.ai/stdlib'
```

## T

### Tag

A label that can be attached to elements, requirements, or ADRs to enable filtering, grouping, and traceability.

**Example:**

```sruja
App = system "My App" {
  tags ["production", "critical"]
}
```

## V

### View

A diagram perspective that shows a subset of the architecture. Views can be explicit (defined with `view` blocks) or implicit (automatically generated by Sruja).

**Example:**

```sruja
view index {
  title "Complete System View"
  include *
}
```

## Related Resources

- [C4 Model Documentation](docs/concepts/c4-model.md)
- [DSL Syntax Reference](docs/reference/syntax.md)
- [Getting Started Guide](docs/getting-started.md)

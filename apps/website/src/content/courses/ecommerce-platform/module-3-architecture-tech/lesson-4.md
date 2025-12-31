---
title: "Lesson 4: API Design & Integration"
weight: 4
summary: "Design stable APIs and integrate with external services."
---

# Lesson 4: API Design & Integration

## Why API Design Matters

Well-designed APIs define stable interfaces between services; they reduce coupling and surprises. However, **API schemas belong in OpenAPI, not in Sruja**.

## Sruja's Role: Architecture Modeling

Sruja focuses on **architectural concerns**: which services exist, how they relate, and what they do. For detailed API schemas, use OpenAPI/Swagger.

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"
queue = kind "Queue"

customer = person "Customer"

ecommerce = system "E-Commerce Platform" {
  api = container "Checkout API" {
    technology "Go, Gin"
    // API details defined in openapi.yaml
  }

  events = queue "Order Events" {
    technology "Kafka"
    // Event schemas defined in AsyncAPI or JSON Schema
  }
}

customer -> ecommerce.api "uses"
```

## Best Practice

1. **Model architecture in Sruja**: Services, containers, relationships
2. **Define API schemas in OpenAPI**: Request/response structures, endpoints
3. **Link them**: Reference OpenAPI files in your architecture documentation

## Practice

- Model the `AddToCart` service in Sruja
- Create an OpenAPI spec for the `AddToCart` endpoint
- Show how they work together

---
title: "Microservices: Connect Service Mesh"
summary: "You're building a microservices architecture! Connect the UserService, OrderService, and PaymentService so they can communicate. Each service has its own database."
difficulty: beginner
topic: relations
estimatedTime: "5-10 min"
initialDsl: |
  architecture "Microservices Platform" {
    person Customer "Online Customer"
    
    system ECommerce {
      container UserService "User Management"
      container OrderService "Order Processing"
      container PaymentService "Payment Processing"
      datastore UserDB "User Database"
      datastore OrderDB "Order Database"
      datastore PaymentDB "Payment Database"
    }
    
    Customer -> UserService "Logs in"
    
    // TODO: Connect services in order flow: UserService -> OrderService -> PaymentService
    // TODO: Connect each service to its database
  }
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: relationExists
    source: UserService
    target: OrderService
    message: "Add relation UserService -> OrderService"
  - type: relationExists
    source: OrderService
    target: PaymentService
    message: "Add relation OrderService -> PaymentService"
  - type: relationExists
    source: UserService
    target: UserDB
    message: "Add relation UserService -> UserDB"
  - type: relationExists
    source: OrderService
    target: OrderDB
    message: "Add relation OrderService -> OrderDB"
  - type: relationExists
    source: PaymentService
    target: PaymentDB
    message: "Add relation PaymentService -> PaymentDB"
hints:
  - "Order flow: UserService -> OrderService \"Creates order\""
  - "Then: OrderService -> PaymentService \"Processes payment\""
  - "Each service connects to its own DB: Service -> DB \"Reads/Writes\""
  - "Use descriptive labels for each relation"
solution: |
  architecture "Microservices Platform" {
    person Customer "Online Customer"
    
    system ECommerce {
      container UserService "User Management"
      container OrderService "Order Processing"
      container PaymentService "Payment Processing"
      datastore UserDB "User Database"
      datastore OrderDB "Order Database"
      datastore PaymentDB "Payment Database"
    }
    
    Customer -> UserService "Logs in"
    UserService -> OrderService "Creates order"
    OrderService -> PaymentService "Processes payment"
    UserService -> UserDB "Reads/Writes"
    OrderService -> OrderDB "Reads/Writes"
    PaymentService -> PaymentDB "Reads/Writes"
  }
---

---
title: "Microservices: Connect Service Mesh"
summary: "You're building a microservices architecture! Connect the UserService, OrderService, and PaymentService so they can communicate. Each service has its own database."
difficulty: beginner
topic: relations
estimatedTime: "5-10 min"
initialDsl: |
  specification {
    element person
    element system
    element container
    element component
    element datastore
    element queue
    element external
  }
  
  model {
    Customer = person "Online Customer"
    
      ECommerce = system  {
        UserService = container "User Management"
        OrderService = container "Order Processing"
        PaymentService = container "Payment Processing"
        UserDB = datastore "User Database"
        OrderDB = datastore "Order Database"
        PaymentDB = datastore "Payment Database"
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
  specification {
    element person
    element system
    element container
    element component
    element datastore
    element queue
    element external
  }
  
  model {
    Customer = person "Online Customer"
    
      ECommerce = system  {
        UserService = container "User Management"
        OrderService = container "Order Processing"
        PaymentService = container "Payment Processing"
        UserDB = datastore "User Database"
        OrderDB = datastore "Order Database"
        PaymentDB = datastore "Payment Database"
      }
    
      Customer -> UserService "Logs in"
      UserService -> OrderService "Creates order"
      OrderService -> PaymentService "Processes payment"
      UserService -> UserDB "Reads/Writes"
      OrderService -> OrderDB "Reads/Writes"
      PaymentService -> PaymentDB "Reads/Writes"
    
  }
---

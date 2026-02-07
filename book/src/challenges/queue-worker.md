---
title: "Email Notification System: Build Async Processor"
summary: "Your app sends too many emails synchronously, causing slow responses! Create an async email processing system with a queue and worker to handle notifications in the background."
difficulty: intermediate
topic: async
estimatedTime: "10-15 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  component = kind "Component"
  queue = kind "Queue"

  App = system "App" {
    API = container "Main API" {
      // TODO: Add component EmailWorker "Email Processor" here
    }
    EmailQueue = queue "RabbitMQ"
  }

  // TODO: Connect App.EmailQueue -> App.API.EmailWorker for async processing
  // TODO: Add external EmailService system with tags and connect App.API.EmailWorker -> EmailService
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: elementExists
    name: EmailWorker
    message: "Create EmailWorker component"
  - type: relationExists
    source: EmailQueue
    target: EmailWorker
    message: "Connect EmailQueue -> EmailWorker"
  - type: elementExists
    name: EmailService
    message: "Add external EmailService"
  - type: relationExists
    source: EmailWorker
    target: EmailService
    message: "Connect EmailWorker -> EmailService"
hints:
  - 'Add component EmailWorker "Email Processor" inside the API container'
  - 'Queues deliver messages to workers: App.EmailQueue -> App.API.EmailWorker "Delivers email job"'
  - 'Add external EmailService as a system with tags ["external"] outside the App system block'
  - 'Worker sends emails: App.API.EmailWorker -> EmailService "Sends email"'
solution: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  component = kind "Component"
  queue = kind "Queue"

  App = system "App" {
    API = container "Main API" {
      EmailWorker = component "Email Processor"
    }
    EmailQueue = queue "RabbitMQ"
  }

  EmailService = system "SendGrid" {
    tags ["external"]
  }

  App.EmailQueue -> App.API.EmailWorker "Delivers email job"
  App.API.EmailWorker -> EmailService "Sends email"
---

---
title: "Email Notification System: Build Async Processor"
summary: "Your app sends too many emails synchronously, causing slow responses! Create an async email processing system with a queue and worker to handle notifications in the background."
difficulty: intermediate
topic: async
estimatedTime: "10-15 min"
initialDsl: |
  architecture "Notification System" {
    system App {
      container API "Main API" {
        // TODO: Add component EmailWorker "Email Processor" here
      }
      queue EmailQueue "RabbitMQ"
    }
    
    // TODO: Connect EmailQueue -> EmailWorker for async processing
    // TODO: Add external EmailService (like SendGrid) and connect EmailWorker -> EmailService
  }
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
  - "Add component EmailWorker \"Email Processor\" inside the API container"
  - "Queues deliver messages to workers: EmailQueue -> EmailWorker \"Delivers email job\""
  - "Add external EmailService \"SendGrid\" outside the system block"
  - "Worker sends emails: EmailWorker -> EmailService \"Sends email\""
solution: |
  architecture "Notification System" {
    system App {
      container API "Main API" {
        component EmailWorker "Email Processor"
      }
      queue EmailQueue "RabbitMQ"
    }
    
    external EmailService "SendGrid"
    
    EmailQueue -> EmailWorker "Delivers email job"
    EmailWorker -> EmailService "Sends email"
  }
---

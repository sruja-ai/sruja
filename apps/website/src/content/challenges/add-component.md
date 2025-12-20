---
title: "Social Feed: Add Recommendation Engine"
summary: "Your social media platform needs personalized content! Add a Recommendation component to the FeedService container that suggests posts based on user interests."
difficulty: beginner
topic: components
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
    User = person "Social Media User"
    
      SocialApp = system  {
        FeedService = container "Feed Service" {
          technology "Python, FastAPI"
          // TODO: Add component Recommendation "Recommendation Engine" here
        }
        UserDB = datastore "MongoDB"
      }
    
      User -> FeedService "Views feed"
      FeedService -> UserDB "Queries user data"
    
  }
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: elementExists
    name: Recommendation
    message: "Create Recommendation component in FeedService"
hints:
  - "Components are defined inside containers using curly braces"
  - "Add component Recommendation \"Recommendation Engine\" inside the FeedService block"
  - "Make sure to open the FeedService container block with { before adding the component"
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
    User = person "Social Media User"
    
      SocialApp = system  {
        FeedService = container "Feed Service" {
          technology "Python, FastAPI"
          Recommendation = component "Recommendation Engine"
        }
        UserDB = datastore "MongoDB"
      }
    
      User -> FeedService "Views feed"
      FeedService -> UserDB "Queries user data"
    
  }
---

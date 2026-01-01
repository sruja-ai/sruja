---
title: "Social Feed: Add Recommendation Engine"
summary: "Your social media platform needs personalized content! Add a Recommendation component to the FeedService container that suggests posts based on user interests."
difficulty: beginner
topic: components
estimatedTime: "5-10 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  component = kind "Component"
  database = kind "Database"

  User = person "Social Media User"

  SocialApp = system "Social App" {
    FeedService = container "Feed Service" {
      technology "Python, FastAPI"
      // TODO: Add component Recommendation "Recommendation Engine" here
    }
    UserDB = database "MongoDB"
  }

  User -> SocialApp.FeedService "Views feed"
  SocialApp.FeedService -> SocialApp.UserDB "Queries user data"
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: elementExists
    name: Recommendation
    message: "Create Recommendation component in FeedService"
hints:
  - "Components are defined inside containers using curly braces"
  - 'Add component Recommendation "Recommendation Engine" inside the FeedService block'
  - "Make sure to open the FeedService container block with { before adding the component"
solution: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  component = kind "Component"
  database = kind "Database"

  User = person "Social Media User"

  SocialApp = system "Social App" {
    FeedService = container "Feed Service" {
      technology "Python, FastAPI"
      Recommendation = component "Recommendation Engine"
    }
    UserDB = database "MongoDB"
  }

  User -> SocialApp.FeedService "Views feed"
  SocialApp.FeedService -> SocialApp.UserDB "Queries user data"
---

---
title: "Code Review: Fix the Ride-Sharing App"
summary: "A junior developer wrote this code for a ride-sharing app, but it has syntax errors. Find and fix all the issues to get it compiling!"
difficulty: beginner
topic: validation
estimatedTime: "5-8 min"
initialDsl: |
  architecture "RideShare" {
    person Rider "App User"
    
    system RideApp {
      container MobileApp "Mobile Application"
      container MatchingService "Driver Matching"  // Syntax issue here
      datastore LocationDB "Location Database"
    }
    
    Rider -> MobileApp "Requests ride"
    MobileApp -> MatchingService Finds driver  // Missing quotes
    MatchingService -> LocationDB "Queries locations"
  }
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: relationExists
    source: MobileApp
    target: MatchingService
    message: "Fix relation MobileApp -> MatchingService"
  - type: relationExists
    source: MatchingService
    target: LocationDB
    message: "Ensure relation MatchingService -> LocationDB exists"
hints:
  - "Check the MatchingService container definition - is it properly closed?"
  - "Look at the relation 'MobileApp -> MatchingService Finds driver' - what's missing?"
  - "All relation labels must be wrapped in double quotes"
  - "Container definitions need proper closing braces"
solution: |
  architecture "RideShare" {
    person Rider "App User"
    
    system RideApp {
      container MobileApp "Mobile Application"
      container MatchingService "Driver Matching"
      datastore LocationDB "Location Database"
    }
    
    Rider -> MobileApp "Requests ride"
    MobileApp -> MatchingService "Finds driver"
    MatchingService -> LocationDB "Queries locations"
  }
---

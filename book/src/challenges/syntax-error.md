---
title: "Code Review: Fix the Ride-Sharing App"
summary: "A junior developer wrote this code for a ride-sharing app, but it has syntax errors. Find and fix all the issues to get it compiling!"
difficulty: beginner
topic: validation
estimatedTime: "5-8 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Rider = person "App User"

  RideApp = system "RideApp" {
    MobileApp = container "Mobile Application"
    MatchingService = container "Driver Matching"
    LocationDB = datastore "Location Database"
  }

  Rider -> RideApp.MobileApp "Requests ride"
  RideApp.MobileApp -> RideApp.MatchingService Finds driver  // Missing quotes
  RideApp.MatchingService -> RideApp.LocationDB "Queries locations"
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
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Rider = person "App User"

  RideApp = system "RideApp" {
    MobileApp = container "Mobile Application"
    MatchingService = container "Driver Matching"
    LocationDB = datastore "Location Database"
  }

  Rider -> RideApp.MobileApp "Requests ride"
  RideApp.MobileApp -> RideApp.MatchingService "Finds driver"
  RideApp.MatchingService -> RideApp.LocationDB "Queries locations"
---

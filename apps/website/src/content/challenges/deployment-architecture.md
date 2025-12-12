---
title: "CDN Architecture: Add Cache Layer"
summary: "Your content delivery network needs a caching layer! Add a Cache datastore and connect it to both the API and CDN edge servers for faster content delivery."
difficulty: intermediate
topic: deployment
estimatedTime: "15-20 min"
initialDsl: |
  architecture "Content Delivery" {
    person Viewer "Content Viewer"
    
    system CDN {
      container EdgeServer "Edge Server"
      container API "Origin API"
      datastore OriginDB "Origin Database"
    }
    
    Viewer -> EdgeServer "Requests content"
    EdgeServer -> API "Fetches from origin"
    API -> OriginDB "Reads content"
    
    // TODO: Add Cache datastore and connect EdgeServer -> Cache and API -> Cache
  }
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: elementExists
    name: Cache
    message: "Add Cache datastore"
  - type: relationExists
    source: EdgeServer
    target: Cache
    message: "Add relation EdgeServer -> Cache"
  - type: relationExists
    source: API
    target: Cache
    message: "Add relation API -> Cache"
hints:
  - "Add datastore Cache \"Redis Cache\" inside the CDN system"
  - "EdgeServer reads from cache: EdgeServer -> Cache \"Reads cached content\""
  - "API writes to cache: API -> Cache \"Writes content\""
  - "Cache sits between EdgeServer and API to reduce origin load"
solution: |
  architecture "Content Delivery" {
    person Viewer "Content Viewer"
    
    system CDN {
      container EdgeServer "Edge Server"
      container API "Origin API"
      datastore Cache "Redis Cache"
      datastore OriginDB "Origin Database"
    }
    
    Viewer -> EdgeServer "Requests content"
    EdgeServer -> API "Fetches from origin"
    EdgeServer -> Cache "Reads cached content"
    API -> OriginDB "Reads content"
    API -> Cache "Writes content"
  }
---

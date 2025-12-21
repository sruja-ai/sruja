---
title: "CDN Architecture: Add Cache Layer"
summary: "Your content delivery network needs a caching layer! Add a Cache datastore and connect it to both the API and CDN edge servers for faster content delivery."
difficulty: intermediate
topic: deployment
estimatedTime: "15-20 min"
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
    Viewer = person "Content Viewer"
    
      CDN = system  {
        EdgeServer = container "Edge Server"
        API = container "Origin API"
        OriginDB = datastore "Origin Database"
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
    Viewer = person "Content Viewer"
    
      CDN = system  {
        EdgeServer = container "Edge Server"
        API = container "Origin API"
        Cache = datastore "Redis Cache"
        OriginDB = datastore "Origin Database"
      }
    
      Viewer -> EdgeServer "Requests content"
      EdgeServer -> API "Fetches from origin"
      EdgeServer -> Cache "Reads cached content"
      API -> OriginDB "Reads content"
      API -> Cache "Writes content"
    
  }
---

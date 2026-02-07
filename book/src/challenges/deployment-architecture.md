---
title: "CDN Architecture: Add Cache Layer"
summary: "Your content delivery network needs a caching layer! Add a Cache datastore and connect it to both the API and CDN edge servers for faster content delivery."
difficulty: intermediate
topic: deployment
estimatedTime: "15-20 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Viewer = person "Content Viewer"

  CDN = system "CDN" {
    EdgeServer = container "Edge Server"
    API = container "Origin API"
    OriginDB = datastore "Origin Database"
  }

  Viewer -> CDN.EdgeServer "Requests content"
  CDN.EdgeServer -> CDN.API "Fetches from origin"
  CDN.API -> CDN.OriginDB "Reads content"

  // TODO: Add Cache datastore and connect EdgeServer -> Cache and API -> Cache
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
  - 'Add datastore Cache "Redis Cache" inside the CDN system'
  - 'EdgeServer reads from cache: EdgeServer -> Cache "Reads cached content"'
  - 'API writes to cache: API -> Cache "Writes content"'
  - "Cache sits between EdgeServer and API to reduce origin load"
solution: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Viewer = person "Content Viewer"

  CDN = system "CDN" {
    EdgeServer = container "Edge Server"
    API = container "Origin API"
    Cache = datastore "Redis Cache"
    OriginDB = datastore "Origin Database"
  }

  Viewer -> CDN.EdgeServer "Requests content"
  CDN.EdgeServer -> CDN.API "Fetches from origin"
  CDN.EdgeServer -> CDN.Cache "Reads cached content"
  CDN.API -> CDN.OriginDB "Reads content"
  CDN.API -> CDN.Cache "Writes content"
---

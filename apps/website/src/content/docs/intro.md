---
title: "Introduction"
weight: 1
---

# Introduction

Sruja is an architecture DSL. Use Studio for interactive modeling.

```sruja
system App "My App" {
  container Web "Web Server"
  datastore DB "Database"
}
person User "User"
User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"
```

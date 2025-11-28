---
title: Playground
description: Try Sruja in your browser
weight: 10
---

# Playground

Use the online playground to experiment with Sruja. Edit the example and click Run to compile to SVG.

{{< playground >}}
architecture "Quick Start" {
  person User "User"
  system Web "Web App" {
    container Frontend "Frontend"
    datastore DB "Database"
  }
  User -> Frontend "Visits"
  Frontend -> DB "Reads/Writes"
}
{{< /playground >}}


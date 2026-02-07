---
title: "Lesson 1: Security by Design"
weight: 1
summary: "Modeling security standards with tags and metadata, validated in CI."
---

# Lesson 1: Security by Design

Security isn't something you "add on" at the end. It must be baked into the architecture.

## The Requirement

**GDPR Article 32**: Personal data must be encrypted.

## Modeling Security Signals

Use tags and metadata to make security posture explicit.

```sruja
import { * } from 'sruja.ai/stdlib'


Shop = system "Shop" {
  UserDB = datastore "User DB" {
    tags ["pii", "encrypted"]
    metadata {
      retention "90d"
    }
  }
}

view index {
include *
}
```

## Validating in CI

Run `sruja validate` in CI to enforce architectural rules (unique IDs, valid references, layering, external boundary checks). Combine with linters to flag missing tags for sensitive resources. This is **Compliance as Code**.

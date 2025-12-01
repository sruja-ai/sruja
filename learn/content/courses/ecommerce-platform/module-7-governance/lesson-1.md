---
title: "Lesson 1: Security by Design (Policies)"
weight: 1
summary: "Enforcing security standards with automated policies."
---

# Lesson 1: Security by Design

Security isn't something you "add on" at the end. It must be baked into the architecture.

## The Requirement
**GDPR Article 32**: Personal data must be encrypted.

## The Policy
We define a global policy that enforces encryption for any database storing PII (Personally Identifiable Information).

```sruja
policy Security "Data Protection Standards" {
    
    rule Encryption "PII Must Be Encrypted" {
        // Pseudo-code: If a component has tag 'pii', it must also have tag 'encrypted'
        check "tags.contains('pii') implies tags.contains('encrypted')"
    }
}
```

## Applying it
Now, if a developer defines a database with PII but forgets encryption:

```sruja
container UserDB {
    tags ["pii"]
    // Missing "encrypted" tag!
}
```

`sruja validate` will fail. This is **Compliance as Code**.

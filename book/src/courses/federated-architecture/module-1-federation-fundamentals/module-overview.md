---
title: "Draft: Module 1 - Federation Fundamentals"
weight: 1
summary: "Understand federation concepts, bundle publishing, and basic composition."
---

# Draft: Module 1 - Federation Fundamentals

**Core concepts, bundle creation, and basic composition.**

## Overview

This module introduces federated architecture and teaches you to create and compose architecture bundles.

## Lessons

### Lesson 1: [What is Federation?](lesson-1.md)
_Core concepts and architecture_

- Why federated architecture?
- Key concepts: bundles, system index, canonical IDs
- Federation architecture diagram
- When to use federation

### Lesson 2: [Publishing Bundles](lesson-2.md)
_Create and distribute repo bundles_

- Bundle structure
- Publishing commands
- Bundle contents deep dive
- Versioning and updates

### Lesson 3: [Basic Composition](lesson-3.md)
_Compose bundles into system index_

- Compose command
- System index structure
- Basic cross-repo view
- Verification and validation

## Learning Outcomes

- ✅ Understand federation concepts
- ✅ Publish a repo bundle
- ✅ Compose bundles into system index
- ✅ View cross-repo architecture

## Prerequisites

- Basic Sruja DSL knowledge
- Understanding of microservices
- Command line familiarity

## Raw Thoughts

### v0.35.0 Features to Cover:
- `sruja publish` command
- `sruja compose` command
- Architecture Index MVP with Federated Registry
- Bundle schema and system index schema

### Key Concepts:
- `repo.bundle.json` - published architecture artifact
- `system.index.json` - composed multi-repo graph
- Canonical IDs: `repo_id::local_id`
- Conflicts array for duplicates
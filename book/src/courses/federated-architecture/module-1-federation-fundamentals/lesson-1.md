---
title: "Lesson 1: What is Federation?"
weight: 1
summary: "Core concepts and architecture of federated multi-repo architecture."
---

# Lesson 1: What is Federation?

## The Problem: Fragmented Architecture

When you have 50+ microservices across dozens of repos:
- Each team has their own view
- No one sees the whole picture
- Cross-repo dependencies are invisible
- Governance is inconsistent

## Solution: Federated Architecture

Federation creates a **single source of truth** across all repos:

```
┌─────────────┐
│   Repo A    │──┐
│ (User Service)│  │
└─────────────┘  │
┌─────────────┐  │    ┌─────────────┐
│   Repo B    │──┼───→│ System Index│
│(Order Service)│  │    └─────────────┘
└─────────────┘  │
┌─────────────┐  │
│   Repo C    │──┘
│(Payment Service)│
└─────────────┘
```

## Key Concepts

### 1. Bundle

A **bundle** is a portable export of a single repo's architecture:

```bash
sruja publish -r . -o repo.bundle.json
```

Contains:
- `repo_id`: Unique identifier
- `baseline_dsl`: Full architecture definition
- `context`: Architecture graph
- `truth_status`: reviewed/drifted/unknown
- `git_commit`: Current commit hash

### 2. System Index

The **system index** is a composed view of all repos:

```bash
sruja compose -i ./bundles -o system.index.json
```

Contains:
- `nodes`: All components with canonical IDs
- `edges`: All relationships
- `conflicts`: Duplicate elements to resolve

### 3. Canonical IDs

Each element gets a **canonical ID** to avoid collisions:

```
repo_id::local_id
```

Example:
- `user-service::UserAPI`
- `order-service::OrderProcessor`
- `payment-service::PaymentGateway`

## Why Federation?

Traditional architecture tools work per-repo, leaving you with fragmented views and no cross-repo visibility. Federation creates a **single source of truth** across all repos:

| Without Federation | With Federation |
|-------------------|-----------------|
| Fragmented views | Single source of truth |
| Invisible dependencies | Full dependency graph |
| Inconsistent governance | Unified policies |
| Manual tracking | Automated updates |

## When to Use Federation

Federation is ideal for:
- ✅ Multi-repo microservices (50+ services)
- ✅ Organizations with multiple teams
- ✅ Acquired companies being integrated
- ✅ Monorepo to multi-repo migrations
- ✅ Platform teams needing visibility

It may be overkill for:
- ⚠️ Small projects (< 10 repos)
- ⚠️ Single-team organizations
- ⚠️ Tightly coupled monoliths

## Federation Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     AI Editor / MCP Server                   │
│                                                              │
│  1. Load system.index.json (or slice)                       │
│  2. Query cross-repo relationships                          │
│  3. Understand blast radius of changes                      │
└─────────────────────────────────────────────────────────────┘
                              ↑
                              │
                    ┌─────────────────┐
                    │   CI/CD Pipeline │
                    │                  │
                    │ sruja publish     │
                    │ sruja compose    │
                    │ sruja drift      │
                    └─────────────────┘
                              ↑
                              │
    ┌───────────────┬─────────┴────────┐
    ↓               ↓                   ↓
┌─────────┐    ┌─────────┐        ┌─────────┐
│  Repo A │    │  Repo B │        │  Repo C │
│ bundle  │    │ bundle  │        │ bundle  │
└─────────┘    └─────────┘        └─────────┘
```

## Hands-On: Explore Federation Concepts

1. **List available federation commands:**
   ```bash
   sruja --help | grep -E "publish|compose|impact"
   ```

2. **Initialize a sample repo for federation:**
   ```bash
   mkdir federation-demo && cd federation-demo
   sruja init --auto
   ```

3. **Publish a bundle from your repo:**
   ```bash
   sruja publish -r . -o repo.bundle.json
   cat repo.bundle.json | jq '.repo_id, .git_commit'
   ```

## Learning Outcomes

- ✅ Understand what federated architecture is and when to use it
- ✅ Explain the key concepts: bundles, system index, and canonical IDs
- ✅ Describe how federation creates a single source of truth across repos
- ✅ Identify use cases where federation provides the most value

## Quiz: Test Your Understanding

### Q1: What is a bundle in the context of Sruja federation?

A) A compressed archive of all repository files
B) A portable export of a single repo's architecture containing DSL, context, and metadata
C) A backup of the entire repository
D) A Docker container image

### Q2: What does a canonical ID prevent?

A) unauthorized access
B) naming collisions across repositories
C) data loss
D) code conflicts

### Q3: Which command is used to create a system index from multiple bundles?

A) `sruja merge`
B) `sruja index`
C) `sruja compose`
D) `sruja aggregate`

## Next Steps

Lesson 2 covers publishing your first bundle.
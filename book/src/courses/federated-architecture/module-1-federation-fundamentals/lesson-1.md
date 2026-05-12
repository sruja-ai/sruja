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
│                     AI Editor / Skill                       │
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
                    │ sruja publish    │
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

## Next Steps

Lesson 2 covers publishing your first bundle.
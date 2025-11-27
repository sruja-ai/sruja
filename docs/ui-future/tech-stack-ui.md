# Tech Stack (Future UI)

Technology stack for future web-based UI and editor.

**Status**: Planned for future UI phase  
**Current Focus**: Go CLI tool (see [Main Documentation](../README.md) and [Go Technology Stack](../implementation/technology-stack.md))

## Overview

This document outlines the planned technology stack for the future web-based UI. The current implementation focuses on Go CLI tooling.

## Frontend (React + TypeScript)

- **React** (latest)
- **TypeScript** (strict mode)
- **Optional: Next.js** (routing, SSR; adopt as needed)
- **TypeScript 5.6+** (strict mode, latest features)
- **React Flow 12.9.3+** (@xyflow/react - diagram editor with latest features)
- **Monaco Editor 0.55.1+** (DSL editor with LSP integration)
- **Monaco Language Client** (LSP client for Monaco)
- **vscode-ws-jsonrpc** (WebSocket JSON-RPC for LSP)
- **Zustand** (local UI state)
- **TanStack Query** (server state)
  - Works seamlessly with Eden Treaty for type-safe API calls
  - Eden Treaty provides the typed client, TanStack Query handles caching/refetching
- **TailwindCSS** (utility-first styling)
- **shadcn/ui** (optional, for consistent UI components)

## Backend (Go)

- **Go HTTP server** for UI APIs (project/model endpoints)
- **Go LSP server** for DSL (JSON-RPC over stdio/TCP; WebSocket bridge if needed)
- **Parser**: participle (Go) with shared AST and serializer
- **Validation Engine**: Go rule system (in-process)
- **Git integration**: go-git

## Core Engine

- **Go parser** (participle) + Go AST
- **JSON Metamodel** (Go structs; `encoding/json`)
- **Transformation engine** (Go; serializer ensures round-trip stability)
- **Layout engine: ELK.js** in frontend (or Dagre as fallback)

## Runtime & Tooling (Frontend)

- **pnpm** or **npm** (workspaces optional)
- **Vitest** (unit & integration testing)
- **Playwright** (E2E testing)
- **@testing-library/react** (component testing)
- **ESLint** + **Prettier** (code quality)

## Collaboration

- **WebSockets** (backend + Next.js client, real-time sync)
- **Redis 7.x** (pub/sub) — future, not MVP
- **Yjs** (optional, for CRDT-based collaboration later)

---

[← Back to UI & Future Features](./README.md) | [Go Technology Stack](../implementation/technology-stack.md) | [Main Documentation](../README.md)

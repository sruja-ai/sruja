# Tech Stack

Complete technology stack for the Architecture DSL Platform.

## Frontend (Next.js 16 + React 19 + TypeScript)

- **Next.js 16** (App Router, Server Actions, Turbopack default, Partial Pre-Rendering)
- **React 19** (Server Components, Suspense, React Compiler, View Transitions API)
- **TypeScript 5.6+** (strict mode, latest features)
- **React Flow 12.9.3+** (@xyflow/react - diagram editor with latest features)
- **Monaco Editor 0.55.1+** (DSL editor with LSP integration)
- **Monaco Language Client** (LSP client for Monaco)
- **vscode-ws-jsonrpc** (WebSocket JSON-RPC for LSP)
- **Zustand 5.x** (local UI state for ephemeral stuff)
- **TanStack Query v5** (server state, React 19 compatible)
  - Works seamlessly with Eden Treaty for type-safe API calls
  - Eden Treaty provides the typed client, TanStack Query handles caching/refetching
- **TailwindCSS 4.x** (latest with CSS variables, container queries)
- **shadcn/ui** (optional, for consistent UI components)

## Backend (Go)

- **Go HTTP server** (REST/JSON endpoints for project/model operations)
- **Go LSP server** (JSON-RPC over stdio/TCP; optional WebSocket bridge)
- **Parser**: participle (Go) with shared AST structs
- **Validation Engine**: Go rule system
- **Git integration**: go-git (pure Go)

## Core Engine

- **Go parser** (participle) and Go AST types
- **JSON Metamodel** (Go structs; `encoding/json`) stored in Git
- **Transformation engine** (Go; serializer ensures round-trip stability)
- **Layout engine: ELK.js** (frontend), Dagre as fallback

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

[← Back to Documentation Index](../README.md)

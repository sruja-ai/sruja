# Sruja Memory

> SQLite + FTS5 indexed memory store for cross-session context lineage.

## Purpose

Provides a SQLite-backed persistent memory store with full-text search (FTS5) for cross-session context lineage. Stores agent learnings, context events, and decision records. Supports search with filtering by element, decision, HITL kind, source, and trust level.

## Responsibilities

- Persist learnings, context events, and decision records in SQLite
- Full-text search (FTS5) across all stored entries
- Timeline queries anchored by ID or timestamp
- Filter by element, decision, HITL kind, source, trust level
- Cross-session memory retrieval

## Dependencies

- **Internal**: Sruja_Agent
- **External**: rusqlite, serde, serde_json, thiserror, chrono, serde_yaml

## Key Types

- `MemoryStore` — Main store interface
- `MemorySearchHit` — Search result
- `MemoryTimelineEntry`, `MemoryTimelineResult` — Timeline query results
- `SearchMemoryOptions` — Search configuration
- `TimelineOptions` — Timeline query configuration

## Code Locations

- `crates/sruja-memory/` — Memory crate
- `src/store.rs` — Main store implementation (939 lines)
- `src/error.rs` — Error types

## Notes

- Native-only (not WASM) — uses rusqlite
- Never writes to `repo.sruja` — only to `.sruja/memory.sqlite`
- The `.sruja/memory.sqlite` database is created on first use

---
*Last updated: 2026-06-06*

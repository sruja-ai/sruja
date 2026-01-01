# DSL Feature Relevance Analysis

**Date:** 2025-01-01
**Status:** Review Complete

---

## Executive Summary

A comprehensive review of the Sruja DSL (`pkg/language`) against the specification and roadmap reveals a solid core but some accumulated technical debt in the form of unused AST structures and redundant features.

**Key Actions Required:**

1. 🗑️ **Delete Dead Code**: Remove unused AST structs (`DeprecationBlock`, `CompatibilityBlock`, `GuaranteesBlock`).
2. ⚠️ **Deprecate Redundancy**: Deprecate `properties` in favor of `metadata`.
3. ❓ **Resolve Experimental**: Decide on `behavior` block (currently undocumented and confusing).
4. 📚 **Clarify Aliases**: Formally document `flow` vs `scenario`.

---

## 1. Core Healthy Features

The following features are fully implemented, documented, and aligned with the C4 model and project roadmap:

- **Structure**: `person`, `system`, `container`, `component`, `database`, `queue`.
- **Relationships**: `->` (with inference).
- **Governance**: `requirement`, `adr`, `policy`, `constraints`, `conventions`.
- **Operations**: `slo` (Availability, Latency, etc.), `scale` (Min/Max/Metric), `deployment`.
- **Views**: `view` (Context, Container, Component, Deployment), `styles`.
- **Meta**: `overview`, `metadata`.

**Verdict:** ✅ **Keep & Maintain**

---

## 2. Redundant Features

### `properties` vs `metadata`

- **Current State**:
  - `metadata { key "value" list ["a", "b"] }`: Supports string values and arrays.
  - `properties { "key": "value" }`: Supports only string values.
- **Analysis**:
  - `properties` appears to be a legacy artifact or Structurizr compatibility layer.
  - `metadata` is strictly more powerful and is the "blessed" way in documentation.
  - Having two ways to do K/V pairs causes confusion.
- **Recommendation**:
  - **Deprecate `properties`**.
  - Update parser to treat `properties` as an alias for `metadata` (if backward compatibility is needed) or remove it entirely in v2.

---

## 3. Ambiguous / Alias Features

### `scenario` vs `story` vs `flow`

- **Current State**:
  - All three share the identical AST structure (Steps).
  - `story` is explicitly documented as an alias for `scenario`.
  - `flow` is documented as "DFD-style data flows".
- **Analysis**:
  - Syntactically identical.
  - Semantically distinct: `scenario` usually implies User/Persona journey; `flow` implies data/system interaction.
- **Recommendation**:
  - ✅ **Keep all**. The semantic distinction aids readability.
  - Ensure documentation clarifies they are structurally the same.

---

## 4. Dead / Zombie Code

The following AST structures exist in `pkg/language/ast_core.go` but are **never referenced** in the parser grammar for `BodyItem` or `TopLevelItem`. They are effectively unreachable code.

- `DeprecationBlock` (`deprecation { ... }`)
- `CompatibilityBlock` (`compatibility { ... }`)
- `GuaranteesBlock` (`guarantees { ... }`)

**Verdict:** 🗑️ **DELETE IMMEDIATELY**. They add noise to the codebase and confuse contributors.

---

## 5. Undocumented / Experimental

### `behavior` block

- **Current State**:
  - Defined in `ast_elements.go` inside `ComponentItem`.
  - Structure: `behavior { key "value" }` (K/V pairs).
  - **NOT** documented in `LANGUAGE_SPECIFICATION.md`.
- **Analysis**:
  - Confusing naming. In C4, "behavior" usually refers to dynamic diagrams (scenarios/flows).
  - This block seems to be another generic K/V store, redundant with `metadata`.
- **Recommendation**:
  - 🗑️ **Remove**. It overlaps with `metadata` and conflicts with the semantic meaning of "behavior" in architecture modeling.

---

## Implementation Plan

1.  **Cleanup PR**:
    - Remove `DeprecationBlock`, `CompatibilityBlock`, `GuaranteesBlock` structs.
    - Remove `BehaviorBlock` from `ComponentItem` and `ast_core.go`.
2.  **Deprecation PR**:
    - Add warning to `PropertiesBlock` usage in validator: "Use `metadata` instead."
3.  **Documentation update**:
    - Ensure `Start Simple` guide only uses `metadata`.

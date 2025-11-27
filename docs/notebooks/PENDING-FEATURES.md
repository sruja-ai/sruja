# Pending Features for Sruja Kernel

[← Back to Notebooks Index](./README.md)

## Overview

The Sruja Kernel core is **complete**, but several features remain to be implemented for full notebook functionality.

## 🔄 Pending Features

### 1. Query Engine Integration (SrujaQL)

**Status:** ✅ **COMPLETE**

**Current State:**
- ✅ Query engine exists in `pkg/query/engine.go`
- ✅ Query parser exists in `pkg/query/parser.go`
- ✅ Integrated into kernel
- ✅ Query cells execute queries successfully
- ✅ Queries work over ArchitectureStore (IR)
- ✅ Query results formatted as JSON and text
- ✅ Added IR-based query methods (`filterSystemsFromModel`, etc.)

**Files Updated:**
- `pkg/kernel/kernel.go` - `executeQueryCell()` method implemented
- `pkg/query/engine.go` - Added IR support (`NewEngineFromModel`, `SetModel`, `ExecuteFromModel`)
- `pkg/query/engine_ir.go` - IR-based filter methods
- `pkg/query/parser.go` - Fixed Float/Int token definitions

**Estimated Effort:** ✅ Complete (2-3 hours)

---

### 2. Diagram Generation

**Status:** ✅ **COMPLETE**

**Current State:**
- ✅ Mermaid compiler exists in `pkg/compiler/mermaid.go`
- ✅ D2 compiler exists in `pkg/compiler/d2.go`
- ✅ Integrated into kernel
- ✅ Diagram cells generate diagrams
- ✅ Diagram command parsing implemented
- ✅ Generates diagrams from ArchitectureStore (IR)
- ✅ Outputs Mermaid and D2 formats
- ✅ Supports filtering by system/container/component
- ✅ Added `CompileFromModel()` methods to compilers

**What's Remaining:**
- Support multiple diagram types (future enhancement):
  - Entity relationship diagrams
  - Event flow diagrams
  - Lifecycle FSMs
- SVG output (optional, can be generated from Mermaid/D2)

**Files Updated:**
- `pkg/kernel/kernel.go` - `executeDiagramCell()` method implemented
- `pkg/kernel/diagram_parser.go` - Diagram command parsing
- `pkg/compiler/mermaid.go` - Added `CompileFromModel()` method
- `pkg/compiler/d2.go` - Added `CompileFromModel()` method

**Estimated Effort:** ✅ Complete (2-3 hours)

---

### 3. Enhanced Validation Cells

**Status:** ✅ **COMPLETE**

**Current State:**
- ✅ Validation engine integrated
- ✅ Diagnostics collection works
- ✅ Validation command parsing implemented
- ✅ Can validate specific elements (system, container, component)
- ✅ Can validate specific rules (basic support)
- ✅ Better validation output formatting
- ✅ Stores last parsed program for validation

**Features:**
- ✅ Parse validation commands:
  - `validate` or `validate all` - Validate entire architecture
  - `validate system Billing` - Validate specific system
  - `validate container BillingAPI` - Validate specific container
  - `validate component PaymentService` - Validate specific component
  - `validate rule UniqueIDs` - Validate using specific rule
- ✅ Selective validation filtering
- ✅ Formatted text and JSON output
- ✅ Error/warning counts and summaries

**Files Updated:**
- `pkg/kernel/kernel.go` - `executeValidationCell()` method implemented
- `pkg/kernel/validation_parser.go` - Validation command parser
- `pkg/kernel/kernel.go` - Added `lastProgram` field to store AST for validation

**Estimated Effort:** ✅ Complete (2-3 hours)

---

### 4. AI Cell Integration

**Status:** ❌ Not implemented

**Current State:**
- ✅ MCP integration exists (`pkg/mcp/`)
- ✅ MCP tools defined (see `docs/notebooks/mcp-tools.md`)
- ❌ AI cell execution not implemented
- ❌ No AI intent parsing
- ❌ No MCP tool integration in kernel

**What's Needed:**
- Parse AI cell commands:
  - `ai refine system Billing for reliability`
  - `ai suggest improvements`
  - `ai fix violations`
- Integrate with MCP tools
- Generate architecture patches from AI suggestions
- Apply patches with user approval

**Files to Create:**
- `pkg/kernel/ai.go` - AI cell execution
- `pkg/kernel/ai_parser.go` - AI command parsing

**Files to Update:**
- `pkg/kernel/kernel.go` - `executeAICell()` method

**Estimated Effort:** 6-8 hours

---

### 5. Event Simulation Engine

**Status:** ✅ **COMPLETED**

**Current State:**
- ✅ Event model exists in AST
- ✅ Lifecycle FSMs defined in DSL
- ✅ Simulation engine implemented
- ✅ Event lifecycle simulation working
- ✅ FSM extraction from entities
- ✅ Event effect registration
- ✅ State transition validation
- ✅ Simulation command parsing
- ✅ Integration with kernel

**Files Created:**
- `pkg/kernel/simulation.go` - Event simulation engine
- `pkg/kernel/simulation_parser.go` - Simulation command parser
- `pkg/kernel/simulation_test.go` - Comprehensive tests

**Features:**
- Builds FSMs from entity lifecycle definitions
- Registers event lifecycle effects
- Simulates event sequences
- Validates state transitions
- Detects invalid transitions
- Outputs formatted simulation results (text + JSON)
- Integrated with kernel cell execution
- Jupyter protocol support

---

### 6. Enhanced Variant Diff/Merge

**Status:** ✅ **COMPLETED**

**Current State:**
- ✅ Variant creation and storage works
- ✅ Proper diff algorithm implemented
- ✅ Conflict detection working
- ✅ Three-way merge support
- ✅ Human-readable merge explanations
- ✅ Integration with kernel and magic commands

**What Was Implemented:**
- ✅ Diff engine for comparing models (`pkg/kernel/diff.go`)
- ✅ Proper variant diff computation (variant vs base)
- ✅ Conflict detection (base, variant, current)
- ✅ Three-way merge algorithm
- ✅ Human-readable merge explanations
- ✅ Magic command support (`%variant merge`)

**Files Created/Updated:**
- `pkg/kernel/diff.go` - Complete diff engine
- `pkg/kernel/variant.go` - Enhanced `ComputeVariantDiff()` and `MergeVariant()`
- `pkg/kernel/diff_test.go` - Diff engine tests
- `pkg/kernel/variant_merge_test.go` - Merge functionality tests
- `pkg/kernel/kernel.go` - Magic command integration

---

### 7. WASM Compilation Support

**Status:** ✅ **COMPLETE**

**Implementation Date:** Today

**Current State:**
- ✅ Kernel compiled to WASM successfully (~5MB)
- ✅ WASM build configuration complete
- ✅ All kernel functions exported to JS
- ✅ JS bridge/wrapper implemented
- ✅ Example HTML page created

**What's Implemented:**
- ✅ WASM entry point with JS exports
- ✅ JavaScript loader class
- ✅ Build automation (Makefile.wasm)
- ✅ Example usage page
- ✅ Complete documentation

**Build:**
```bash
make -f Makefile.wasm wasm
```

**Files Created:**
- `cmd/sruja-kernel-wasm/main.go` - WASM entry point
- `pkg/kernel/wasm/wasm_loader.js` - JavaScript loader
- `pkg/kernel/wasm/example.html` - Example page
- `pkg/kernel/wasm/README.md` - Documentation
- `Makefile.wasm` - Build automation

**WASM Size:** ~5MB (valid WebAssembly binary)

**Features:**
- ✅ All kernel operations exported
- ✅ Snapshot and variant operations
- ✅ LSP features (autocomplete, inspect)
- ✅ Error handling and initialization

**Estimated Effort:** ✅ Complete (4-6 hours)

---

### 8. Jupyter Kernel Protocol Integration

**Status:** ✅ **COMPLETE** (stdio transport)

**Implementation Date:** Today

**Current State:**
- ✅ Kernel execution works
- ✅ Jupyter protocol implementation complete
- ✅ stdio transport support (works with VSCode/JupyterLite)
- ✅ Kernel messaging protocol implemented
- ⏳ ZeroMQ transport (future enhancement)
- ⏳ WebSocket transport (future enhancement)

**What's Implemented:**
- ✅ Jupyter Kernel Messaging Protocol structures
- ✅ Message routing (execute_request, kernel_info_request, etc.)
- ✅ Output formatting as Jupyter display_data messages
- ✅ Cell type auto-detection
- ✅ Kernel spec file (kernel.json)
- ✅ CLI entry point (cmd/sruja-kernel)

**Files Created:**
- `pkg/kernel/jupyter/protocol.go` - Jupyter protocol types
- `pkg/kernel/jupyter/server.go` - Kernel server implementation
- `pkg/kernel/jupyter/kernel.json` - Kernel spec
- `pkg/kernel/jupyter/README.md` - Documentation
- `cmd/sruja-kernel/main.go` - CLI entry point

**Supported Message Types:**
- ✅ `kernel_info_request` - Kernel capabilities
- ✅ `execute_request` - Code execution
- ✅ `complete_request` - Autocomplete
- ✅ `inspect_request` - Hover/inspection
- ✅ `is_complete_request` - Syntax checking
- ✅ `shutdown_request` - Shutdown/restart

**Usage:**
```bash
# Run kernel directly
go run cmd/sruja-kernel/main.go

# Install as Jupyter kernel
jupyter kernelspec install pkg/kernel/jupyter/kernel.json --name sruja
```

**Future Enhancements:**
- ZeroMQ transport for classic JupyterLab
- WebSocket transport for web clients
- Enhanced completion using LSP
- Enhanced inspection using symbol table

**Estimated Effort:** ✅ Complete (8-12 hours)

---

### 9. Diagram Command Parsing

**Status:** ❌ Not implemented

**What's Needed:**
- Parse commands like:
  - `diagram system Billing`
  - `diagram lifecycle Payment`
  - `diagram event-flow PaymentCompleted`
  - `diagram dependencies of Billing`
- Extract diagram type and target
- Route to appropriate diagram generator

**Files to Create:**
- `pkg/kernel/diagram_parser.go` - Diagram command parser

**Estimated Effort:** 2-3 hours

---

### 10. Magic Commands Support

**Status:** ✅ **COMPLETE**

**Implementation Date:** Today

**Current State:**
- ✅ Magic command parsing implemented
- ✅ All major magic commands supported
- ✅ Routes to appropriate kernel operations
- ✅ Works in any cell type (except markdown)

**Features:**
- ✅ `%ir` - Show current IR (JSON + text)
- ✅ `%snapshot <name> [description]` - Create snapshot
- ✅ `%snapshot list` - List all snapshots
- ✅ `%snapshot load <name>` - Load snapshot
- ✅ `%snapshot delete <name>` - Delete snapshot
- ✅ `%variant list` - List all variants
- ✅ `%variant create <name> [base] [description]` - Create variant
- ✅ `%variant apply <name>` - Apply variant
- ✅ `%validate` - Validate architecture (delegates to validation cell)
- ✅ `%reset` - Reset kernel state

**Files Created:**
- `pkg/kernel/magic.go` - Magic command parser and execution
- `pkg/kernel/magic_test.go` - Comprehensive tests

**Usage Examples:**
```
%ir                              # Show current IR
%snapshot iteration-3            # Create snapshot
%snapshot list                   # List all snapshots
%snapshot load iteration-3       # Load snapshot
%variant list                    # List variants
%variant create async-payments base-snap  # Create variant
%variant apply async-payments    # Apply variant
%validate                        # Validate architecture
%reset                           # Reset kernel
```

**Estimated Effort:** ✅ Complete (2-3 hours)

---

## Priority Ranking

### High Priority (Core Notebook Features)

1. ✅ **Query Engine Integration** - **COMPLETE**
2. ✅ **Diagram Generation** - **COMPLETE**
3. ✅ **Enhanced Validation Cells** - **COMPLETE**
4. ✅ **Magic Commands** - **COMPLETE**

### Medium Priority (Enhanced Features)

5. **AI Cell Integration** - Powerful but optional
6. **Event Simulation** - Useful for event-driven architectures
7. **Enhanced Variant Diff/Merge** - Better variant management

### Lower Priority (Infrastructure)

8. ✅ **WASM Compilation** - **COMPLETE**
9. ✅ **Jupyter Protocol** - **COMPLETE**
10. ✅ **Diagram Command Parsing** - **COMPLETE** (implemented with diagram generation)

## ✅ Quick Wins - COMPLETED

These quick wins have all been implemented:

1. ✅ **Query Engine Integration** - **COMPLETE**
   - Query engine integrated with kernel
   - IR-based query support added

2. ✅ **Magic Commands** - **COMPLETE**
   - Command parsing implemented
   - Routes to kernel methods working

3. ✅ **Enhanced Validation** - **COMPLETE**
   - Validation commands parsed
   - Enhanced output formatting done

## Estimated Total Effort

- **Quick Wins:** 6-10 hours
- **Medium Priority:** 16-24 hours
- **Lower Priority:** 12-18 hours

**Total:** ~34-52 hours of development

## ✅ Next Steps - COMPLETED

All high-priority quick wins are now complete:

1. ✅ **Query Engine Integration** - **DONE**
2. ✅ **Diagram Generation** - **DONE**
3. ✅ **Magic Commands** - **DONE**
4. ✅ **Enhanced Validation** - **DONE**

## Remaining Next Steps

The following features remain for enhanced notebook functionality:

1. **AI Cell Integration** - Integrate with AI/MCP layer (Recommended next)
2. **Event Simulation Engine** - Simulate event-driven lifecycles
3. **Enhanced Variant Diff/Merge** - Better conflict detection

**Infrastructure Complete:**
- ✅ **WASM Compilation** - Browser-based execution
- ✅ **Jupyter Protocol** - Full Jupyter integration

See [Next Steps](./NEXT-STEPS.md) for detailed recommendations and implementation guide.

## Files Reference

### Existing Code to Leverage

- `pkg/query/engine.go` - Query engine (ready to integrate)
- `pkg/query/parser.go` - Query parser (ready)
- `pkg/compiler/mermaid.go` - Mermaid compiler (ready)
- `pkg/compiler/d2.go` - D2 compiler (ready)
- `pkg/mcp/server.go` - MCP server (for AI integration)

### Files Needing Updates

- `pkg/kernel/kernel.go` - Add missing cell type handlers
- `pkg/kernel/README.md` - Update status

### Files to Create

- `pkg/kernel/diagram.go` - Diagram generation logic
- `pkg/kernel/ai.go` - AI cell execution
- `pkg/kernel/simulation.go` - Event simulation
- `pkg/kernel/magic.go` - Magic commands
- `pkg/kernel/diagram_parser.go` - Diagram command parsing
- `pkg/kernel/validation_parser.go` - Validation command parsing
- `pkg/kernel/diff.go` - Enhanced diff engine


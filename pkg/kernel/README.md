# Sruja Architecture Kernel

The Architecture Kernel is the execution engine behind Sruja Notebooks. It maintains stateful architecture model and provides incremental execution, validation, and diagram generation.

## Overview

The kernel provides:

- ✅ **Stateful Architecture Store** - Maintains architecture IR in memory
- ✅ **Incremental Cell Execution** - Execute DSL cells one at a time
- ✅ **Symbol Table** - For LSP features (autocomplete, go-to-definition, etc.)
- ✅ **Symbol Extraction** - Automatically extracts symbols from AST
- ✅ **Enhanced Diagnostics** - Comprehensive error and warning handling
- ✅ **LSP Features** - Autocomplete, hover, go-to-definition, references
- ✅ **Validation** - Runs validators on architecture
- ✅ **IR Management** - Export/import architecture state
- ✅ **Snapshot Management** - Save and load architecture state
- ✅ **Variant Management** - Create and merge experimental variants

## Core Components

### ArchitectureStore

Maintains the canonical architecture model (IR) in memory:

```go
store := kernel.NewArchitectureStore()
model := store.GetModel()
store.UpdateModel(newModel)
irJSON, _ := store.ToJSON()
```

### Kernel

Main execution engine:

```go
k, _ := kernel.NewKernel()
result, _ := k.ExecuteCell("cell-1", kernel.CellTypeDSL, "system Billing { ... }")
irJSON, _ := k.ExportIR()
```

### SymbolTable

Registry of all defined symbols for LSP features:

```go
symbols := kernel.NewSymbolTable()
symbols.AddSymbol("Payment", kernel.SymbolKindEntity, "Payment", location)
entry, _ := symbols.GetSymbol("Payment")
```

Symbols are automatically extracted from parsed AST when cells execute.

### LSP Features

```go
// Get autocomplete suggestions
completions := k.GetCompletions("sys", 3)

// Get hover information
hover, _ := k.GetHover("Payment")

// Get definition location
loc, _ := k.GetDefinition("Payment")

// Get references
refs := k.GetReferences("Payment")
```

## Usage Example

```go
// Create kernel
k, err := kernel.NewKernel()
if err != nil {
    log.Fatal(err)
}

// Execute DSL cell
result, err := k.ExecuteCell(
    kernel.CellID("cell-1"),
    kernel.CellTypeDSL,
    `system Billing {
        container BillingAPI {
            component PaymentService {
                api POST /payments
            }
        }
    }`,
)

if err != nil {
    log.Fatal(err)
}

// Check results
if result.Success {
    fmt.Println("Architecture updated!")
    for _, output := range result.Outputs {
        fmt.Printf("Output: %s\n", output.OutputType)
    }
} else {
    for _, diag := range result.Diagnostics {
        fmt.Printf("%s: %s\n", diag.Severity, diag.Message)
    }
}

// Export IR
irJSON, _ := k.ExportIR()
fmt.Println(string(irJSON))
```

## Cell Types

- `CellTypeDSL` - Architecture DSL code
- `CellTypeQuery` - SrujaQL queries ✅
- `CellTypeDiagram` - Diagram generation ✅
- `CellTypeValidation` - Validation execution ✅
- `CellTypeSimulation` - Event simulation ✅
- `CellTypeAI` - AI-assisted refinement (deferred - using IDE AI instead)
- `CellTypeMarkdown` - Markdown documentation (no-op)

## Snapshot & Variant Operations

```go
// Create snapshot
snapshot, _ := k.CreateSnapshot("iteration-5", "Before async changes")

// Create variant
variant, _ := k.CreateVariant("async-payments", "iteration-5", "Async payment variant")

// Apply variant
k.ApplyVariant("async-payments")

// Merge variant
k.MergeVariant("async-payments")

// List snapshots/variants
snapshots := k.ListSnapshots()
variants := k.ListVariants()
```

## Status

**Core Features Implemented** - The kernel is functional for basic notebook operations. Remaining work:

- [ ] Query engine integration (SrujaQL)
- [ ] Diagram generation
- [ ] Enhanced validation
- [ ] Symbol table population from AST
- [ ] AI cell integration
- [ ] Improved variant diff computation

## Next Steps

See [notebooks documentation](../../docs/notebooks/kernel.md) for complete design details.


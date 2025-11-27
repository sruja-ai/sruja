//go:build js && wasm
// +build js,wasm

// apps/kernel-wasm/main.go
// WebAssembly entry point for Sruja Architecture Kernel

package main

import (
	"encoding/json"
	"fmt"
	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/kernel"
)

var globalKernel *kernel.Kernel

// init initializes the kernel when WASM module loads.
func init() {
	k, err := kernel.NewKernel()
	if err != nil {
		// Log error but don't fail - kernel will be initialized lazily
		js.Global().Get("console").Call("error", fmt.Sprintf("Failed to create kernel: %v", err))
		return
	}
	globalKernel = k
}

func main() {
	// Register WASM exports
	js.Global().Set("srujaKernel", js.ValueOf(map[string]interface{}{
		"init":           js.FuncOf(initKernel),
		"execute":        js.FuncOf(executeCell),
		"query":          js.FuncOf(executeQuery),
		"diagram":        js.FuncOf(generateDiagram),
		"validate":       js.FuncOf(validateCode),
		"exportIR":       js.FuncOf(exportIR),
		"importIR":       js.FuncOf(importIR),
		"reset":          js.FuncOf(resetKernel),
		"getDiagnostics": js.FuncOf(getDiagnostics),
		"autocomplete":   js.FuncOf(autocomplete),
		"inspect":        js.FuncOf(inspect),
		"snapshot":       js.FuncOf(createSnapshot),
		"loadSnapshot":   js.FuncOf(loadSnapshot),
		"listSnapshots":  js.FuncOf(listSnapshots),
		"createVariant":  js.FuncOf(createVariant),
		"applyVariant":   js.FuncOf(applyVariant),
		"listVariants":   js.FuncOf(listVariants),
	}))

	// Initialize kernel
	ensureKernel()

	// Keep alive
	select {}
}

// ensureKernel ensures the kernel is initialized.
func ensureKernel() *kernel.Kernel {
	if globalKernel == nil {
		k, err := kernel.NewKernel()
		if err != nil {
			panic(fmt.Sprintf("Failed to create kernel: %v", err))
		}
		globalKernel = k
	}
	return globalKernel
}

// initKernel initializes the kernel (can be called multiple times safely).
func initKernel(this js.Value, args []js.Value) interface{} {
	ensureKernel()
	return map[string]interface{}{
		"success": true,
		"message": "Kernel initialized",
	}
}

// executeCell executes a notebook cell.
// args: [code string, cellId string, cellType string]
func executeCell(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 2 {
		return map[string]interface{}{
			"success": false,
			"error":   "execute requires code and cellId arguments",
		}
	}

	code := args[0].String()
	cellID := kernel.CellID(args[1].String())

	// Detect cell type if not provided
	cellType := kernel.CellTypeDSL
	if len(args) >= 3 && !args[2].IsNull() && !args[2].IsUndefined() {
		cellType = kernel.CellType(args[2].String())
	}

	result, err := k.ExecuteCell(cellID, cellType, code)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return resultToJS(result)
}

// executeQuery executes a SrujaQL query.
// args: [query string]
func executeQuery(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "query requires a query string argument",
		}
	}

	queryStr := args[0].String()
	result, err := k.ExecuteCell("query-cell", kernel.CellTypeQuery, queryStr)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return resultToJS(result)
}

// generateDiagram generates a diagram.
// args: [target string, format string]
func generateDiagram(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "diagram requires a target argument",
		}
	}

	target := args[0].String()
	format := "mermaid"
	if len(args) >= 2 && !args[1].IsNull() && !args[1].IsUndefined() {
		format = args[1].String()
	}

	cmd := fmt.Sprintf("diagram %s %s", format, target)
	result, err := k.ExecuteCell("diagram-cell", kernel.CellTypeDiagram, cmd)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return resultToJS(result)
}

// validateCode validates architecture code.
// args: [code string]
func validateCode(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "validate requires code argument",
		}
	}

	code := args[0].String()
	result, err := k.ExecuteCell("validate-cell", kernel.CellTypeValidation, code)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return resultToJS(result)
}

// exportIR exports the architecture IR as JSON.
func exportIR(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	irJSON, err := k.ExportIR()
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"ir":      string(irJSON),
	}
}

// importIR imports architecture IR from JSON.
// args: [irJSON string]
func importIR(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "importIR requires irJSON argument",
		}
	}

	irJSON := args[0].String()
	err := k.ImportIR([]byte(irJSON))
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"message": "IR imported successfully",
	}
}

// resetKernel resets the kernel state.
func resetKernel(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()
	k.Reset()

	return map[string]interface{}{
		"success": true,
		"message": "Kernel reset",
	}
}

// getDiagnostics retrieves diagnostics for a cell.
// args: [cellId string]
func getDiagnostics(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "getDiagnostics requires cellId argument",
		}
	}

	cellID := kernel.CellID(args[0].String())
	result, ok := k.GetCellHistory(cellID)
	if !ok {
		return map[string]interface{}{
			"success": false,
			"error":   "Cell not found",
		}
	}

	diagJSON, err := json.Marshal(result.Diagnostics)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success":     true,
		"diagnostics": string(diagJSON),
	}
}

// autocomplete provides completion suggestions.
// args: [code string, cursorPos int]
func autocomplete(this js.Value, args []js.Value) interface{} {
	// Placeholder - would need LSP integration
	return map[string]interface{}{
		"success": true,
		"matches": []interface{}{},
	}
}

// inspect provides hover/inspection information.
// args: [code string, cursorPos int]
func inspect(this js.Value, args []js.Value) interface{} {
	// Placeholder - would need LSP integration
	return map[string]interface{}{
		"success": true,
		"found":   false,
	}
}

// createSnapshot creates a snapshot.
// args: [name string, description string]
func createSnapshot(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "createSnapshot requires name argument",
		}
	}

	name := args[0].String()
	description := ""
	if len(args) >= 2 && !args[1].IsNull() && !args[1].IsUndefined() {
		description = args[1].String()
	}

	snap, err := k.CreateSnapshot(name, description)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"snapshot": map[string]interface{}{
			"name":        snap.Name,
			"description": snap.Description,
			"timestamp":   snap.Timestamp.Format("2006-01-02T15:04:05Z"),
		},
	}
}

// loadSnapshot loads a snapshot.
// args: [name string]
func loadSnapshot(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "loadSnapshot requires name argument",
		}
	}

	name := args[0].String()
	err := k.LoadSnapshot(name)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"message": fmt.Sprintf("Snapshot '%s' loaded", name),
	}
}

// listSnapshots lists all snapshots.
func listSnapshots(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	snapshots := k.ListSnapshots()
	snaps := make([]interface{}, len(snapshots))
	for i, snap := range snapshots {
		snaps[i] = map[string]interface{}{
			"name":        snap.Name,
			"description": snap.Description,
			"timestamp":   snap.Timestamp.Format("2006-01-02T15:04:05Z"),
		}
	}

	return map[string]interface{}{
		"success":   true,
		"snapshots": snaps,
	}
}

// createVariant creates a variant.
// args: [name string, baseSnapshot string, description string]
func createVariant(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "createVariant requires name argument",
		}
	}

	name := args[0].String()
	base := ""
	if len(args) >= 2 && !args[1].IsNull() && !args[1].IsUndefined() {
		base = args[1].String()
	}
	description := ""
	if len(args) >= 3 && !args[2].IsNull() && !args[2].IsUndefined() {
		description = args[2].String()
	}

	variant, err := k.CreateVariant(name, base, description)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"variant": map[string]interface{}{
			"name":        variant.Name,
			"base":        variant.Base,
			"description": variant.Description,
		},
	}
}

// applyVariant applies a variant.
// args: [name string]
func applyVariant(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	if len(args) < 1 {
		return map[string]interface{}{
			"success": false,
			"error":   "applyVariant requires name argument",
		}
	}

	name := args[0].String()
	err := k.ApplyVariant(name)
	if err != nil {
		return map[string]interface{}{
			"success": false,
			"error":   err.Error(),
		}
	}

	return map[string]interface{}{
		"success": true,
		"message": fmt.Sprintf("Variant '%s' applied", name),
	}
}

// listVariants lists all variants.
func listVariants(this js.Value, args []js.Value) interface{} {
	k := ensureKernel()

	variants := k.ListVariants()
	vars := make([]interface{}, len(variants))
	for i, variant := range variants {
		vars[i] = map[string]interface{}{
			"name":        variant.Name,
			"base":        variant.Base,
			"description": variant.Description,
		}
	}

	return map[string]interface{}{
		"success":  true,
		"variants": vars,
	}
}

// resultToJS converts ExecutionResult to JavaScript object.
func resultToJS(result *kernel.ExecutionResult) map[string]interface{} {
	outputs := make([]interface{}, len(result.Outputs))
	for i, output := range result.Outputs {
		outputs[i] = map[string]interface{}{
			"type":      output.OutputType,
			"data":      output.Data,
			"cellId":    string(output.CellID),
			"timestamp": output.Timestamp.Format("2006-01-02T15:04:05Z"),
		}
	}

	diagnostics := make([]interface{}, len(result.Diagnostics))
	for i, diag := range result.Diagnostics {
		diagnostics[i] = map[string]interface{}{
			"severity":  diag.Severity,
			"message":   diag.Message,
			"elementId": diag.ElementID,
		}
	}

	return map[string]interface{}{
		"success":     result.Success,
		"irChanged":   result.IRChanged,
		"outputs":     outputs,
		"diagnostics": diagnostics,
		"error":       result.Error,
		"timestamp":   result.Timestamp.Format("2006-01-02T15:04:05Z"),
	}
}

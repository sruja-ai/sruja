//go:build js && wasm

// Package main provides the WebAssembly (WASM) entry point for Sruja.
//
// This package exposes Sruja's functionality to JavaScript/TypeScript code running
// in the browser. Functions are registered with the JavaScript global object and
// can be called from JavaScript code.
package main

import (
	"encoding/json"
	"fmt"
	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
)

const (
	// defaultFilename is the default filename used when none is provided
	defaultFilename = "input.sruja"
	// minArgsRequired is the minimum number of arguments required for most functions
	minArgsRequired = 1
)

// main initializes the WASM module and registers all exported functions.
//
// Registers Go functions with the JavaScript global object. The channel blocks
// forever to keep the WASM module alive (WASM modules are garbage collected when
// the goroutine exits).
func main() {
	c := make(chan struct{})

	parseDslFn := js.FuncOf(parseDsl)
	jsonToDslFn := js.FuncOf(jsonToDsl)
	js.Global().Set("sruja_parse_dsl", parseDslFn)
	js.Global().Set("sruja_json_to_dsl", jsonToDslFn)

	js.Global().Set("sruja_get_diagnostics", js.FuncOf(getDiagnostics))
	js.Global().Set("sruja_get_symbols", js.FuncOf(getSymbols))
	js.Global().Set("sruja_hover", js.FuncOf(hover))
	js.Global().Set("sruja_completion", js.FuncOf(completion))
	js.Global().Set("sruja_go_to_definition", js.FuncOf(goToDefinition))
	js.Global().Set("sruja_find_references", js.FuncOf(findReferences))
	js.Global().Set("sruja_rename", js.FuncOf(rename))
	js.Global().Set("sruja_format", js.FuncOf(format))
	js.Global().Set("sruja_code_actions", js.FuncOf(codeActions))
	js.Global().Set("sruja_semantic_tokens", js.FuncOf(semanticTokens))
	js.Global().Set("sruja_document_links", js.FuncOf(documentLinks))
	js.Global().Set("sruja_folding_ranges", js.FuncOf(foldingRanges))
	js.Global().Set("sruja_score", js.FuncOf(score))
	js.Global().Set("sruja_dsl_to_mermaid", js.FuncOf(dslToMermaid))
	js.Global().Set("sruja_dsl_to_markdown", js.FuncOf(dslToMarkdown))
	js.Global().Set("sruja_dsl_to_likec4", js.FuncOf(dslToLikeC4))
	js.Global().Set("sruja_dsl_to_dot", js.FuncOf(dslToDot))

	// Keep function references to prevent dead-code elimination
	_ = parseDslFn
	_ = jsonToDslFn

	<-c
}

// parseArgs extracts and validates arguments from JavaScript function calls.
// Returns the DSL text, filename (defaults to "input.sruja"), and any validation error.
func parseArgs(args []js.Value) (input, filename string, err error) {
	if len(args) < minArgsRequired {
		return "", "", fmt.Errorf("invalid arguments: expected at least %d argument(s), got %d", minArgsRequired, len(args))
	}

	input = args[0].String()
	if input == "" {
		return "", "", fmt.Errorf("invalid arguments: input cannot be empty")
	}

	filename = defaultFilename
	if len(args) >= 2 {
		if fn := args[1].String(); fn != "" {
			filename = fn
		}
	}

	return input, filename, nil
}

// parseDsl parses DSL text and exports it to JSON format.
// Arguments: args[0] is DSL text (required), args[1] is filename (optional).
func parseDsl(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = result(false, "", fmt.Sprint(r))
		}
	}()

	input, filename, err := parseArgs(args)
	if err != nil {
		return result(false, "", err.Error())
	}

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", err.Error())
	}

	exporter := jexport.NewLikeC4Exporter()
	jsonStr, err := exporter.Export(program)
	if err != nil {
		return result(false, "", fmt.Errorf("export failed: %w", err).Error())
	}

	return result(true, jsonStr, "")
}

func jsonToDsl(this js.Value, args []js.Value) interface{} {
	_, _, err := parseArgs(args)
	if err != nil {
		return result(false, "", err.Error())
	}
	return result(false, "", "JSON to DSL conversion is no longer supported - Architecture struct removed. Use LikeC4 format instead")
}

func dslToMermaid(this js.Value, args []js.Value) interface{} {
	input, filename, err := parseArgs(args)
	if err != nil {
		return result(false, "", err.Error())
	}

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", fmt.Errorf("parse failed: %w", err).Error())
	}

	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
	}

	// Check if Model.Items is empty
	if len(program.Model.Items) == 0 {
		return result(false, "", "model block is empty - no items found in Model.Items")
	}

	exporter := mermaid.NewExporter(mermaid.DefaultConfig())
	output := exporter.Export(program)

	// Validate that we got a non-empty result
	if output == "" {
		// Provide diagnostic information about what was found
		modelItemCount := len(program.Model.Items)
		elementDefCount := 0
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				elementDefCount++
			}
		}
		errorMsg := fmt.Sprintf("mermaid exporter returned empty output - no elements found in model (Model.Items: %d, ElementDef items: %d)", modelItemCount, elementDefCount)
		return result(false, "", errorMsg)
	}

	return result(true, output, "")
}

func dslToMarkdown(this js.Value, args []js.Value) interface{} {
	input, filename, err := parseArgs(args)
	if err != nil {
		return result(false, "", err.Error())
	}

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", fmt.Errorf("parse failed: %w", err).Error())
	}

	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
	}

	exporter := markdown.NewExporter(markdown.DefaultOptions())
	output := exporter.Export(program)
	return result(true, output, "")
}

func dslToLikeC4(this js.Value, args []js.Value) interface{} {
	input, filename, err := parseArgs(args)
	if err != nil {
		return result(false, "", err.Error())
	}

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", fmt.Errorf("parse failed: %w", err).Error())
	}

	// Export directly from LikeC4 Program AST
	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
	}

	// Use LikeC4 exporter to generate LikeC4-compatible JSON
	exporter := jexport.NewLikeC4Exporter()
	output, err := exporter.Export(program)
	if err != nil {
		return result(false, "", fmt.Errorf("export failed: %w", err).Error())
	}

	return result(true, output, "")
}

// dslToDot converts DSL to Graphviz DOT format for diagram layout.
// Args: dsl string, [viewLevel int], [focusNodeId string], [nodeSizesJson string]
func dslToDot(this js.Value, args []js.Value) interface{} {
	if len(args) < 1 {
		return result(false, "", "at least 1 argument required")
	}

	input := args[0].String()

	// Optional: viewLevel (default 1 = L1 Context)
	viewLevel := 1
	if len(args) > 1 && args[1].Type() == js.TypeNumber {
		viewLevel = args[1].Int()
	}

	// Optional: focusNodeId (default empty)
	focusNodeId := ""
	if len(args) > 2 && args[2].Type() == js.TypeString {
		focusNodeId = args[2].String()
	}

	// Optional: nodeSizes (default empty) - passed as JSON string
	var nodeSizes map[string]struct{ Width, Height float64 }
	if len(args) > 3 && args[3].Type() == js.TypeString {
		sizesJson := args[3].String()
		if sizesJson != "" {
			// Helper struct for JSON unmarshaling
			type nodeSize struct {
				Width  float64 `json:"width"`
				Height float64 `json:"height"`
			}
			var tempSizes map[string]nodeSize
			if err := json.Unmarshal([]byte(sizesJson), &tempSizes); err == nil {
				nodeSizes = make(map[string]struct{ Width, Height float64 })
				for id, s := range tempSizes {
					nodeSizes[id] = struct{ Width, Height float64 }{s.Width, s.Height}
				}
			}
		}
	}

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil {
		return result(false, "", fmt.Errorf("parse failed: %w", err).Error())
	}

	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
	}

	// Check if Model.Items is empty
	if len(program.Model.Items) == 0 {
		return result(false, "", "model block is empty - no items found")
	}

	// Create config with view parameters
	config := dot.DefaultConfig()
	config.ViewLevel = viewLevel
	config.FocusNodeID = focusNodeId
	config.NodeSizes = nodeSizes

	exporter := dot.NewExporter(config)
	res := exporter.Export(program)

	if res.DOT == "" && len(res.Elements) == 0 {
		return result(false, "", "DOT exporter returned empty output - no elements found for this view")
	}

	// Convert Elements to []map[string]interface{} for JS compatibility
	elements := make([]interface{}, len(res.Elements))
	for i, elem := range res.Elements {
		elements[i] = map[string]interface{}{
			"id":          elem.ID,
			"kind":        elem.Kind,
			"title":       elem.Title,
			"technology":  elem.Technology,
			"description": elem.Description,
			"parentId":    elem.ParentID,
			"width":       elem.Width,
			"height":      elem.Height,
		}
	}

	// Convert Relations to []map[string]interface{} for JS compatibility
	relations := make([]interface{}, len(res.Relations))
	for i, rel := range res.Relations {
		relations[i] = map[string]interface{}{
			"from":  rel.From,
			"to":    rel.To,
			"label": rel.Label,
		}
	}

	resultData := map[string]interface{}{
		"dot":       res.DOT,
		"elements":  elements,
		"relations": relations,
	}

	return result(true, resultData, "")
}

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
	"time"

	"github.com/sruja-ai/sruja/pkg/export/dot"
	"github.com/sruja-ai/sruja/pkg/export/dsl"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
)

const (
	// defaultFilename is the default filename used when none is provided
	defaultFilename = "input.sruja"
	// minArgsRequired is the minimum number of arguments required for most functions
	minArgsRequired = 1
	// defaultExportTimeout is the default timeout for export operations
	defaultExportTimeout = 30 * time.Second
)

// main initializes the WASM module and registers all exported functions.
//
// Registers Go functions with the JavaScript global object. The channel blocks
// forever to keep the WASM module alive (WASM modules are garbage collected when
// the goroutine exits).
func main() {
	c := make(chan struct{})

	// parseDslFn := js.FuncOf(parseDsl) // Removed
	// jsonToDslFn := js.FuncOf(jsonToDsl) // Removed
	// js.Global().Set("sruja_parse_dsl", parseDslFn) // Removed
	// js.Global().Set("sruja_json_to_dsl", jsonToDslFn) // Removed

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
	js.Global().Set("sruja_analyze_governance", js.FuncOf(score))
	js.Global().Set("sruja_dsl_to_mermaid", js.FuncOf(dslToMermaid))
	js.Global().Set("sruja_dsl_to_markdown", js.FuncOf(dslToMarkdown))
	js.Global().Set("sruja_dsl_to_model", js.FuncOf(dslToModel))
	js.Global().Set("sruja_dsl_to_dot", js.FuncOf(dslToDot))
	js.Global().Set("sruja_model_to_dsl", js.FuncOf(modelToDsl))

	// Keep function references to prevent dead-code elimination
	// _ = parseDslFn
	// _ = jsonToDslFn

	<-c
}

// parseArgs extracts and validates arguments from JavaScript function calls.
// Returns the DSL text, filename (defaults to "input.sruja"), and any validation error.
// This function now uses structured error handling.
func parseArgs(args []js.Value) (input, filename string, err *ExportError) {
	if len(args) < minArgsRequired {
		return "", "", NewExportError(ErrCodeInvalidArgs,
			fmt.Sprintf("expected at least %d argument(s), got %d", minArgsRequired, len(args))).
			WithContext("expected", minArgsRequired).
			WithContext("got", len(args))
	}

	input = args[0].String()
	if validationErr := ValidateInput(input); validationErr != nil {
		return "", "", validationErr
	}

	filename = defaultFilename
	if len(args) >= 2 {
		if fn := args[1].String(); fn != "" {
			if validationErr := ValidateFilename(fn); validationErr != nil {
				return "", "", validationErr
			}
			filename = SanitizeFilename(fn)
			if filename == "" {
				filename = defaultFilename
			}
		}
	}

	return input, filename, nil
}

// parseDsl and jsonToDsl functions removed

func dslToMermaid(this js.Value, args []js.Value) (ret interface{}) {
	startTime := time.Now()
	metrics := StartMetrics("dslToMermaid", 0)

	defer func() {
		if r := recover(); r != nil {
			err := NewExportError(ErrCodePanic,
				fmt.Sprintf("panic during mermaid export: %v", r)).
				WithContext("panic", fmt.Sprint(r))
			metrics.FinishMetrics(false, err, 0, 0, 0)
			LogError("dslToMermaid", "Panic during export", ErrCodePanic, map[string]interface{}{
				"panic": fmt.Sprint(r),
			})
			ret = resultWithError(err)
		}
	}()

	input, filename, err := parseArgs(args)
	if err != nil {
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToMermaid", "Invalid arguments", err.Code, err.Context)
		return resultWithError(err)
	}

	metrics.InputSize = len(input)

	// Check resource limits before parsing
	limits := DefaultResourceLimits()
	if memErr := CheckMemoryLimit(limits.MaxMemoryMB); memErr != nil {
		metrics.FinishMetrics(false, memErr, 0, 0, 0)
		LogError("dslToMermaid", "Memory limit exceeded", memErr.Code, memErr.Context)
		return resultWithError(memErr)
	}

	// Parse Configuration
	viewLevel := 1
	targetId := ""

	if len(args) > 1 && args[1].Type() == js.TypeString {
		configJson := args[1].String()
		if configJson != "" {
			var cfg struct {
				ViewLevel int    `json:"viewLevel"`
				TargetID  string `json:"targetId"`
			}
			if err := json.Unmarshal([]byte(configJson), &cfg); err != nil {
				// Warn but continue with defaults? Or return error?
				// dslToDot returns error. Let's return error for consistency.
				parseErr := NewExportError(ErrCodeInvalidArgs, "failed to parse config JSON").WithContext("error", err.Error())
				metrics.FinishMetrics(false, parseErr, 0, 0, 0)
				return resultWithError(parseErr)
			}
			if cfg.ViewLevel > 0 {
				viewLevel = cfg.ViewLevel
			}
			targetId = cfg.TargetID
		}
	}

	if validationErr := ValidateViewLevel(viewLevel); validationErr != nil {
		metrics.FinishMetrics(false, validationErr, 0, 0, 0)
		return resultWithError(validationErr)
	}

	parseResult := parseAndValidate(input, filename)
	if parseResult.Error != nil {
		metrics.FinishMetrics(false, parseResult.Error, 0, 0, 0)
		LogError("dslToMermaid", "Parse failed", parseResult.Error.Code, parseResult.Error.Context)
		return resultWithError(parseResult.Error)
	}

	// Check timeout
	if elapsed := time.Since(startTime); elapsed > limits.MaxDuration {
		err := NewExportError(ErrCodeExportTimeout,
			fmt.Sprintf("operation exceeded maximum duration of %v", limits.MaxDuration)).
			WithContext("duration", elapsed).
			WithContext("maxDuration", limits.MaxDuration)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToMermaid", "Timeout exceeded", err.Code, err.Context)
		return resultWithError(err)
	}

	LogInfo("dslToMermaid", "Starting mermaid export", map[string]interface{}{
		"filename":  filename,
		"inputSize": len(input),
		"viewLevel": viewLevel,
		"targetId":  targetId,
	})

	config := mermaid.DefaultConfig()
	config.ViewLevel = viewLevel
	config.TargetID = targetId

	exporter := mermaid.NewExporter(config)
	output := exporter.Export(parseResult.Program)

	// Validate that we got a non-empty result
	if output == "" {
		// Provide diagnostic information about what was found
		modelItemCount := len(parseResult.Program.Model.Items)
		elementDefCount := 0
		for _, item := range parseResult.Program.Model.Items {
			if item.ElementDef != nil {
				elementDefCount++
			}
		}
		err := NewExportError(ErrCodeExportEmpty,
			"mermaid exporter returned empty output - no elements found in model or view").
			WithContext("modelItemCount", modelItemCount).
			WithContext("elementDefCount", elementDefCount).
			WithContext("viewLevel", viewLevel).
			WithContext("targetId", targetId)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToMermaid", "Empty export output", err.Code, err.Context)
		return resultWithError(err)
	}

	// Count elements and relations for metrics
	elementCount := len(parseResult.Program.Model.Items)
	relationCount := 0 // Mermaid doesn't return relation count separately

	metrics.ViewLevel = viewLevel
	metrics.FinishMetrics(true, nil, elementCount, relationCount, len(output))
	LogInfo("dslToMermaid", "Export completed", map[string]interface{}{
		"outputSize":   len(output),
		"elementCount": elementCount,
		"duration":     metrics.Duration.String(),
	})

	return resultWithData(output)
}

func dslToMarkdown(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = resultWithError(NewExportError(ErrCodePanic,
				fmt.Sprintf("panic during markdown export: %v", r)).
				WithContext("panic", fmt.Sprint(r)))
		}
	}()

	input, filename, err := parseArgs(args)
	if err != nil {
		return resultWithError(err)
	}

	parseResult := parseAndValidate(input, filename)
	if parseResult.Error != nil {
		return resultWithError(parseResult.Error)
	}

	exporter := markdown.NewExporter(markdown.DefaultOptions())
	output := exporter.Export(parseResult.Program)
	return resultWithData(output)
}

func dslToModel(this js.Value, args []js.Value) (ret interface{}) {
	startTime := time.Now()
	metrics := StartMetrics("dslToModel", 0)

	defer func() {
		if r := recover(); r != nil {
			err := NewExportError(ErrCodePanic,
				fmt.Sprintf("panic during model export: %v", r)).
				WithContext("panic", fmt.Sprint(r))
			metrics.FinishMetrics(false, err, 0, 0, 0)
			LogError("dslToModel", "Panic during export", ErrCodePanic, map[string]interface{}{
				"panic": fmt.Sprint(r),
			})
			ret = resultWithError(err)
		}
	}()

	input, filename, err := parseArgs(args)
	if err != nil {
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToModel", "Invalid arguments", err.Code, err.Context)
		return resultWithError(err)
	}

	metrics.InputSize = len(input)

	limits := DefaultResourceLimits()
	if memErr := CheckMemoryLimit(limits.MaxMemoryMB); memErr != nil {
		metrics.FinishMetrics(false, memErr, 0, 0, 0)
		LogError("dslToModel", "Memory limit exceeded", memErr.Code, memErr.Context)
		return resultWithError(memErr)
	}

	parseResult := parseAndValidate(input, filename)
	if parseResult.Error != nil {
		metrics.FinishMetrics(false, parseResult.Error, 0, 0, 0)
		LogError("dslToModel", "Parse failed", parseResult.Error.Code, parseResult.Error.Context)
		return resultWithError(parseResult.Error)
	}

	if elapsed := time.Since(startTime); elapsed > limits.MaxDuration {
		err := NewExportError(ErrCodeExportTimeout,
			fmt.Sprintf("operation exceeded maximum duration of %v", limits.MaxDuration)).
			WithContext("duration", elapsed).
			WithContext("maxDuration", limits.MaxDuration)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToModel", "Timeout exceeded", err.Code, err.Context)
		return resultWithError(err)
	}

	LogInfo("dslToModel", "Starting model export", map[string]interface{}{
		"filename":  filename,
		"inputSize": len(input),
	})

	// Use JSON exporter to generate model JSON
	exporter := jexport.NewExporter()
	output, exportErr := exporter.Export(parseResult.Program)
	if exportErr != nil {
		err := WrapError(exportErr, ErrCodeExportFailed).
			WithContext("filename", filename)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToModel", "Export failed", err.Code, err.Context)
		return resultWithError(err)
	}

	elementCount := len(parseResult.Program.Model.Items)
	metrics.FinishMetrics(true, nil, elementCount, 0, len(output))
	LogInfo("dslToModel", "Export completed", map[string]interface{}{
		"outputSize":   len(output),
		"elementCount": elementCount,
		"duration":     metrics.Duration.String(),
	})

	return resultWithData(output)
}

// dslToDot converts DSL to Graphviz DOT format for diagram layout.
// Args: dsl string, [viewLevel int], [focusNodeId string], [nodeSizesJson string]
func dslToDot(this js.Value, args []js.Value) (ret interface{}) {
	startTime := time.Now()
	metrics := StartMetrics("dslToDot", 0)

	defer func() {
		if r := recover(); r != nil {
			err := NewExportError(ErrCodePanic,
				fmt.Sprintf("panic during DOT export: %v", r)).
				WithContext("panic", fmt.Sprint(r))
			metrics.FinishMetrics(false, err, 0, 0, 0)
			LogError("dslToDot", "Panic during export", ErrCodePanic, map[string]interface{}{
				"panic": fmt.Sprint(r),
			})
			ret = resultWithError(err)
		}
	}()

	if len(args) < 1 {
		return resultWithError(NewExportError(ErrCodeInvalidArgs,
			"at least 1 argument required").
			WithContext("got", len(args)))
	}

	input := args[0].String()
	metrics.InputSize = len(input)

	limits := DefaultResourceLimits()

	if validationErr := ValidateInput(input); validationErr != nil {
		metrics.FinishMetrics(false, validationErr, 0, 0, 0)
		LogError("dslToDot", "Invalid input", validationErr.Code, validationErr.Context)
		return resultWithError(validationErr)
	}

	if memErr := CheckMemoryLimit(limits.MaxMemoryMB); memErr != nil {
		metrics.FinishMetrics(false, memErr, 0, 0, 0)
		LogError("dslToDot", "Memory limit exceeded", memErr.Code, memErr.Context)
		return resultWithError(memErr)
	}

	// Parse Configuration
	viewLevel := 1
	focusNodeId := ""
	var nodeSizes map[string]struct{ Width, Height float64 }

	if len(args) > 1 {
		if args[1].Type() == js.TypeString {
			// New Way: Config Object (JSON string)
			configJson := args[1].String()
			if configJson != "" {
				var cfg struct {
					ViewLevel   int                                        `json:"viewLevel"`
					FocusNodeId string                                     `json:"focusNodeId"`
					NodeSizes   map[string]struct{ Width, Height float64 } `json:"nodeSizes"`
				}
				if err := json.Unmarshal([]byte(configJson), &cfg); err != nil {
					parseErr := NewExportError(ErrCodeInvalidArgs, "failed to parse config JSON").WithContext("error", err.Error())
					metrics.FinishMetrics(false, parseErr, 0, 0, 0)
					return resultWithError(parseErr)
				}

				if cfg.ViewLevel > 0 {
					viewLevel = cfg.ViewLevel
				}
				focusNodeId = cfg.FocusNodeId

				if len(cfg.NodeSizes) > 0 {
					nodeSizes = make(map[string]struct{ Width, Height float64 })
					for id, s := range cfg.NodeSizes {
						if s.Width < 0 || s.Height < 0 || s.Width > 10000 || s.Height > 10000 {
							sizeErr := NewExportError(ErrCodeInvalidInput, "node size values must be between 0 and 10000").
								WithContext("nodeId", id)
							metrics.FinishMetrics(false, sizeErr, 0, 0, 0)
							return resultWithError(sizeErr)
						}
						nodeSizes[id] = s
					}
				}
			}
		} else if args[1].Type() == js.TypeNumber {
			// Legacy Way: Positional Arguments
			viewLevel = args[1].Int()

			if len(args) > 2 && args[2].Type() == js.TypeString {
				focusNodeId = args[2].String()
				if len(focusNodeId) > 255 {
					focusNodeId = focusNodeId[:255]
				}
			}

			if len(args) > 3 && args[3].Type() == js.TypeString {
				sizesJson := args[3].String()
				if sizesJson != "" {
					type nodeSize struct {
						Width  float64 `json:"width"`
						Height float64 `json:"height"`
					}
					var tempSizes map[string]nodeSize
					if err := json.Unmarshal([]byte(sizesJson), &tempSizes); err != nil {
						return resultWithError(NewExportError(ErrCodeInvalidInput, "failed to parse node sizes JSON"))
					}
					nodeSizes = make(map[string]struct{ Width, Height float64 })
					for id, s := range tempSizes {
						nodeSizes[id] = struct{ Width, Height float64 }{s.Width, s.Height}
					}
				}
			}
		}
	}

	if validationErr := ValidateViewLevel(viewLevel); validationErr != nil {
		metrics.FinishMetrics(false, validationErr, 0, 0, 0)
		return resultWithError(validationErr)
	}

	parseResult := parseAndValidate(input, defaultFilename)
	if parseResult.Error != nil {
		metrics.FinishMetrics(false, parseResult.Error, 0, 0, 0)
		LogError("dslToDot", "Parse failed", parseResult.Error.Code, parseResult.Error.Context)
		return resultWithError(parseResult.Error)
	}

	if elapsed := time.Since(startTime); elapsed > limits.MaxDuration {
		err := NewExportError(ErrCodeExportTimeout,
			fmt.Sprintf("operation exceeded maximum duration of %v", limits.MaxDuration)).
			WithContext("duration", elapsed).
			WithContext("maxDuration", limits.MaxDuration)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToDot", "Timeout exceeded", err.Code, err.Context)
		return resultWithError(err)
	}

	LogInfo("dslToDot", "Starting DOT export", map[string]interface{}{
		"viewLevel":   viewLevel,
		"focusNodeId": focusNodeId,
		"inputSize":   len(input),
	})

	// Create config with view parameters
	config := dot.DefaultConfig()
	config.ViewLevel = viewLevel
	config.FocusNodeID = focusNodeId
	config.NodeSizes = nodeSizes

	// Optimize layout based on View Level
	if viewLevel == 3 {
		// Component View: Keep TB but increase spacing to prevent overlaps
		config.RankDir = "TB"
		// Increase spacing for complex component views
		config.RankSep = 220
		config.NodeSep = 180
	} else {
		// System/Container View: Hierarchy looks best Top-Bottom
		config.RankDir = "TB"
		// Standard spacing
		config.RankSep = 180
		config.NodeSep = 150
	}

	exporter := dot.NewExporter(config)
	res := exporter.Export(parseResult.Program)

	if res.DOT == "" && len(res.Elements) == 0 {
		err := NewExportError(ErrCodeExportEmpty,
			"DOT exporter returned empty output - no elements found for this view").
			WithContext("viewLevel", viewLevel).
			WithContext("focusNodeId", focusNodeId)
		metrics.FinishMetrics(false, err, 0, 0, 0)
		LogError("dslToDot", "Empty export output", err.Code, err.Context)
		return resultWithError(err)
	}

	metrics.ViewLevel = viewLevel
	metrics.FinishMetrics(true, nil, len(res.Elements), len(res.Relations), len(res.DOT))
	LogInfo("dslToDot", "Export completed", map[string]interface{}{
		"outputSize":    len(res.DOT),
		"elementCount":  len(res.Elements),
		"relationCount": len(res.Relations),
		"viewLevel":     viewLevel,
		"duration":      metrics.Duration.String(),
	})

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

	return resultWithData(resultData)
}

// modelToDsl converts a JSON model dump back to DSL format.
// Args: modelJson string
func modelToDsl(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = resultWithError(NewExportError(ErrCodePanic,
				fmt.Sprintf("panic during model to DSL conversion: %v", r)).
				WithContext("panic", fmt.Sprint(r)))
		}
	}()

	if len(args) < 1 {
		return resultWithError(NewExportError(ErrCodeInvalidArgs,
			"at least 1 argument required (model JSON)").
			WithContext("got", len(args)))
	}

	jsonStr := args[0].String()
	if jsonStr == "" {
		return resultWithData("")
	}

	var model jexport.SrujaModelDump
	if err := json.Unmarshal([]byte(jsonStr), &model); err != nil {
		return resultWithError(NewExportError(ErrCodeInvalidInput,
			"failed to parse model JSON").
			WithContext("parseError", err.Error()))
	}

	// JSON to DSL conversion
	output := dsl.Print(&model)
	return resultWithData(output)
}

//go:build js && wasm

// cmd/wasm/lsp_handlers.go
// LSP handler functions for WASM module
package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
)

// getDiagnostics returns diagnostics (errors/warnings) for the DSL text
func getDiagnostics(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	ws, program, err := parseToWorkspace(input, "input.sruja")
	var diags []diagnostics.Diagnostic
	if ws != nil {
		diags = ws.Diags
	}
	if err != nil {
		// If we have diagnostics, return them even if there's an error
		// Otherwise, return the error
		if len(diags) == 0 {
			return lspResult(false, nil, err.Error())
		}
	}

	// Validator now works directly with LikeC4 AST (no conversion needed)
	if program != nil {
		validator := engine.NewValidator()
		validator.RegisterRule(&engine.UniqueIDRule{})
		validator.RegisterRule(&engine.ValidReferenceRule{})
		validator.RegisterRule(&engine.RelationTagRule{})
		validator.RegisterRule(&engine.CompletenessRule{})
		errs := validator.Validate(program)
		for _, e := range errs {
			diags = append(diags, diagnostics.Diagnostic{
				Code:     e.Code,
				Severity: e.Severity,
				Message:  e.Message,
				Location: e.Location,
			})
		}
	}

	diagJSON := make([]map[string]interface{}, len(diags))
	for i, d := range diags {
		diagJSON[i] = map[string]interface{}{
			"code":     d.Code,
			"severity": string(d.Severity),
			"message":  d.Message,
			"location": map[string]interface{}{
				"file":   d.Location.File,
				"line":   d.Location.Line,
				"column": d.Location.Column,
			},
		}
	}

	jsonBytes, err := json.Marshal(diagJSON)
	if err != nil {
		return lspResult(false, nil, fmt.Sprintf("failed to marshal diagnostics: %v", err))
	}
	return lspResult(true, string(jsonBytes), "")
}

// getSymbols returns all symbols (identifiers) in the DSL
func getSymbols(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil || program == nil {
		return lspResult(true, "[]", "")
	}

	symbols := extractSymbolsFromProgram(program)
	// Always return an array, even if empty, for consistent JSON serialization
	if symbols == nil {
		symbols = []Symbol{}
	}
	jsonBytes, err := json.Marshal(symbols)
	if err != nil {
		return lspResult(false, nil, fmt.Sprintf("failed to marshal symbols: %v", err))
	}
	return lspResult(true, string(jsonBytes), "")
}

// codeActions returns code actions (quick fixes) for diagnostics
func codeActions(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 2 {
		return lspResult(false, nil, "invalid arguments: need (text, diagnostics)")
	}
	input := args[0].String()
	diagsJSON := args[1].String()

	var diags []map[string]interface{}
	if err := json.Unmarshal([]byte(diagsJSON), &diags); err != nil {
		return lspResult(false, nil, "invalid diagnostics JSON: "+err.Error())
	}

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil || program == nil {
		return lspResult(true, "[]", "")
	}

	actions := []map[string]interface{}{}
	for _, diag := range diags {
		code, _ := diag["code"].(string)
		message, _ := diag["message"].(string)

		// Generate quick fixes based on diagnostic code
		if code == "REFERENCE_NOT_FOUND" {
			// Extract element name from message
			startIdx := strings.Index(message, "'")
			if startIdx >= 0 {
				endIdx := strings.Index(message[startIdx+1:], "'")
				if endIdx >= 0 {
					elementName := message[startIdx+1 : startIdx+1+endIdx]
					// Find similar elements
					symbols := extractSymbolsFromProgram(program)
					for _, sym := range symbols {
						if strings.Contains(strings.ToLower(sym.Name), strings.ToLower(elementName)) {
							// Safely extract location information with type assertions
							location, ok := diag["location"].(map[string]interface{})
							if !ok {
								continue
							}
							line, _ := location["line"].(float64)
							column, _ := location["column"].(float64)

							actions = append(actions, map[string]interface{}{
								"title":   "Replace with '" + sym.Name + "'",
								"command": "sruja.replaceElement",
								"arguments": []interface{}{
									"input.sruja",
									int(line),
									int(column),
									sym.Name,
								},
							})
							break
						}
					}
				}
			}
		}
	}

	jsonBytes, err := json.Marshal(actions)
	if err != nil {
		return lspResult(false, nil, fmt.Sprintf("failed to marshal code actions: %v", err))
	}
	return lspResult(true, string(jsonBytes), "")
}

// score calculates the architecture score for the given DSL
func score(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = result(false, "", fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	input := args[0].String()

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil {
		return result(false, "", "parse error: "+err.Error())
	}
	if program == nil {
		return result(false, "", "program is nil")
	}

	// Scorer now works directly with LikeC4 AST (no conversion needed)
	scorer := engine.NewScorer()
	card := scorer.CalculateScore(program)

	jsonBytes, err := json.Marshal(card)
	if err != nil {
		return result(false, "", err.Error())
	}

	ret = result(true, string(jsonBytes), "")
	return
}

//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"syscall/js"
)

// hover returns hover information at the given position
func hover(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 3 {
		return lspResult(false, nil, "invalid arguments: need (text, line, column)")
	}
	input := args[0].String()
	line := args[1].Int()
	column := args[2].Int()

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil || program == nil {
		return lspResult(true, "null", "")
	}

	hoverInfo := findHoverInfoFromProgram(program, input, line, column)
	if hoverInfo == nil {
		return lspResult(true, "null", "")
	}

	jsonBytes, err := json.Marshal(hoverInfo)
	if err != nil {
		return lspResult(false, nil, fmt.Sprintf("failed to marshal hover info: %v", err))
	}
	return lspResult(true, string(jsonBytes), "")
}

// completion returns completion suggestions at the given position
func completion(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 3 {
		return lspResult(false, nil, "invalid arguments: need (text, line, column)")
	}
	input := args[0].String()

	keywords := []string{"specification", "model", "views", "system", "container", "component", "database", "queue", "person", "relation", "requirement", "adr", "library", "import", "metadata", "description"}

	_, program, _ := parseToWorkspace(input, "input.sruja")
	if program != nil {
		// Extract symbols from Model block
		symbols := extractSymbolsFromProgram(program)
		for _, sym := range symbols {
			keywords = append(keywords, sym.Name)
		}
	}

	// Pre-allocate with exact capacity for better performance
	completions := make([]CompletionItem, len(keywords))
	for i, kw := range keywords {
		completions[i] = CompletionItem{
			Label: kw,
			Kind:  "keyword",
		}
	}

	jsonBytes, err := json.Marshal(completions)
	if err != nil {
		return lspResult(false, nil, fmt.Sprintf("failed to marshal completions: %v", err))
	}
	return lspResult(true, string(jsonBytes), "")
}

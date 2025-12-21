//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"
)

// goToDefinition returns the definition location for a symbol at the given position
func goToDefinition(this js.Value, args []js.Value) (ret interface{}) {
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

	def := findDefinitionFromProgram(program, input, line, column)
	if def == nil {
		return lspResult(true, "null", "")
	}

	jsonBytes, _ := json.Marshal(def)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

// findReferences finds all references to a symbol at the given position
func findReferences(this js.Value, args []js.Value) (ret interface{}) {
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
		return lspResult(true, "[]", "")
	}

	def := findDefinitionFromProgram(program, input, line, column)
	if def == nil {
		return lspResult(true, "[]", "")
	}

	lines := strings.Split(input, "\n")
	if def["line"] == nil {
		return lspResult(true, "[]", "")
	}
	defLine := int(def["line"].(float64))
	if defLine < 1 || defLine > len(lines) {
		return lspResult(true, "[]", "")
	}

	defLineText := lines[defLine-1]
	defCol := int(def["column"].(float64))
	if defCol < 1 || defCol > len(defLineText) {
		return lspResult(true, "[]", "")
	}

	start := defCol - 1
	for start > 0 && (isIdentChar(defLineText[start-1]) || defLineText[start-1] == '.') {
		start--
	}
	end := defCol - 1
	for end < len(defLineText) && (isIdentChar(defLineText[end]) || defLineText[end] == '.') {
		end++
	}
	if start >= end {
		return lspResult(true, "[]", "")
	}

	symbolName := defLineText[start:end]
	references := findSymbolReferencesFromProgram(program, input, symbolName)

	refJSON := make([]map[string]interface{}, len(references))
	for i, ref := range references {
		refJSON[i] = map[string]interface{}{
			"file":   "input.sruja",
			"line":   ref["line"],
			"column": ref["column"],
		}
	}

	jsonBytes, _ := json.Marshal(refJSON)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

// rename renames a symbol and all its references
func rename(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 4 {
		return lspResult(false, nil, "invalid arguments: need (text, line, column, newName)")
	}
	input := args[0].String()
	line := args[1].Int()
	column := args[2].Int()
	newName := args[3].String()

	if newName == "" {
		return lspResult(false, nil, "new name cannot be empty")
	}

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil || program == nil {
		return lspResult(false, nil, "parse error: "+err.Error())
	}

	// Rename works directly with text (no AST conversion needed)
	lines := strings.Split(input, "\n")
	if line < 1 || line > len(lines) {
		return lspResult(false, nil, "invalid line")
	}

	lineText := lines[line-1]
	if column < 1 || column > len(lineText) {
		return lspResult(false, nil, "invalid column")
	}

	start := column - 1
	for start > 0 && (isIdentChar(lineText[start-1]) || lineText[start-1] == '.') {
		start--
	}
	end := column - 1
	for end < len(lineText) && (isIdentChar(lineText[end]) || lineText[end] == '.') {
		end++
	}
	if start >= end {
		return lspResult(false, nil, "could not extract symbol name at cursor")
	}

	oldName := lineText[start:end]
	result := performRename(input, oldName, newName)
	ret = lspResult(true, result, "")
	return
}

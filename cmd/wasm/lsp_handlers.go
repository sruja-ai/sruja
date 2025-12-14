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
	"github.com/sruja-ai/sruja/pkg/language"
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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, diags, err := p.Parse("input.sruja", input)
	if err != nil {
		if len(diags) == 0 {
			return lspResult(false, nil, err.Error())
		}
	}

	if program != nil && program.Architecture != nil {
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

	jsonBytes, _ := json.Marshal(diagJSON)
	ret = lspResult(true, string(jsonBytes), "")
	return
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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(true, "[]", "")
	}

	symbols := extractSymbols(program.Architecture)
	jsonBytes, _ := json.Marshal(symbols)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(true, "null", "")
	}

	hoverInfo := findHoverInfo(program.Architecture, input, line, column)
	if hoverInfo == nil {
		return lspResult(true, "null", "")
	}

	jsonBytes, _ := json.Marshal(hoverInfo)
	ret = lspResult(true, string(jsonBytes), "")
	return
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

	keywords := []string{"architecture", "system", "container", "component", "datastore", "queue", "person", "relation", "requirement", "adr", "library", "import", "metadata", "description"}

	p, err := language.NewParser()
	if err == nil {
		program, _, _ := p.Parse("input.sruja", input)
		if program != nil && program.Architecture != nil {
			symbols := extractSymbols(program.Architecture)
			for _, sym := range symbols {
				keywords = append(keywords, sym["name"].(string))
			}
		}
	}

	completions := make([]map[string]interface{}, len(keywords))
	for i, kw := range keywords {
		completions[i] = map[string]interface{}{
			"label": kw,
			"kind":  "keyword",
		}
	}

	jsonBytes, _ := json.Marshal(completions)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(true, "null", "")
	}

	def := findDefinition(program.Architecture, input, line, column)
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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(true, "[]", "")
	}

	def := findDefinition(program.Architecture, input, line, column)
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
	references := findSymbolReferences(program.Architecture, input, symbolName)

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

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(false, nil, "parse error: "+err.Error())
	}

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

// format formats the DSL text
func format(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	p, err := language.NewParser()
	if err != nil {
		return lspResult(false, nil, err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil || program == nil || program.Architecture == nil {
		return lspResult(true, input, "")
	}

	printer := language.NewPrinter()
	formatted := printer.Print(program)
	ret = lspResult(true, formatted, "")
	return
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

	p, err := language.NewParser()
	if err != nil {
		return result(false, "", err.Error())
	}

	program, _, err := p.Parse("input.sruja", input)
	if err != nil {
		return result(false, "", "parse error: "+err.Error())
	}
	if program == nil {
		return result(false, "", "program is nil")
	}

	scorer := engine.NewScorer()
	card := scorer.CalculateScore(program)

	jsonBytes, err := json.Marshal(card)
	if err != nil {
		return result(false, "", err.Error())
	}

	ret = result(true, string(jsonBytes), "")
	return
}

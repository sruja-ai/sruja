//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strings"
	"syscall/js"

	"github.com/sruja-ai/sruja/internal/converter"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {
	c := make(chan struct{})

	// Register all functions with explicit references to prevent dead-code elimination
	parseDslFn := js.FuncOf(parseDsl)
	jsonToDslFn := js.FuncOf(jsonToDsl)

	js.Global().Set("sruja_parse_dsl", parseDslFn)
	js.Global().Set("sruja_json_to_dsl", jsonToDslFn)

	// LSP functions
	js.Global().Set("sruja_get_diagnostics", js.FuncOf(getDiagnostics))
	js.Global().Set("sruja_get_symbols", js.FuncOf(getSymbols))
	js.Global().Set("sruja_hover", js.FuncOf(hover))
	js.Global().Set("sruja_completion", js.FuncOf(completion))
	js.Global().Set("sruja_go_to_definition", js.FuncOf(goToDefinition))
	js.Global().Set("sruja_find_references", js.FuncOf(findReferences))
	js.Global().Set("sruja_rename", js.FuncOf(rename))
	js.Global().Set("sruja_format", js.FuncOf(format))
	js.Global().Set("sruja_score", js.FuncOf(score))

	// Explicitly keep function references to prevent optimization
	_ = parseDslFn
	_ = jsonToDslFn

	<-c
}

// Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters

func parseDsl(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = result(false, "", fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	input := args[0].String()
	filename := "input.sruja"
	if len(args) >= 2 {
		fn := args[1].String()
		if fn != "" {
			filename = fn
		}
	}

	p, err := language.NewParser()
	if err != nil {
		return result(false, "", err.Error())
	}

	program, _, err := p.Parse(filename, input)
	if err != nil {
		return result(false, "", err.Error())
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.ScenarioFQNRule{})

	errs := validator.Validate(program)
	if len(errs) > 0 {
		// Build error message efficiently with strings.Builder
		var msgBuilder strings.Builder
		msgBuilder.Grow(len(errs) * 50) // Pre-allocate space
		msgBuilder.WriteString("Validation errors:\n")
		for i, e := range errs {
			if i > 0 {
				msgBuilder.WriteByte('\n')
			}
			msgBuilder.WriteString(e.Message)
		}
		return result(false, "", msgBuilder.String())
	}

	// Convert to JSON
	archJson := converter.ConvertToJSON(program.Architecture)
	jsonBytes, err := json.Marshal(archJson)
	if err != nil {
		return result(false, "", err.Error())
	}

	ret = result(true, string(jsonBytes), "")
	return
}

func jsonToDsl(this js.Value, args []js.Value) (ret interface{}) {
	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	input := args[0].String()

	var archJson converter.ArchitectureJSON
	if err := json.Unmarshal([]byte(input), &archJson); err != nil {
		return result(false, "", "json unmarshal failed: "+err.Error())
	}

	// Convert JSON back to AST
	astArch := converter.ConvertFromJSON(archJson)
	program := &language.Program{
		Architecture: astArch,
	}

	// Print AST to DSL
	printer := language.NewPrinter()
	dsl := printer.Print(program)

	ret = result(true, dsl, "")
	return
}

func result(ok bool, data string, err string) interface{} {
	res := make(map[string]interface{}, 3) // Pre-allocate with known capacity
	res["ok"] = ok
	if ok {
		res["json"] = data // For parseDsl
		res["dsl"] = data  // For jsonToDsl
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

// LSP Functions

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
		// Even if there's an error, we might have diagnostics
		if len(diags) == 0 {
			return lspResult(false, nil, err.Error())
		}
		// Continue to return diagnostics even if there's an error
	}

	// Also run validation if we have a valid program
	if program != nil && program.Architecture != nil {
		validator := engine.NewValidator()
		validator.RegisterRule(&engine.UniqueIDRule{})
		validator.RegisterRule(&engine.ValidReferenceRule{})
		validator.RegisterRule(&engine.RelationTagRule{})
		validator.RegisterRule(&engine.CompletenessRule{}) // Add completeness check
		errs := validator.Validate(program)
		for _, e := range errs {
			diags = append(diags, diagnostics.Diagnostic{
				Code:     e.Code,     // Use code from rule
				Severity: e.Severity, // Use severity from rule (don't override to Error)
				Message:  e.Message,
				Location: e.Location, // Use location from rule
			})
		}
	}

	// Convert diagnostics to JSON
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
		return lspResult(true, "[]", "") // Empty symbols on parse error
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
		return lspResult(true, "null", "") // No hover info on parse error
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
	_ = args[1].Int()
	_ = args[2].Int()

	// Get keywords and symbols for completion
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

	// Pre-allocate completions slice
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
		return lspResult(true, "null", "") // No definition on parse error
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
		return lspResult(true, "[]", "") // Empty references on parse error
	}

	// First, find the symbol at the position
	def := findDefinition(program.Architecture, input, line, column)
	if def == nil {
		return lspResult(true, "[]", "") // No definition, no references
	}

	// Extract the symbol name from the definition location
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

	// Extract symbol name from definition line
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

	// Find all references to this symbol
	references := findSymbolReferences(program.Architecture, input, symbolName)

	// Convert to JSON
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

// findSymbolReferences finds all references to a symbol in the architecture
func findSymbolReferences(arch *language.Architecture, text string, symbolName string) []map[string]interface{} {
	references := []map[string]interface{}{}
	lines := strings.Split(text, "\n")

	// Helper to check if a qualified name matches the symbol
	matchesSymbol := func(qname string) bool {
		// Exact match
		if qname == symbolName {
			return true
		}
		// Check if symbol is part of qualified name (e.g., "App" matches "App.API")
		if strings.HasPrefix(qname, symbolName+".") {
			return true
		}
		// Check if qualified name ends with symbol (e.g., "API" matches "App.API")
		if strings.HasSuffix(qname, "."+symbolName) {
			return true
		}
		// Check if symbol is in the middle (e.g., "App" matches "MyApp.API")
		parts := strings.Split(qname, ".")
		for _, part := range parts {
			if part == symbolName {
				return true
			}
		}
		return false
	}

	// Find references in relations
	findInRelations := func(rels []*language.Relation) {
		for _, rel := range rels {
			fromStr := rel.From.String()
			toStr := rel.To.String()

			if matchesSymbol(fromStr) {
				loc := rel.Location()
				// Find the actual position in text
				lineNum := findLineForRelation(lines, fromStr, loc.Line)
				if lineNum > 0 {
					colNum := findColumnForSymbol(lines[lineNum-1], fromStr)
					if colNum > 0 {
						references = append(references, map[string]interface{}{
							"line":   lineNum,
							"column": colNum,
						})
					}
				}
			}

			if matchesSymbol(toStr) {
				loc := rel.Location()
				lineNum := findLineForRelation(lines, toStr, loc.Line)
				if lineNum > 0 {
					colNum := findColumnForSymbol(lines[lineNum-1], toStr)
					if colNum > 0 {
						references = append(references, map[string]interface{}{
							"line":   lineNum,
							"column": colNum,
						})
					}
				}
			}
		}
	}

	// Search in architecture-level relations
	findInRelations(arch.Relations)

	// Search in system relations
	for _, sys := range arch.Systems {
		findInRelations(sys.Relations)
		// Search in container relations
		for _, cont := range sys.Containers {
			findInRelations(cont.Relations)
		}
	}

	return references
}

// Helper functions for finding positions
func findLineForRelation(lines []string, symbol string, hintLine int) int {
	// Start searching from hint line
	for i := hintLine - 1; i < len(lines); i++ {
		if strings.Contains(lines[i], symbol) {
			return i + 1
		}
	}
	// Search backwards
	for i := hintLine - 2; i >= 0; i-- {
		if strings.Contains(lines[i], symbol) {
			return i + 1
		}
	}
	return 0
}

func findColumnForSymbol(lineText string, symbol string) int {
	// Find the position of the symbol in the line
	idx := strings.Index(lineText, symbol)
	if idx >= 0 {
		return idx + 1 // Convert to 1-based
	}
	return 0
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

	// Extract old symbol name from cursor position (not definition position)
	lines := strings.Split(input, "\n")
	if line < 1 || line > len(lines) {
		return lspResult(false, nil, "invalid line")
	}

	lineText := lines[line-1]
	if column < 1 || column > len(lineText) {
		return lspResult(false, nil, "invalid column")
	}

	// Extract the symbol name at cursor position
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

	// Verify the symbol exists (optional check)
	def := findDefinition(program.Architecture, input, line, column)
	if def == nil {
		// Still allow rename even if definition not found - might be renaming a reference
	}

	// Perform rename: replace all occurrences intelligently
	result := input

	// Strategy: Replace the symbol name in all contexts
	// 1. Replace exact word matches (standalone identifiers) - use word boundaries
	wordBoundaryPattern := `\b` + regexp.QuoteMeta(oldName) + `\b`
	result = regexp.MustCompile(wordBoundaryPattern).ReplaceAllString(result, newName)

	// 2. Replace in qualified names (e.g., "App.API" -> "App.API2")
	// Match oldName when it appears after a dot or arrow, followed by whitespace/punctuation
	qualifiedAfterDot := `\.` + regexp.QuoteMeta(oldName) + `(\s|->|"|\.|{|}|,|\[|\]|$)`
	result = regexp.MustCompile(qualifiedAfterDot).ReplaceAllStringFunc(result, func(match string) string {
		return "." + newName + match[len(oldName)+1:]
	})

	// 3. Replace in qualified names before arrow (e.g., "App.API ->" -> "App.API2 ->")
	qualifiedBeforeArrow := regexp.QuoteMeta(oldName) + `\s*->`
	result = regexp.MustCompile(qualifiedBeforeArrow).ReplaceAllString(result, newName+" ->")

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
		// On parse error, return original text
		return lspResult(true, input, "")
	}

	// Format by printing and re-parsing
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

// Helper functions

func lspResult(ok bool, data interface{}, err string) interface{} {
	res := make(map[string]interface{}, 3) // Pre-allocate
	res["ok"] = ok
	if ok {
		if data != nil {
			if str, ok := data.(string); ok {
				res["data"] = str
			} else {
				res["data"] = data
			}
		} else {
			res["data"] = nil
		}
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

// extractSymbols extracts all symbols from the architecture
func extractSymbols(arch *language.Architecture) []map[string]interface{} {
	// Pre-allocate with estimated capacity
	estimatedSize := len(arch.Systems) + len(arch.Containers) + len(arch.Components) + len(arch.DataStores) + len(arch.Persons)
	symbols := make([]map[string]interface{}, 0, estimatedSize)

	// Extract systems
	for _, sys := range arch.Systems {
		symbols = append(symbols, map[string]interface{}{
			"name": sys.ID,
			"kind": "system",
			"line": sys.Pos.Line, // Use actual line from AST
		})
	}

	// Extract containers
	for _, cont := range arch.Containers {
		symbols = append(symbols, map[string]interface{}{
			"name": cont.ID,
			"kind": "container",
			"line": cont.Pos.Line,
		})
	}

	// Extract components
	for _, comp := range arch.Components {
		symbols = append(symbols, map[string]interface{}{
			"name": comp.ID,
			"kind": "component",
			"line": comp.Pos.Line,
		})
	}

	// Extract datastores
	for _, ds := range arch.DataStores {
		symbols = append(symbols, map[string]interface{}{
			"name": ds.ID,
			"kind": "datastore",
			"line": ds.Pos.Line,
		})
	}

	// Extract persons
	for _, person := range arch.Persons {
		symbols = append(symbols, map[string]interface{}{
			"name": person.ID,
			"kind": "person",
			"line": person.Pos.Line,
		})
	}

	return symbols
}

// findHoverInfo finds hover information at the given position
func findHoverInfo(arch *language.Architecture, text string, line, column int) map[string]interface{} {
	lines := strings.Split(text, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]
	if column < 1 || column > len(lineText) {
		return nil
	}

	// Find word at position more efficiently
	start := column - 1
	end := column - 1
	for start > 0 && isIdentChar(lineText[start-1]) {
		start--
	}
	for end < len(lineText) && isIdentChar(lineText[end]) {
		end++
	}
	if start >= end {
		return nil
	}

	word := lineText[start:end]

	// Check if this word matches a symbol
	for _, sys := range arch.Systems {
		if sys.ID == word {
			var contents strings.Builder
			contents.Grow(len(sys.ID) + len(sys.Label) + 20)
			contents.WriteString("System: ")
			contents.WriteString(sys.ID)
			if sys.Label != "" {
				contents.WriteString("\n")
				contents.WriteString(sys.Label)
			}
			return map[string]interface{}{
				"contents": contents.String(),
			}
		}
	}

	return nil
}

// isIdentChar checks if a character is a valid identifier character
func isIdentChar(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_'
}

// findDefinition finds the definition location for a symbol at the given position
func findDefinition(arch *language.Architecture, text string, line, column int) map[string]interface{} {
	lines := strings.Split(text, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]

	// Extract the full qualified name (including dots)
	// Strategy: find the word boundaries, but also include dots and adjacent identifiers
	// First, find where the qualified name starts (go backwards until we hit non-identifier/non-dot)
	start := column - 1
	for start > 0 {
		prevChar := lineText[start-1]
		if isIdentChar(prevChar) || prevChar == '.' {
			start--
		} else {
			break
		}
	}

	// Then find where it ends (go forwards until we hit non-identifier/non-dot)
	end := column - 1
	for end < len(lineText) {
		currChar := lineText[end]
		if isIdentChar(currChar) || currChar == '.' {
			end++
		} else {
			break
		}
	}

	if start >= end {
		return nil
	}

	qualifiedName := lineText[start:end]

	// Split by dots to get the path parts
	parts := strings.Split(qualifiedName, ".")
	if len(parts) == 0 {
		return nil
	}

	// Navigate through the hierarchy
	// First part should be a System
	if len(parts) == 1 {
		// Single identifier - check for System
		for _, sys := range arch.Systems {
			if sys.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   sys.Pos.Line,
					"column": sys.Pos.Column,
				}
			}
		}
		// Check top-level containers, components, datastores
		for _, cont := range arch.Containers {
			if cont.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   cont.Pos.Line,
					"column": cont.Pos.Column,
				}
			}
		}
		for _, comp := range arch.Components {
			if comp.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		for _, ds := range arch.DataStores {
			if ds.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		return nil
	}

	// Multi-part qualified name - navigate hierarchy
	// Find the System (first part)
	var currentSystem *language.System
	for _, sys := range arch.Systems {
		if sys.ID == parts[0] {
			currentSystem = sys
			break
		}
	}
	if currentSystem == nil {
		return nil
	}

	// If only 2 parts, look for Container/Component/DataStore in System
	if len(parts) == 2 {
		// Check containers in system
		for _, cont := range currentSystem.Containers {
			if cont.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   cont.Pos.Line,
					"column": cont.Pos.Column,
				}
			}
		}
		// Check components in system
		for _, comp := range currentSystem.Components {
			if comp.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		// Check datastores in system
		for _, ds := range currentSystem.DataStores {
			if ds.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		return nil
	}

	// 3+ parts: System.Container.Child
	if len(parts) >= 3 {
		// Find container (second part)
		var currentContainer *language.Container
		for _, cont := range currentSystem.Containers {
			if cont.ID == parts[1] {
				currentContainer = cont
				break
			}
		}
		if currentContainer == nil {
			return nil
		}

		// Find child in container (third part)
		childID := parts[2]
		// Check components
		for _, comp := range currentContainer.Components {
			if comp.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		// Check datastores
		for _, ds := range currentContainer.DataStores {
			if ds.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		// Check queues
		for _, q := range currentContainer.Queues {
			if q.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   q.Pos.Line,
					"column": q.Pos.Column,
				}
			}
		}
	}

	return nil
}

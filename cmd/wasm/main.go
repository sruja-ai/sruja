//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"

	"github.com/sruja-ai/sruja/internal/converter"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/html"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/svg"
	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {
	c := make(chan struct{})
	fmt.Println("WASM Go Starting...")
	js.Global().Set("sruja_parse_dsl", js.FuncOf(parseDsl))
	fmt.Println("Registered sruja_parse_dsl")
	js.Global().Set("sruja_json_to_dsl", js.FuncOf(jsonToDsl))
	fmt.Println("Registered sruja_json_to_dsl")
	js.Global().Set("sruja_dsl_to_markdown", js.FuncOf(dslToMarkdown))
	fmt.Println("Registered sruja_dsl_to_markdown")
	js.Global().Set("sruja_dsl_to_svg", js.FuncOf(dslToSvg))
	fmt.Println("Registered sruja_dsl_to_svg")
	js.Global().Set("sruja_dsl_to_html", js.FuncOf(dslToHtml))
	fmt.Println("Registered sruja_dsl_to_html")
	js.Global().Set("sruja_dsl_to_html_v2", js.FuncOf(dslToHtmlV2))
	fmt.Println("Registered sruja_dsl_to_html_v2")
	// LSP functions
	js.Global().Set("sruja_get_diagnostics", js.FuncOf(getDiagnostics))
	fmt.Println("Registered sruja_get_diagnostics")
	js.Global().Set("sruja_get_symbols", js.FuncOf(getSymbols))
	fmt.Println("Registered sruja_get_symbols")
	js.Global().Set("sruja_hover", js.FuncOf(hover))
	fmt.Println("Registered sruja_hover")
	js.Global().Set("sruja_completion", js.FuncOf(completion))
	fmt.Println("Registered sruja_completion")
	js.Global().Set("sruja_go_to_definition", js.FuncOf(goToDefinition))
	fmt.Println("Registered sruja_go_to_definition")
	js.Global().Set("sruja_format", js.FuncOf(format))
	fmt.Println("Registered sruja_format")
	js.Global().Set("sruja_score", js.FuncOf(score))
	fmt.Println("Registered sruja_score")
	fmt.Println("WASM Go Initialized")
	<-c
}

// ... existing parseDsl and jsonToDsl ...

func dslToMarkdown(this js.Value, args []js.Value) interface{} {
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
		return result(false, "", err.Error())
	}

	if program.Architecture == nil {
		return result(false, "", "architecture is nil")
	}

	exporter := markdown.NewExporter()
	md, err := exporter.Export(program.Architecture)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, md, "")
}

func dslToSvg(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in dslToSvg:", r)
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
		return result(false, "", err.Error())
	}

	if program.Architecture == nil {
		return result(false, "", "architecture is nil")
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.RelationTagRule{})
	validator.RegisterRule(&engine.ScenarioFQNRule{})
	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += e.Message + "\n"
		}
		return result(false, "", "Validation errors:\n"+msg)
	}

	exporter := svg.NewExporter()
	exporter.EmbedFonts = true
	svgContent, err := exporter.ExportAll(program.Architecture)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, svgContent, "")
}

func dslToHtml(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in dslToHtml:", r)
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
		return result(false, "", err.Error())
	}

	if program.Architecture == nil {
		return result(false, "", "architecture is nil")
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.RelationTagRule{})
	validator.RegisterRule(&engine.ScenarioFQNRule{})
	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += e.Message + "\n"
		}
		return result(false, "", "Validation errors:\n"+msg)
	}

	// Export to HTML using V2 mode (Cytoscape.js + Dagre layout)
	// Creates a fully standalone HTML file with embedded libraries and proper edge rendering
	exporter := html.NewExporter()
	exporter.Mode = html.ModeSingleFile // Use V1-style React viewer with full UI
	exporter.EmbedJSON = true
	exporter.Minify = true
	htmlContent, err := exporter.ExportFromArchitecture(program.Architecture)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, htmlContent, "")
}

// dslToHtmlV2 exports DSL to HTML using V2 mode (Cytoscape.js + ELK layout)
// This is an experimental mode with interactive navigation and documentation panels
func dslToHtmlV2(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in dslToHtmlV2:", r)
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
		return result(false, "", err.Error())
	}

	if program.Architecture == nil {
		return result(false, "", "architecture is nil")
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.RelationTagRule{})
	validator.RegisterRule(&engine.ScenarioFQNRule{})
	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += e.Message + "\n"
		}
		return result(false, "", "Validation errors:\n"+msg)
	}

	// Export to HTML using V2 mode (Cytoscape.js + ELK layout)
	// This mode provides interactive navigation with sidebar, breadcrumbs, and documentation panels
	exporter := html.NewExporter()
	exporter.Mode = html.ModeV2 // Use V2 mode (Cytoscape.js + ELK)
	exporter.EmbedJSON = true
	exporter.Minify = true
	htmlContent, err := exporter.ExportFromArchitecture(program.Architecture)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, htmlContent, "")
}

func parseDsl(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic:", r)
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
		return result(false, "", err.Error())
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	// Add other rules as needed
	validator.RegisterRule(&engine.ScenarioFQNRule{})

	errs := validator.Validate(program)
	if len(errs) > 0 {
		// For now, we might want to return validation errors, but let's just log them
		// and proceed if possible, or fail if critical.
		// The viewer might want to show errors.
		// For now, let's fail on validation error to be safe.
		msg := ""
		for _, e := range errs {
			msg += e.Message + "\n"
		}
		return result(false, "", "Validation errors:\n"+msg)
	}

	// Convert to JSON
	archJson := converter.ConvertToJSON(program.Architecture)
	jsonBytes, err := json.Marshal(archJson)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, string(jsonBytes), "")
}

func jsonToDsl(this js.Value, args []js.Value) interface{} {
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

	return result(true, dsl, "")
}

func result(ok bool, data string, err string) interface{} {
	res := make(map[string]interface{})
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
func getDiagnostics(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in getDiagnostics:", r)
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
	return lspResult(true, string(jsonBytes), "")
}

// getSymbols returns all symbols (identifiers) in the DSL
func getSymbols(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in getSymbols:", r)
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
	return lspResult(true, string(jsonBytes), "")
}

// hover returns hover information at the given position
func hover(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in hover:", r)
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
	return lspResult(true, string(jsonBytes), "")
}

// completion returns completion suggestions at the given position
func completion(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in completion:", r)
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

	// Simple completion: return keywords
	completions := make([]map[string]interface{}, len(keywords))
	for i, kw := range keywords {
		completions[i] = map[string]interface{}{
			"label": kw,
			"kind":  "keyword",
		}
	}

	jsonBytes, _ := json.Marshal(completions)
	return lspResult(true, string(jsonBytes), "")
}

// goToDefinition returns the definition location for a symbol at the given position
func goToDefinition(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in goToDefinition:", r)
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
	return lspResult(true, string(jsonBytes), "")
}

// format formats the DSL text
func format(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in format:", r)
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
	return lspResult(true, formatted, "")
}

// score calculates the architecture score for the given DSL
func score(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in score:", r)
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

	return result(true, string(jsonBytes), "")
}

// Helper functions

func lspResult(ok bool, data interface{}, err string) interface{} {
	res := make(map[string]interface{})
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
	var symbols []map[string]interface{}

	// Extract systems
	for _, sys := range arch.Systems {
		symbols = append(symbols, map[string]interface{}{
			"name": sys.ID,
			"kind": "system",
			"line": 1, // TODO: get actual line from AST
		})
	}

	// Extract containers
	for _, cont := range arch.Containers {
		symbols = append(symbols, map[string]interface{}{
			"name": cont.ID,
			"kind": "container",
			"line": 1,
		})
	}

	// Extract components
	for _, comp := range arch.Components {
		symbols = append(symbols, map[string]interface{}{
			"name": comp.ID,
			"kind": "component",
			"line": 1,
		})
	}

	// Extract datastores
	for _, ds := range arch.DataStores {
		symbols = append(symbols, map[string]interface{}{
			"name": ds.ID,
			"kind": "datastore",
			"line": 1,
		})
	}

	// Extract persons
	for _, person := range arch.Persons {
		symbols = append(symbols, map[string]interface{}{
			"name": person.ID,
			"kind": "person",
			"line": 1,
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

	// Simple implementation: find word at position
	words := strings.Fields(lineText)
	for _, word := range words {
		if strings.Contains(lineText, word) {
			// Check if this word matches a symbol
			for _, sys := range arch.Systems {
				if sys.ID == word {
					return map[string]interface{}{
						"contents": fmt.Sprintf("System: %s\n%s", sys.ID, sys.Label),
					}
				}
			}
		}
	}

	return nil
}

// findDefinition finds the definition location for a symbol at the given position
func findDefinition(arch *language.Architecture, text string, line, column int) map[string]interface{} {
	lines := strings.Split(text, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]
	words := strings.Fields(lineText)
	for _, word := range words {
		// Check if this word matches a symbol
		for _, sys := range arch.Systems {
			if sys.ID == word {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   1, // TODO: get actual line
					"column": 1,
				}
			}
		}
	}

	return nil
}

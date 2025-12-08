//go:build js && wasm

package main

import (
    "encoding/json"
    "fmt"
    "strings"
    "syscall/js"

    "github.com/sruja-ai/sruja/pkg/diagnostics"
    "github.com/sruja-ai/sruja/pkg/engine"
    exportjson "github.com/sruja-ai/sruja/pkg/export/json"
    "github.com/sruja-ai/sruja/pkg/export/markdown"
    "github.com/sruja-ai/sruja/pkg/language"
)

func parseDSL(this js.Value, args []js.Value) interface{} {
    if len(args) < 1 {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": "missing dsl"})
    }
    dsl := args[0].String()
    parser, err := language.NewParser()
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    program, err := parser.Parse("inline.sruja", dsl)
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    exp := exportjson.NewExporter()
    out, err := exp.Export(program.Architecture)
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    return js.ValueOf(map[string]interface{}{"ok": true, "json": out})
}

func jsonToDSL(this js.Value, args []js.Value) interface{} {
    if len(args) < 1 {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": "missing json"})
    }
    var doc exportjson.ArchitectureJSON
    if err := json.Unmarshal([]byte(args[0].String()), &doc); err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    arch := exportjson.ToArchitecture(doc)
    printer := language.NewPrinter()
    dsl := printer.Print(&language.Program{Architecture: arch})
    return js.ValueOf(map[string]interface{}{"ok": true, "dsl": dsl})
}

func dslToMarkdown(this js.Value, args []js.Value) interface{} {
    if len(args) < 1 {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": "missing dsl"})
    }
    dsl := args[0].String()
    parser, err := language.NewParser()
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    program, err := parser.Parse("inline.sruja", dsl)
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    if program.Architecture == nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": "architecture is nil"})
    }
    exporter := markdown.NewExporter()
    md, err := exporter.Export(program.Architecture)
    if err != nil {
        return js.ValueOf(map[string]interface{}{"ok": false, "error": err.Error()})
    }
    return js.ValueOf(map[string]interface{}{"ok": true, "dsl": md})
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
        if len(diags) == 0 {
            return lspResult(false, nil, err.Error())
        }
    }

    // Also run validation if we have a valid program
    if program != nil && program.Architecture != nil {
        validator := engine.NewValidator()
        validator.RegisterRule(&engine.UniqueIDRule{})
        validator.RegisterRule(&engine.ValidReferenceRule{})
        validator.RegisterRule(&engine.RelationTagRule{})
        validator.RegisterRule(&engine.ScenarioFQNRule{})
        errs := validator.Validate(program)
        for _, e := range errs {
            diags = append(diags, diagnostics.Diagnostic{
                Code:     "VALIDATION_ERROR",
                Severity: diagnostics.SeverityError,
                Message:  e.Error(),
                Location: diagnostics.SourceLocation{File: "input.sruja"},
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
    line := args[1].Int()
    column := args[2].Int()

    // Get keywords and symbols for completion
    keywords := []string{"architecture", "system", "container", "component", "datastore", "queue", "person", "relation", "requirement", "adr", "library", "import", "metadata", "description", "scenario", "story", "flow"}
    
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

func main() {
    js.Global().Set("sruja_parse_dsl", js.FuncOf(parseDSL))
    js.Global().Set("sruja_json_to_dsl", js.FuncOf(jsonToDSL))
    js.Global().Set("sruja_dsl_to_markdown", js.FuncOf(dslToMarkdown))
    // LSP functions
    js.Global().Set("sruja_get_diagnostics", js.FuncOf(getDiagnostics))
    js.Global().Set("sruja_get_symbols", js.FuncOf(getSymbols))
    js.Global().Set("sruja_hover", js.FuncOf(hover))
    js.Global().Set("sruja_completion", js.FuncOf(completion))
    js.Global().Set("sruja_go_to_definition", js.FuncOf(goToDefinition))
    js.Global().Set("sruja_format", js.FuncOf(format))
    select {}
}

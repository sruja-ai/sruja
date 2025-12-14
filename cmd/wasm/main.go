//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"

	"github.com/sruja-ai/sruja/internal/converter"
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
		var msgBuilder strings.Builder
		msgBuilder.Grow(len(errs) * 50)
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

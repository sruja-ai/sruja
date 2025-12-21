//go:build js && wasm

package main

import (
	"fmt"
	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/engine"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
	"github.com/sruja-ai/sruja/pkg/language"
)

func parseDsl(this js.Value, args []js.Value) interface{} {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered from panic in parseDsl:", r)
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

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.ScenarioFQNRule{})
	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += e.Message + "\n"
		}
		return result(false, "", "Validation errors:\n"+msg)
	}

	exporter := jexport.NewExporter()
	output, err := exporter.Export(program)
	if err != nil {
		return result(false, "", err.Error())
	}
	return result(true, output, "")
}

func jsonToDSL(this js.Value, args []js.Value) interface{} {
	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}

	// Convert JSON back to AST
	// ConvertFromJSON removed - Architecture struct removed (old syntax no longer supported)
	// Use LikeC4 importers instead
	return result(false, "", "JSON to DSL conversion is no longer supported - Architecture struct removed. Use LikeC4 format instead")
}

func dslToMermaid(this js.Value, args []js.Value) interface{} {
	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	input := args[0].String()
	filename := "input.sruja"

	p, err := language.NewParser()
	if err != nil {
		return result(false, "", err.Error())
	}

	program, _, err := p.Parse(filename, input)
	if err != nil {
		return result(false, "", err.Error())
	}

	exporter := mermaid.NewExporter(mermaid.DefaultConfig())
	output := exporter.Export(program)
	return result(true, output, "")
}

func dslToMarkdown(this js.Value, args []js.Value) interface{} {
	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	input := args[0].String()
	filename := "input.sruja"

	p, err := language.NewParser()
	if err != nil {
		return result(false, "", err.Error())
	}

	program, _, err := p.Parse(filename, input)
	if err != nil {
		return result(false, "", err.Error())
	}

	exporter := markdown.NewExporter(markdown.DefaultOptions())
	output := exporter.Export(program)
	return result(true, output, "")
}

// Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters

func result(ok bool, data string, err string) interface{} {
	res := make(map[string]interface{})
	res["ok"] = ok
	if ok {
		res["dsl"] = data
		res["json"] = data
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

// LSP functions removed - not needed for viewer production

func main() {
	// Minimal WASM for viewer production - only parsing and JSON conversion
	// Export functions (markdown/mermaid) removed - now handled by TypeScript exporters
	// Register all functions to prevent dead-code elimination
	parseDslFn := js.FuncOf(parseDsl)
	jsonToDSLFn := js.FuncOf(jsonToDSL)
	dslToMermaidFn := js.FuncOf(dslToMermaid)
	dslToMarkdownFn := js.FuncOf(dslToMarkdown)

	js.Global().Set("sruja_parse_dsl", parseDslFn)
	js.Global().Set("sruja_json_to_dsl", jsonToDSLFn)
	js.Global().Set("sruja_dsl_to_mermaid", dslToMermaidFn)
	js.Global().Set("sruja_dsl_to_markdown", dslToMarkdownFn)

	fmt.Println("WASM Go initialized - functions registered")
	fmt.Println("Registered sruja_parse_dsl")
	fmt.Println("Registered sruja_json_to_dsl")

	// Explicitly keep function references to prevent optimization
	_ = parseDslFn
	_ = jsonToDSLFn
	_ = dslToMermaidFn
	_ = dslToMarkdownFn

	// Keep the goroutine alive - TinyGo needs this
	c := make(chan struct{})
	<-c
}

//go:build js && wasm

package main

import (
	"fmt"
	"syscall/js"

	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
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
	js.Global().Set("sruja_code_actions", js.FuncOf(codeActions))
	js.Global().Set("sruja_semantic_tokens", js.FuncOf(semanticTokens))
	js.Global().Set("sruja_document_links", js.FuncOf(documentLinks))
	js.Global().Set("sruja_folding_ranges", js.FuncOf(foldingRanges))
	js.Global().Set("sruja_score", js.FuncOf(score))
	js.Global().Set("sruja_dsl_to_mermaid", js.FuncOf(dslToMermaid))
	js.Global().Set("sruja_dsl_to_markdown", js.FuncOf(dslToMarkdown))
	js.Global().Set("sruja_dsl_to_likec4", js.FuncOf(dslToLikeC4))

	// Explicitly keep function references to prevent optimization
	_ = parseDslFn
	_ = jsonToDslFn

	<-c
}

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

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", err.Error())
	}

	// Use LikeC4 exporter to generate JSON
	exporter := jexport.NewLikeC4Exporter()
	jsonStr, err := exporter.Export(program)
	if err != nil {
		return result(false, "", err.Error())
	}

	ret = result(true, jsonStr, "")
	return
}

func jsonToDsl(this js.Value, args []js.Value) (ret interface{}) {
	if len(args) < 1 {
		return result(false, "", "invalid arguments")
	}
	return result(false, "", "JSON to DSL conversion is no longer supported - Architecture struct removed. Use LikeC4 format instead")
}

func dslToMermaid(this js.Value, args []js.Value) interface{} {
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

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", err.Error())
	}

	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
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
	if len(args) >= 2 {
		fn := args[1].String()
		if fn != "" {
			filename = fn
		}
	}

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", err.Error())
	}

	if program == nil || program.Model == nil {
		return result(false, "", "no model found")
	}

	exporter := markdown.NewExporter(markdown.DefaultOptions())
	output := exporter.Export(program)
	return result(true, output, "")
}

func dslToLikeC4(this js.Value, args []js.Value) interface{} {
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

	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return result(false, "", err.Error())
	}

	// Export directly from LikeC4 Program AST
	if program.Model == nil {
		return result(false, "", "no model found")
	}

	// Use LikeC4 exporter to generate LikeC4-compatible JSON
	exporter := jexport.NewLikeC4Exporter()
	output, err := exporter.Export(program)
	if err != nil {
		return result(false, "", err.Error())
	}

	return result(true, output, "")
}

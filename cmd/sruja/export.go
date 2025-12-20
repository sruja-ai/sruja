package main

import (
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/sruja-ai/sruja/pkg/engine"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/likec4"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo,goconst // Export logic is complex, distinct strings needed
func runExport(args []string, stdout, stderr io.Writer) int {
	exportCmd := flag.NewFlagSet("export", flag.ContinueOnError)
	exportCmd.SetOutput(stderr)

	// Define flags
	_ = exportCmd.Bool("single-file", false, "(deprecated) Generate single file (legacy)")
	_ = exportCmd.String("out", "", "(deprecated) Output directory")
	extended := exportCmd.Bool("extended", false, "Include pre-computed views in JSON output (for viewer apps)")
	_ = exportCmd.Bool("local", false, "Use local assets")
	compact := exportCmd.Bool("compact", false, "Output compact JSON without indentation (likec4 only)")

	// AI-friendly export options
	scope := exportCmd.String("scope", "", "Scope to specific element (format: type:id, e.g., system:OrderService)")
	tokenLimit := exportCmd.Int("token-limit", 0, "Limit output to approximately N tokens (0 = no limit)")
	context := exportCmd.String("context", "default", "Context type: default, code_generation, review, analysis")

	if err := exportCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing export flags: %v\n", err)
		return 1
	}

	if exportCmd.NArg() < 2 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja export <format> <file>")
		_, _ = fmt.Fprintln(stderr, "Formats: json, mermaid, markdown, likec4")
		return 1
	}

	format := exportCmd.Arg(0)
	filePath := exportCmd.Arg(1)
	info, err := os.Stat(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error accessing path: %v\n", err)
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	var program *language.Program
	if info.IsDir() {
		ws, err := p.ParseWorkspace(filePath)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Workspace Parser Error: %v\n", err)
			return 1
		}
		// Resolve references across the workspace
		engine.RunWorkspaceResolution(ws)
		program = ws.MergedProgram()
	} else {
		fileContent, err := os.ReadFile(filePath)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
			return 1
		}
		program, _, err = p.Parse(filePath, string(fileContent))
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
			return 1
		}
		// Resolve references in the single file
		engine.RunResolution(program)
	}

	if program.Model == nil {
		_, _ = fmt.Fprintf(stderr, "Error: no model found in file\n")
		return 1
	}

	var output string
	var outputBytes []byte
	switch format {
	case "json":
		exporter := jexport.NewLikeC4Exporter()
		exporter.Extended = *extended
		output, err = exporter.Export(program)
	case "markdown":
		// Parse scope if provided
		var scopeObj *markdown.Scope
		if *scope != "" {
			var err error
			scopeObj, err = markdown.ParseScope(*scope)
			if err != nil {
				_, _ = fmt.Fprintf(stderr, "Error parsing scope: %v\n", err)
				return 1
			}
		} else {
			scopeObj = &markdown.Scope{Type: "full", ID: ""}
		}

		// Parse context type
		contextType := markdown.ContextType(*context)
		switch contextType {
		case markdown.ContextDefault, markdown.ContextCodeGeneration,
			markdown.ContextReview, markdown.ContextAnalysis:
			// Valid
		default:
			_, _ = fmt.Fprintf(stderr, "Invalid context type: %s (expected: default, code_generation, review, analysis)\n", *context)
			return 1
		}

		// Create markdown exporter with options
		options := markdown.DefaultOptions()
		options.Scope = scopeObj
		options.TokenLimit = *tokenLimit
		options.Context = contextType

		exporter := markdown.NewExporter(options)
		output = exporter.Export(program)
	case "mermaid":
		// TODO: Update mermaid exporter to work with LikeC4 AST
		_, _ = fmt.Fprintf(stderr, "Error: mermaid export not yet updated for LikeC4 syntax\n")
		return 1
	case "likec4":
		exporter := likec4.NewExporter()
		if *compact {
			outputBytes, err = exporter.ExportCompact(program)
		} else {
			outputBytes, err = exporter.Export(program)
		}
		output = string(outputBytes)
	case "likec4-dsl", "c4":
		dslExporter := likec4.NewDSLExporter()
		output = dslExporter.ExportDSL(program)
	default:
		_, _ = fmt.Fprintf(stderr, "Unsupported export format: %s. Supported formats: json, mermaid, markdown, likec4, likec4-dsl, c4\n", format)
		return 1
	}

	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Export Error: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprint(stdout, output)
	return 0
}

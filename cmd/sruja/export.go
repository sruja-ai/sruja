package main

import (
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/sruja-ai/sruja/pkg/engine"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
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

	if err := exportCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing export flags: %v\n", err)
		return 1
	}

	if exportCmd.NArg() < 2 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja export <format> <file>")
		return 1
	}

	format := exportCmd.Arg(0)
	filePath := exportCmd.Arg(1)

	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	program, _, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	// Resolve references (smart resolution for short names)
	engine.RunResolution(program)

	var output string
	switch format {
	case "json":
		exporter := jexport.NewExporter()
		exporter.Extended = *extended
		output, err = exporter.Export(program.Architecture)
	case "markdown":
		_, _ = fmt.Fprintf(stderr, "Markdown export is temporarily disabled. Use the TypeScript exporter in frontend apps or request this feature to be re-enabled.\n")
		return 1
	case "mermaid":
		_, _ = fmt.Fprintf(stderr, "Mermaid export is temporarily disabled. Use the TypeScript exporter in frontend apps or request this feature to be re-enabled.\n")
		return 1
	default:
		_, _ = fmt.Fprintf(stderr, "Unsupported export format: %s. Supported formats: json\n", format)
		return 1
	}

	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Export Error: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprint(stdout, output)
	return 0
}

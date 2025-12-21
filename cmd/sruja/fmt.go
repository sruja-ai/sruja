package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/export/likec4"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runFmt(args []string, stdout, stderr io.Writer) int {
	fmtCmd := flag.NewFlagSet("fmt", flag.ContinueOnError)
	fmtCmd.SetOutput(stderr)

	if err := fmtCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing fmt flags: %v", err)))
		return 1
	}

	if fmtCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, dx.Error("Usage: sruja fmt <file>"))
		return 1
	}

	filePath := fmtCmd.Arg(0)

	info, err := os.Stat(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error accessing path: %v\n", err)
		return 1
	}
	if info.IsDir() {
		_, _ = fmt.Fprintf(stderr, "Formatting directories not supported yet\n")
		return 1
	}

	content, err := os.ReadFile(filepath.Clean(filePath))
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

	if program.Model == nil {
		_, _ = fmt.Fprintf(stderr, "Error: no model found in file\n")
		return 1
	}

	// Use LikeC4 DSL exporter for formatting
	dslExporter := likec4.NewDSLExporter()
	formatted := dslExporter.ExportDSL(program)
	_, _ = fmt.Fprint(stdout, formatted)
	return 0
}

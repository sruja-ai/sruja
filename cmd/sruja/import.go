package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"

	jimport "github.com/sruja-ai/sruja/pkg/import/json"
)

func runImport(args []string, stdout, stderr io.Writer) int {
	importCmd := flag.NewFlagSet("import", flag.ContinueOnError)
	importCmd.SetOutput(stderr)

	// Define flags
	outDir := importCmd.String("out", "", "Output directory")

	if err := importCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing import flags: %v\n", err)
		return 1
	}

	if importCmd.NArg() < 2 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja import <format> <file>")
		return 1
	}

	format := importCmd.Arg(0)
	filePath := importCmd.Arg(1)

	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return 1
	}

	switch format {
	case "json":
		converter := jimport.NewConverter()
		// Default to single file output for now, unless we want to expose multiple file option
		files, err := converter.ToDSL(content, jimport.OutputFormatSingleFile)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Import Error: %v\n", err)
			return 1
		}

		if *outDir != "" {
			if err := os.MkdirAll(*outDir, 0o755); err != nil {
				_, _ = fmt.Fprintf(stderr, "Error creating output directory: %v\n", err)
				return 1
			}
			for _, f := range files {
				outPath := filepath.Join(*outDir, f.Path)
				if err := os.WriteFile(outPath, []byte(f.Content), 0o644); err != nil { //nolint:gosec // consistent with other generated files
					_, _ = fmt.Fprintf(stderr, "Error writing output file %s: %v\n", f.Path, err)
					return 1
				}
			}
			_, _ = fmt.Fprintln(stdout, "Import successful.")
		} else if len(files) > 0 {
			// Print content of the first file (usually the main architecture file) to stdout
			_, _ = fmt.Fprint(stdout, files[0].Content)
		}
	default:
		_, _ = fmt.Fprintf(stderr, "Unsupported import format: %s. Supported formats: json\n", format)
		return 1
	}

	return 0
}




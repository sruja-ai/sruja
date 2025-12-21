package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
)

func runImport(args []string, stdout, stderr io.Writer) int {
	importCmd := flag.NewFlagSet("import", flag.ContinueOnError)
	importCmd.SetOutput(stderr)

	// Define flags

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

	info, err := os.Stat(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error accessing path: %v\n", err)
		return 1
	}
	if info.IsDir() {
		_, _ = fmt.Fprintf(stderr, "Import does not support directories yet\n")
		return 1
	}

	content, err := os.ReadFile(filepath.Clean(filePath))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return 1
	}

	switch format {
	case "json":
		// Try Sruja's internal JSON format first (ArchitectureJSON)
		var arch jsonexport.ArchitectureJSON
		if err := json.Unmarshal(content, &arch); err == nil && len(arch.Architecture.Systems) > 0 {
			for _, sys := range arch.Architecture.Systems {
				_, _ = fmt.Fprintf(stdout, "system %s \"%s\"\n", sys.ID, sys.Label)
			}
			return 0
		}

		// Fallback to LikeC4-compatible SrujaModelDump
		var dump jsonexport.SrujaModelDump
		if err := json.Unmarshal(content, &dump); err == nil && len(dump.Elements) > 0 {
			for _, elem := range dump.Elements {
				if elem.Kind == "system" {
					_, _ = fmt.Fprintf(stdout, "system %s \"%s\"\n", elem.ID, elem.Title)
				}
			}
			return 0
		}

		// Final fallback to direct SystemJSON
		var sysJSON jsonexport.SystemJSON
		if err := json.Unmarshal(content, &sysJSON); err == nil && sysJSON.ID != "" {
			_, _ = fmt.Fprintf(stdout, "system %s \"%s\"\n", sysJSON.ID, sysJSON.Label)
			return 0
		}

		_, _ = fmt.Fprintln(stderr, "Error: Could not identify architecture in JSON")
		return 1
	default:
		_, _ = fmt.Fprintf(stderr, "Unsupported import format: %s. Supported formats: json\n", format)
		return 1
	}
}

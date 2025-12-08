package main

import (
	"fmt"
	"os"

	"io"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

const srujaFileExt = ".sruja"

// findSrujaFile finds a .sruja file in the current directory if filePath is empty
func findSrujaFile(filePath string) string {
	if filePath != "" {
		return filePath
	}
	files, err := os.ReadDir(".")
	if err != nil {
		return ""
	}
	for _, file := range files {
		if !file.IsDir() && len(file.Name()) > len(srujaFileExt) && file.Name()[len(file.Name())-len(srujaFileExt):] == srujaFileExt {
			return file.Name()
		}
	}
	return ""
}

// parseArchitectureFile parses an architecture file and returns the program
func parseArchitectureFile(filePath string, stderr io.Writer) (*language.Program, error) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return nil, err
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return nil, err
	}

	program, diags, err := p.Parse(filePath, string(content))

	// Print diagnostics
	if len(diags) > 0 {
		//nolint:gocritic // Copying diagnostics is acceptable here
		for _, d := range diags {
			_, _ = fmt.Fprintln(stderr, diagnostics.FormatDiagnostic(d))
		}
	}

	if err != nil {
		// Critical error
		_, _ = fmt.Fprintf(stderr, "Parser Critical Error: %v\n", err)
		return nil, err
	}

	// If we have error diagnostics but no critical error, we might still want to fail
	// depending on strictness. For now, if we have syntax errors (which prevent AST generation),
	// program will be nil.
	if program == nil {
		return nil, fmt.Errorf("parsing failed with %d errors", len(diags))
	}

	return program, nil
}

func main() {
	os.Exit(Run(os.Args[1:], os.Stdout, os.Stderr))
}

func Run(args []string, stdout, stderr io.Writer) int {
	rootCmd.SetArgs(args)
	rootCmd.SetOut(stdout)
	rootCmd.SetErr(stderr)

	if err := rootCmd.Execute(); err != nil {
		_, _ = fmt.Fprintln(stderr, err)
		return 1
	}
	return 0
}

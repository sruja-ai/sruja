package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {
	examplesDir := "examples"
	if len(os.Args) > 1 {
		examplesDir = os.Args[1]
	}

	var allFiles []string
	err := filepath.Walk(examplesDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(path, ".sruja") {
			allFiles = append(allFiles, path)
		}
		return nil
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to walk examples directory: %v\n", err)
		os.Exit(1)
	}

	parser, err := language.NewParser()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to create parser: %v\n", err)
		os.Exit(1)
	}

	var failed []string
	var passed int

	for _, path := range allFiles {
		// Skip invalid examples intentionally
		if strings.Contains(path, "invalid") || strings.Contains(path, "violation") {
			continue
		}

		content, err := os.ReadFile(path) //nolint:gosec // path is trusted
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to read file %s: %v\n", path, err)
			failed = append(failed, path)
			continue
		}

		_, diags, err := parser.Parse(path, string(content))
		if err != nil {
			fmt.Fprintf(os.Stderr, "❌ Failed to parse %s: %v\n", path, err)
			failed = append(failed, path)
			continue
		}

		if len(diags) > 0 {
			hasErrors := false
			for _, diag := range diags {
				if diag.Severity == "error" {
					fmt.Fprintf(os.Stderr, "❌ %s: %s\n", path, diag.Message)
					hasErrors = true
				}
			}
			if hasErrors {
				failed = append(failed, path)
				continue
			}
		}

		passed++
		fmt.Printf("✅ %s\n", path)
	}

	fmt.Printf("\n=== Summary ===\n")
	fmt.Printf("Passed: %d\n", passed)
	fmt.Printf("Failed: %d\n", len(failed))

	if len(failed) > 0 {
		fmt.Printf("\nFailed files:\n")
		for _, f := range failed {
			fmt.Printf("  - %s\n", f)
		}
		os.Exit(1)
	}

	fmt.Printf("\n✅ All examples compile successfully!\n")
}

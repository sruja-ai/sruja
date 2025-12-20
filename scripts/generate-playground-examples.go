// scripts/generate-playground-examples.go
// Generates TypeScript examples file from .sruja files based on manifest
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type Manifest struct {
	Examples []ExampleEntry `json:"examples"`
}

type ExampleEntry struct {
	File            string `json:"file"`
	Name            string `json:"name"`
	Order           int    `json:"order"`
	Category        string `json:"category,omitempty"`
	Description     string `json:"description,omitempty"`
	SkipPlayground  bool   `json:"skipPlayground,omitempty"`
	SkipOrphanCheck bool   `json:"skipOrphanCheck,omitempty"`
	ExpectedFailure string `json:"expectedFailure,omitempty"`
}

func main() {
	manifestPath := "examples/manifest.json"
	examplesDir := "examples"
	outputPath := "apps/website/src/lib/examples.generated.ts"

	// Read manifest
	manifestData, err := os.ReadFile(manifestPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading manifest: %v\n", err)
		os.Exit(1)
	}

	var manifest Manifest
	if err := json.Unmarshal(manifestData, &manifest); err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing manifest: %v\n", err)
		os.Exit(1)
	}

	// Filter examples for designer (exclude skipPlayground=true)
	var designerExamples []ExampleEntry
	for _, ex := range manifest.Examples {
		if !ex.SkipPlayground {
			designerExamples = append(designerExamples, ex)
		}
	}

	// Sort by order
	sort.Slice(designerExamples, func(i, j int) bool {
		return designerExamples[i].Order < designerExamples[j].Order
	})

	// Generate TypeScript file
	var sb strings.Builder
	sb.WriteString("// Auto-generated from examples/manifest.json - DO NOT EDIT MANUALLY\n")
	sb.WriteString("// Run: go run scripts/generate-playground-examples.go\n\n")
	sb.WriteString("import type { PlaygroundExample } from './types';\n\n")
	sb.WriteString("export const EXAMPLES: PlaygroundExample[] = [\n")

	for i, entry := range designerExamples {
		// Validate file name to prevent directory traversal
		if strings.Contains(entry.File, "..") || filepath.IsAbs(entry.File) {
			fmt.Fprintf(os.Stderr, "Warning: Skipping invalid file path: %s\n", entry.File)
			continue
		}
		filePath := filepath.Join(examplesDir, entry.File)
		// Clean and validate the final path
		cleanPath := filepath.Clean(filePath)
		// Ensure the cleaned path is still within examplesDir
		if !strings.HasPrefix(cleanPath, examplesDir) {
			fmt.Fprintf(os.Stderr, "Warning: Skipping file outside examples directory: %s\n", entry.File)
			continue
		}
		content, err := os.ReadFile(cleanPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Warning: Could not read %s: %v\n", cleanPath, err)
			continue
		}

		// Remove comment lines (metadata comments) for cleaner designer display
		lines := strings.Split(string(content), "\n")
		var cleanLines []string
		for _, line := range lines {
			trimmed := strings.TrimSpace(line)
			// Skip metadata comments but keep regular comments
			if strings.HasPrefix(trimmed, "// SKIP_") ||
				strings.HasPrefix(trimmed, "// EXPECTED_FAILURE:") {
				continue
			}
			cleanLines = append(cleanLines, line)
		}
		cleanContent := strings.Join(cleanLines, "\n")

		// Escape backticks and format
		escaped := strings.ReplaceAll(cleanContent, "`", "\\`")
		escaped = strings.TrimSpace(escaped)

		sb.WriteString("  {\n")
		sb.WriteString(fmt.Sprintf("    name: %q,\n", entry.Name))
		sb.WriteString("    code: `")
		sb.WriteString(escaped)
		sb.WriteString("`\n")
		sb.WriteString("  }")

		if i < len(manifest.Examples)-1 {
			sb.WriteString(",")
		}
		sb.WriteString("\n")
	}

	sb.WriteString("];\n")

	// Write output
	if err := os.WriteFile(outputPath, []byte(sb.String()), 0o644); err != nil { //nolint:gosec // generator script safe to write world-readable
		fmt.Fprintf(os.Stderr, "Error writing output: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Generated %s with %d examples (filtered from %d total)\n", outputPath, len(designerExamples), len(manifest.Examples))
}

// cmd/sruja/change.go
// Package main provides CLI commands for managing architectural changes.
package main

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// runChangeCreate creates a new change file
func runChangeCreate(changeName, requirement, owner string, stakeholders []string, stdout, stderr io.Writer) int {
	// Determine change number (find highest existing change number)
	changesDir := "changes"
	if err := os.MkdirAll(changesDir, 0o755); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating changes directory: %v\n", err)
		return 1
	}

	// Find next change number
	changeNum := findNextChangeNumber(changesDir)
	changeID := fmt.Sprintf("%03d-%s", changeNum, changeName)
	changeFile := filepath.Join(changesDir, fmt.Sprintf("%03d-%s.sruja", changeNum, changeName))

	// Build change block DSL
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("change %q {\n", changeID))
	sb.WriteString("  status \"pending\"\n")
	if requirement != "" {
		sb.WriteString(fmt.Sprintf("  requirement %q\n", requirement))
	}
	if owner != "" {
		sb.WriteString("  metadata {\n")
		sb.WriteString(fmt.Sprintf("    owner %q\n", owner))
		if len(stakeholders) > 0 {
			sb.WriteString("    stakeholders [")
			for i, s := range stakeholders {
				if i > 0 {
					sb.WriteString(", ")
				}
				sb.WriteString(fmt.Sprintf("%q", s))
			}
			sb.WriteString("]\n")
		}
		sb.WriteString("  }\n")
	}
	sb.WriteString("\n")
	sb.WriteString("  add {\n")
	sb.WriteString("    // Elements to add\n")
	sb.WriteString("  }\n")
	sb.WriteString("\n")
	sb.WriteString("  modify {\n")
	sb.WriteString("    // Elements to modify\n")
	sb.WriteString("  }\n")
	sb.WriteString("\n")
	sb.WriteString("  remove {\n")
	sb.WriteString("    // Elements to remove\n")
	sb.WriteString("  }\n")
	sb.WriteString("}\n")

	// Write file
	if err := os.WriteFile(changeFile, []byte(sb.String()), 0o644); err != nil { //nolint:gosec // consistent with other generated files
		_, _ = fmt.Fprintf(stderr, "Error writing change file: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprintf(stdout, "Created change file: %s\n", changeFile)
	return 0
}

// findNextChangeNumber finds the next available change number
func findNextChangeNumber(changesDir string) int {
	entries, err := os.ReadDir(changesDir)
	if err != nil {
		return 1
	}

	maxNum := 0
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		if !strings.HasSuffix(entry.Name(), ".sruja") {
			continue
		}
		// Extract number from filename like "001-name.sruja"
		var num int
		if _, err := fmt.Sscanf(entry.Name(), "%03d-", &num); err == nil {
			if num > maxNum {
				maxNum = num
			}
		}
	}
	return maxNum + 1
}

// runChangeValidate validates a change file
func runChangeValidate(changeFile string, stderr io.Writer) int {
	content, err := os.ReadFile(changeFile)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading change file: %v\n", err)
		return 1
	}

	parser, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	// Parse the file - ParseFile reads the file, so we use Parse directly
	_, _, err = parser.Parse(changeFile, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parse error: %v\n", err)
		return 1
	}

	// Basic validation - check that file parses successfully
	// Full ChangeBlock validation requires File-level parsing which isn't exposed yet
	// TODO: Add proper ChangeBlock validation when File-level parsing is available
	_, _ = fmt.Fprintf(os.Stdout, "Change file is valid (parsed successfully)\n")
	return 0
}

package main

import (
	"path/filepath"

	"github.com/spf13/cobra"
)

// completeSrujaFiles completes .sruja files in the current directory
func completeSrujaFiles(_ *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	if len(args) > 0 {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	files, err := filepath.Glob("*" + toComplete + "*.sruja")
	if err != nil {
		return nil, cobra.ShellCompDirectiveError
	}

	if len(files) == 0 {
		// If no matches with pattern, try all .sruja files
		files, err = filepath.Glob("*.sruja")
		if err != nil {
			return nil, cobra.ShellCompDirectiveError
		}
	}

	// Filter based on toComplete if Glob didn't handle it fully (e.g. if toComplete has path components)
	// For simplicity, we just return what Glob found for now, but we should handle directories if needed.
	// Cobra handles directory traversal if we return Default, but we want to filter extensions.
	// A better approach for file completion with extension filter:

	return files, cobra.ShellCompDirectiveNoFileComp
}

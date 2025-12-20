package language

import (
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// Workspace represents a collection of Sruja programs (files) that form a single logical system.
// It handles cross-file reference resolution and merging of architectural blocks.
type Workspace struct {
	Programs map[string]*Program      // Filename -> Program
	Diags    []diagnostics.Diagnostic // Combined diagnostics
	Aliases  map[string]string        // Alias -> Absolute Path (e.g., sruja.ai/stdlib -> ...)
}

// NewWorkspace creates a new empty workspace with default aliases.
func NewWorkspace() *Workspace {
	return &Workspace{
		Programs: make(map[string]*Program),
		Aliases: map[string]string{
			"sruja.ai/stdlib": "/Users/dilipkola/Workspace/sruja/pkg/stdlib",
		},
	}
}

// AddProgram adds a parsed program to the workspace.
func (w *Workspace) AddProgram(filename string, prog *Program, diags []diagnostics.Diagnostic) {
	w.Programs[filename] = prog
	w.Diags = append(w.Diags, diags...)
}

// MergedProgram returns a single Program that represents the union of all programs in the workspace.
// It performs a "flat" merge of items from Specification, Model, and Views blocks.
func (w *Workspace) MergedProgram() *Program {
	merged := &Program{
		Specification: &SpecificationBlock{},
		Model:         &ModelBlock{},
		Views:         &LikeC4ViewsBlock{},
	}

	for _, prog := range w.Programs {
		if prog == nil {
			continue
		}
		if prog.Specification != nil {
			merged.Specification.Items = append(merged.Specification.Items, prog.Specification.Items...)
		}
		if prog.Model != nil {
			merged.Model.Items = append(merged.Model.Items, prog.Model.Items...)
		}
		if prog.Views != nil {
			merged.Views.Items = append(merged.Views.Items, prog.Views.Items...)
		}
	}

	// If no items were added, return nil for that block to match single-file behavior
	if len(merged.Specification.Items) == 0 {
		merged.Specification = nil
	}
	if len(merged.Model.Items) == 0 {
		merged.Model = nil
	}
	if len(merged.Views.Items) == 0 {
		merged.Views = nil
	}

	return merged
}

// GetAliasPath returns the absolute path for an alias, or empty string if not found.
func (w *Workspace) GetAliasPath(alias string) string {
	if w.Aliases == nil {
		return ""
	}
	return w.Aliases[alias]
}

// IsStdLib returns true if the path/alias refers to the standard library.
func (w *Workspace) IsStdLib(path string) bool {
	return path == "sruja.ai/stdlib" || strings.Contains(path, "pkg/stdlib")
}

// ResolveRelativePath resolves an import path relative to the current file.
func (w *Workspace) ResolveRelativePath(currentFile, importPath string) string {
	if !strings.HasPrefix(importPath, ".") {
		return importPath
	}
	return filepath.Join(filepath.Dir(currentFile), importPath)
}

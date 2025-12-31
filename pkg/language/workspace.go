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
// It performs a "flat" merge of items from Specification, Model, and Views.
func (w *Workspace) MergedProgram() *Program {
	merged := &Program{
		Specification: &Specification{},
		Model:         &Model{},
		Views:         &Views{},
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

	// If no items were added, return nil matching single-file behavior
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
	return path == "sruja.ai/stdlib" ||
		strings.HasPrefix(path, "sruja.ai/stdlib/") ||
		strings.Contains(path, "pkg/stdlib")
}

// ResolveRelativePath resolves an import path relative to the current file.
func (w *Workspace) ResolveRelativePath(currentFile, importPath string) string {
	if !strings.HasPrefix(importPath, ".") {
		return importPath
	}
	return filepath.Join(filepath.Dir(currentFile), importPath)
}

// ResolveAndMergeImports processes all import statements in the workspace
// and merges specification items (kinds, tags) from imported programs.
func (w *Workspace) ResolveAndMergeImports() {
	for filename, prog := range w.Programs {
		w.mergeImportsForProgram(prog, filename, make(map[string]bool))
	}
}

// mergeImportsForProgram merges imports for a single program.
// The visited map prevents circular import loops.
func (w *Workspace) mergeImportsForProgram(prog *Program, filename string, visited map[string]bool) {
	if prog == nil || prog.Model == nil {
		return
	}

	// Prevent circular imports
	if visited[filename] {
		return
	}
	visited[filename] = true

	for _, item := range prog.Model.Items {
		if item.Import == nil {
			continue
		}

		// Resolve the import path
		importPath := item.Import.From
		resolvedPath := w.resolveImportPath(filename, importPath)

		// Find the imported program(s)
		importedSpecs := w.getSpecificationItemsFromPath(resolvedPath)
		if len(importedSpecs) == 0 {
			continue
		}

		// Merge based on import type (wildcard or named)
		if prog.Specification == nil {
			prog.Specification = &Specification{}
		}

		isWildcard := false
		requested := make(map[string]bool)
		for _, elem := range item.Import.Elements {
			if elem == "*" {
				isWildcard = true
				break
			}
			requested[elem] = true
		}

		for _, specItem := range importedSpecs {
			if isWildcard {
				// Import all
				prog.Specification.Items = append(prog.Specification.Items, specItem)
			} else {
				// Import only requested names
				name := w.getSpecificationItemName(specItem)
				if requested[name] {
					prog.Specification.Items = append(prog.Specification.Items, specItem)
				}
			}
		}
	}
}

// resolveImportPath resolves an import path to an absolute path or alias.
func (w *Workspace) resolveImportPath(currentFile, importPath string) string {
	// Check if it's an alias
	if alias, ok := w.Aliases[importPath]; ok {
		return alias
	}
	// Check for stdlib
	if w.IsStdLib(importPath) {
		return importPath
	}
	// Relative path
	if strings.HasPrefix(importPath, ".") {
		return filepath.Join(filepath.Dir(currentFile), importPath)
	}
	return importPath
}

// getSpecificationItemsFromPath returns specification items from all programs matching the path.
func (w *Workspace) getSpecificationItemsFromPath(path string) []SpecificationItem {
	var items []SpecificationItem

	for filename, prog := range w.Programs {
		if prog == nil || prog.Specification == nil {
			continue
		}

		// Match by:
		// 1. Exact match
		// 2. Path prefix (filename starts with path)
		// 3. Path contained in filename
		// 4. Both are stdlib paths
		matches := filename == path ||
			strings.HasPrefix(filename, path) ||
			strings.Contains(filename, path) ||
			(w.IsStdLib(filename) && w.IsStdLib(path))

		if matches {
			items = append(items, prog.Specification.Items...)
		}
	}

	return items
}

// getSpecificationItemName returns the name of a specification item.
func (w *Workspace) getSpecificationItemName(item SpecificationItem) string {
	if item.Element != nil {
		return item.Element.Name
	}
	if item.Tag != nil {
		return item.Tag.Name
	}
	return ""
}

// cmd/wasm/helpers.go
// Helper functions for WASM LSP operations
// Supports new LikeC4 syntax
//
//nolint:unused
package main

import (
	"regexp"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// parseToWorkspace centralizes workspace creation, import resolution, and global resolution for WASM.
func parseToWorkspace(input, filename string) (*language.Workspace, *language.Program, error) {
	p, err := language.NewParser()
	if err != nil {
		return nil, nil, err
	}

	ws := language.NewWorkspace()
	prog, diags, err := p.Parse(filename, input)
	if err != nil {
		return nil, nil, err
	}
	ws.AddProgram(filename, prog, diags)

	// Resolve imports (loads stdlib from embedded FS if referenced)
	_ = p.ResolveImports(ws, prog, filename)

	// Run global resolution across the workspace
	engine.RunWorkspaceResolution(ws)

	// Return the merged program which represents the full resolved model
	return ws, ws.MergedProgram(), nil
}

// extractSymbolsFromProgram extracts all symbols from a Program
func extractSymbolsFromProgram(program *language.Program) []map[string]interface{} {
	if program == nil || program.Model == nil {
		return nil
	}
	var symbols []map[string]interface{}
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			extractElementSymbols(item.ElementDef, "", &symbols)
		}
	}
	return symbols
}

// extractElementSymbols recursively extracts symbols from a LikeC4 element and its children
func extractElementSymbols(elem *language.LikeC4ElementDef, parentFQN string, symbols *[]map[string]interface{}) {
	id := elem.GetID()
	if id == "" {
		return
	}

	fqn := id
	if parentFQN != "" {
		fqn = parentFQN + "." + id
	}

	kind := elem.GetKind()
	*symbols = append(*symbols, map[string]interface{}{
		"name": fqn,
		"kind": kind,
		"line": elem.Pos.Line,
	})

	// Recursively process nested elements
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Element != nil {
				extractElementSymbols(bodyItem.Element, fqn, symbols)
			}
		}
	}
}

// findHoverInfoFromProgram finds hover information at the given position
func findHoverInfoFromProgram(_ *language.Program, _ string, _, _ int) map[string]interface{} {
	// TODO: Implement actual hover search
	return nil
}

// findDefinitionFromProgram finds the definition location for a symbol at the given position
func findDefinitionFromProgram(_ *language.Program, _ string, _, _ int) map[string]interface{} {
	// TODO: Implement actual definition search
	return nil
}

// findSymbolReferencesFromProgram finds all references to a symbol
func findSymbolReferencesFromProgram(_ *language.Program, _ string, _ string) []map[string]interface{} {
	// TODO: Implement actual references search
	return nil
}

// isIdentChar checks if a character is a valid identifier character
func isIdentChar(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_'
}

// performRename performs the actual rename operation
func performRename(input, oldName, newName string) string {
	result := input

	wordBoundaryPattern := `\b` + regexp.QuoteMeta(oldName) + `\b`
	result = regexp.MustCompile(wordBoundaryPattern).ReplaceAllString(result, newName)

	return result
}

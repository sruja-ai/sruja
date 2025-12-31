// Package main provides helper functions for WASM LSP operations.
//
//nolint:unused
package main

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// parseToWorkspace parses DSL text and builds a complete workspace with resolved references.
// Returns the workspace, merged program, and any error.
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
	_ = p.ResolveImports(ws, prog, filename)
	engine.RunWorkspaceResolution(ws)

	return ws, ws.MergedProgram(), nil
}

// Symbol represents a symbol extracted from the program AST.
type Symbol struct {
	Name string `json:"name"`
	Kind string `json:"kind"`
	Line int    `json:"line"`
}

// Position represents a position in a source file (0-indexed for LSP).
type Position struct {
	Line      int `json:"line"`
	Character int `json:"character"`
}

// Range represents a text range in a source file.
type Range struct {
	Start Position `json:"start"`
	End   Position `json:"end"`
}

// HoverInfo represents hover information for LSP.
type HoverInfo struct {
	Contents string `json:"contents"`
	Range    Range  `json:"range"`
}

// CompletionItem represents a completion suggestion for LSP.
type CompletionItem struct {
	Label string `json:"label"`
	Kind  string `json:"kind"`
}

// extractSymbolsFromProgram extracts all symbols from a Program.
// Returns an empty slice (not nil) if no symbols are found, for consistent JSON serialization.
func extractSymbolsFromProgram(program *language.Program) []Symbol {
	if program == nil || program.Model == nil {
		return []Symbol{}
	}
	var symbols []Symbol
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			extractElementSymbols(item.ElementDef, "", &symbols)
		}
	}
	return symbols
}

// extractElementSymbols recursively extracts symbols from a Sruja element and its children.
func extractElementSymbols(elem *language.ElementDef, parentFQN string, symbols *[]Symbol) {
	if elem == nil {
		return
	}

	id := elem.GetID()
	if id == "" {
		return
	}

	fqn := id
	if parentFQN != "" {
		fqn = parentFQN + "." + id
	}

	kind := elem.GetKind()
	*symbols = append(*symbols, Symbol{
		Name: fqn,
		Kind: kind,
		Line: elem.Pos.Line,
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

// findHoverInfoFromProgram finds hover information at a specific position.
// Line and column are 1-indexed (editor coordinates), converted to 0-indexed in the result.
func findHoverInfoFromProgram(program *language.Program, input string, line, column int) *HoverInfo {
	if program == nil || program.Model == nil {
		return nil
	}

	lines := strings.Split(input, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]
	if column < 1 || column > len(lineText) {
		return nil
	}

	// Extract word at cursor position (handles qualified names like "App.Web")
	start := column - 1
	for start > 0 && (isIdentChar(lineText[start-1]) || lineText[start-1] == '.') {
		start--
	}
	end := column - 1
	for end < len(lineText) && (isIdentChar(lineText[end]) || lineText[end] == '.') {
		end++
	}
	if start >= end {
		return nil
	}

	symbolName := lineText[start:end]
	if symbolName == "" {
		return nil
	}

	// Search AST for matching element (by ID or fully qualified name)
	var foundElem *language.ElementDef
	var searchInElements func(items []language.ModelItem, parentFQN string)
	searchInElements = func(items []language.ModelItem, parentFQN string) {
		for _, item := range items {
			if item.ElementDef != nil {
				elem := item.ElementDef
				id := elem.GetID()
				if id == "" {
					continue
				}

				fqn := id
				if parentFQN != "" {
					fqn = parentFQN + "." + id
				}

				if id == symbolName || fqn == symbolName {
					foundElem = elem
					return
				}

				body := elem.GetBody()
				if body != nil {
					for _, bodyItem := range body.Items {
						if bodyItem.Element != nil {
							tempItems := []language.ModelItem{{ElementDef: bodyItem.Element}}
							searchInElements(tempItems, fqn)
							if foundElem != nil {
								return
							}
						}
					}
				}
			}
		}
	}

	searchInElements(program.Model.Items, "")

	if foundElem == nil {
		return nil
	}

	// Build hover information
	kind := foundElem.GetKind()
	id := foundElem.GetID()
	titlePtr := foundElem.GetTitle()

	contents := []string{}
	if kind != "" {
		contents = append(contents, fmt.Sprintf("**%s**: %s", kind, id))
	}
	if titlePtr != nil && *titlePtr != "" && *titlePtr != id {
		contents = append(contents, fmt.Sprintf("**Title**: %s", *titlePtr))
	}

	// Get description and technology from body items if available
	body := foundElem.GetBody()
	if body != nil {
		for _, item := range body.Items {
			if item.Description != nil && *item.Description != "" {
				contents = append(contents, *item.Description)
			}
			if item.Technology != nil && *item.Technology != "" {
				contents = append(contents, fmt.Sprintf("**Technology**: %s", *item.Technology))
			}
		}
	}

	if len(contents) == 0 {
		return nil
	}

	// Convert 1-indexed editor coordinates to 0-indexed LSP coordinates
	return &HoverInfo{
		Contents: strings.Join(contents, "\n\n"),
		Range: Range{
			Start: Position{
				Line:      line - 1,
				Character: start,
			},
			End: Position{
				Line:      line - 1,
				Character: end,
			},
		},
	}
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

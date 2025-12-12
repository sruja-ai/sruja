// pkg/lsp/rename.go
package lsp

import (
	"context"

	"github.com/sourcegraph/go-lsp"
)

// Rename provides rename functionality for elements across the workspace
func (s *Server) Rename(_ context.Context, params lsp.RenameParams) (*lsp.WorkspaceEdit, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}

	// Get the element at the position
	pos := params.Position
	line := doc.GetLine(pos.Line)
	if pos.Character > len(line) {
		return nil, nil
	}

	// Extract the identifier at the cursor position
	identifier := extractIdentifier(line, pos.Character)
	if identifier == "" {
		return nil, nil
	}

	// Use the existing References functionality to find all occurrences
	refs, _ := s.References(context.Background(), lsp.ReferenceParams{
		TextDocumentPositionParams: lsp.TextDocumentPositionParams{
			TextDocument: params.TextDocument,
			Position:     params.Position,
		},
		Context: lsp.ReferenceContext{
			IncludeDeclaration: true,
		},
	})

	if len(refs) == 0 {
		return nil, nil
	}

	// Create text edits for all references
	changes := make(map[string][]lsp.TextEdit)
	seen := make(map[string]bool)
	for _, ref := range refs {
		// Avoid duplicates
		key := string(ref.URI) + ":" + string(rune(ref.Range.Start.Line)) + ":" + string(rune(ref.Range.Start.Character))
		if seen[key] {
			continue
		}
		seen[key] = true

		uri := string(ref.URI)
		if changes[uri] == nil {
			changes[uri] = make([]lsp.TextEdit, 0, 8)
		}
		changes[uri] = append(changes[uri], lsp.TextEdit{
			Range:   ref.Range,
			NewText: params.NewName,
		})
	}

	return &lsp.WorkspaceEdit{
		Changes: changes,
	}, nil
}

// extractIdentifier extracts the identifier at the given column position
func extractIdentifier(line string, col int) string {
	if col >= len(line) {
		return ""
	}

	// Find the start of the identifier
	start := col
	for start > 0 && (isIdentChar(line[start-1]) || line[start-1] == '.') {
		start--
	}

	// Find the end of the identifier
	end := col
	for end < len(line) && (isIdentChar(line[end]) || line[end] == '.') {
		end++
	}

	if start < end {
		return line[start:end]
	}
	return ""
}

// isIdentChar is defined in completion.go

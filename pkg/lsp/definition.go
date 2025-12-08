package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

func (s *Server) Definition(_ context.Context, params lsp.TextDocumentPositionParams) ([]lsp.Location, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	line := doc.GetLine(params.Position.Line)
	if params.Position.Character > len(line) {
		return nil, nil
	}
	start, end := wordBounds(line, params.Position.Character)
	token := line[start:end]
	if token == "" {
		return nil, nil
	}

	program := doc.EnsureParsed()
	if program == nil || program.Architecture == nil {
		return nil, nil
	}

	if uri, rng, ok := s.workspace.FindDefinition(token); ok {
		return []lsp.Location{{URI: uri, Range: rng}}, nil
	}

	// Fallback to any occurrence
	rng := findIDRange(doc, token)
	if !(rng.Start.Line == 0 && rng.End.Line == 0 && rng.Start.Character == 0 && rng.End.Character == 0) {
		return []lsp.Location{{URI: params.TextDocument.URI, Range: rng}}, nil
	}
	return nil, nil
}

//nolint:unused // Future use
func findDefinitionRange(doc *Document, id string) lsp.Range {
	// Look for lines starting with DSL declarations followed by the id
	decls := []string{"system ", "container ", "component ", "datastore ", "queue ", "person "}
	for i, line := range doc.lines {
		trimmed := strings.TrimSpace(line)
		for _, d := range decls {
			if strings.HasPrefix(trimmed, d) {
				// Check for exact id match after keyword
				rest := strings.TrimSpace(strings.TrimPrefix(trimmed, d))
				if strings.HasPrefix(rest, id) {
					// Range for the id token
					col := strings.Index(line, id)
					if col < 0 {
						col = 0
					}
					start := lsp.Position{Line: i, Character: col}
					end := lsp.Position{Line: i, Character: col + len(id)}
					return lsp.Range{Start: start, End: end}
				}
			}
		}
	}
	return lsp.Range{}
}

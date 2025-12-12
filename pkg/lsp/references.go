//nolint:gocritic // paramTypeCombine acceptable
package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

func (s *Server) References(_ context.Context, params lsp.ReferenceParams) ([]lsp.Location, error) {
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

	// Estimate capacity: assume token appears ~5 times per document on average
	docs := s.workspace.AllDocuments()
	estimatedLocs := len(docs) * 5
	locs := make([]lsp.Location, 0, estimatedLocs)
	for _, d := range docs {
		text := d.Text
		idx := 0
		for {
			i := strings.Index(text[idx:], token)
			if i < 0 {
				break
			}
			absolute := idx + i
			if isTokenBoundary(text, absolute, token) {
				rng := offsetToRange(text, absolute, len(token))
				locs = append(locs, lsp.Location{URI: d.URI, Range: rng})
			}
			idx = absolute + len(token)
		}
	}
	return locs, nil
}

func isTokenBoundary(text string, start int, token string) bool {
	before := start - 1
	after := start + len(token)
	if before >= 0 && before < len(text) {
		if isIdentChar(text[before]) {
			return false
		}
	}
	if after >= 0 && after < len(text) {
		if isIdentChar(text[after]) {
			return false
		}
	}
	return true
}

func offsetToRange(text string, offset int, length int) lsp.Range {
	line := 0
	col := 0
	for i := 0; i < offset && i < len(text); i++ {
		if text[i] == '\n' {
			line++
			col = 0
		} else {
			col++
		}
	}
	start := lsp.Position{Line: line, Character: col}
	end := lsp.Position{Line: line, Character: col + length}
	return lsp.Range{Start: start, End: end}
}

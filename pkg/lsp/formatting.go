package lsp

import (
	"context"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
)

func (s *Server) DocumentFormatting(_ context.Context, params lsp.DocumentFormattingParams) ([]lsp.TextEdit, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	p, err := language.NewParser()
	if err != nil {
		return nil, nil
	}
	program, _, err := p.Parse("format.sruja", doc.Text)
	if err != nil {
		return nil, nil
	}
	printer := &language.Printer{}
	formatted := printer.Print(program)
	return []lsp.TextEdit{{
		Range: lsp.Range{
			Start: lsp.Position{Line: 0, Character: 0},
			End:   lsp.Position{Line: 999999, Character: 0},
		},
		NewText: formatted,
	}}, nil
}

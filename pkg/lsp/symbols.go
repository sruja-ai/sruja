package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

//nolint:funlen // Logic requires length
func (s *Server) DocumentSymbols(_ context.Context, params lsp.DocumentSymbolParams) ([]lsp.SymbolInformation, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	program := doc.EnsureParsed()
	if program == nil || program.Architecture == nil {
		return nil, nil
	}

	arch := program.Architecture
	var symbols []lsp.SymbolInformation //nolint:prealloc // Size calculation is complex due to nested structures

	archRange := fullDocRange(doc)
	symbols = append(symbols, lsp.SymbolInformation{
		Name:          arch.Name,
		Kind:          lsp.SKNamespace,
		Location:      lsp.Location{URI: params.TextDocument.URI, Range: archRange},
		ContainerName: "architecture",
	})

	for _, sys := range arch.Systems {
		sRange := findIDRange(doc, sys.ID)
		sSel := sRange
		symbols = append(symbols, lsp.SymbolInformation{
			Name:          sys.ID,
			Kind:          lsp.SKClass,
			Location:      lsp.Location{URI: params.TextDocument.URI, Range: sSel},
			ContainerName: arch.Name,
		})
		var childrenContainer = sys.ID
		for _, c := range sys.Containers {
			cRange := findIDRange(doc, c.ID)
			symbols = append(symbols, lsp.SymbolInformation{
				Name:          c.ID,
				Kind:          lsp.SKModule,
				Location:      lsp.Location{URI: params.TextDocument.URI, Range: cRange},
				ContainerName: childrenContainer,
			})
			for _, comp := range c.Components {
				compRange := findIDRange(doc, comp.ID)
				symbols = append(symbols, lsp.SymbolInformation{
					Name:          comp.ID,
					Kind:          lsp.SKFunction,
					Location:      lsp.Location{URI: params.TextDocument.URI, Range: compRange},
					ContainerName: c.ID,
				})
			}
			for _, ds := range c.DataStores {
				dsRange := findIDRange(doc, ds.ID)
				symbols = append(symbols, lsp.SymbolInformation{
					Name:          ds.ID,
					Kind:          lsp.SKStruct,
					Location:      lsp.Location{URI: params.TextDocument.URI, Range: dsRange},
					ContainerName: c.ID,
				})
			}
			for _, q := range c.Queues {
				qRange := findIDRange(doc, q.ID)
				symbols = append(symbols, lsp.SymbolInformation{
					Name:          q.ID,
					Kind:          lsp.SKEnum,
					Location:      lsp.Location{URI: params.TextDocument.URI, Range: qRange},
					ContainerName: c.ID,
				})
			}
		}
		for _, comp := range sys.Components {
			compRange := findIDRange(doc, comp.ID)
			symbols = append(symbols, lsp.SymbolInformation{
				Name:          comp.ID,
				Kind:          lsp.SKFunction,
				Location:      lsp.Location{URI: params.TextDocument.URI, Range: compRange},
				ContainerName: sys.ID,
			})
		}
		for _, ds := range sys.DataStores {
			dsRange := findIDRange(doc, ds.ID)
			symbols = append(symbols, lsp.SymbolInformation{
				Name:          ds.ID,
				Kind:          lsp.SKStruct,
				Location:      lsp.Location{URI: params.TextDocument.URI, Range: dsRange},
				ContainerName: sys.ID,
			})
		}
		for _, q := range sys.Queues {
			qRange := findIDRange(doc, q.ID)
			symbols = append(symbols, lsp.SymbolInformation{
				Name:          q.ID,
				Kind:          lsp.SKEnum,
				Location:      lsp.Location{URI: params.TextDocument.URI, Range: qRange},
				ContainerName: sys.ID,
			})
		}
	}

	for _, p := range arch.Persons {
		pRange := findIDRange(doc, p.ID)
		symbols = append(symbols, lsp.SymbolInformation{
			Name:          p.ID,
			Kind:          lsp.SKVariable,
			Location:      lsp.Location{URI: params.TextDocument.URI, Range: pRange},
			ContainerName: arch.Name,
		})
	}
	return symbols, nil
}

func fullDocRange(doc *Document) lsp.Range {
	return lsp.Range{Start: lsp.Position{Line: 0, Character: 0}, End: lsp.Position{Line: len(doc.lines), Character: 0}}
}

func findIDRange(doc *Document, id string) lsp.Range {
	if id == "" {
		return fullDocRange(doc)
	}
	return findTextRange(doc, id)
}

//nolint:unused // Future use
func findNameRange(doc *Document, name string) lsp.Range {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return fullDocRange(doc)
	}
	return findTextRange(doc, trimmed)
}

func findTextRange(doc *Document, text string) lsp.Range {
	idx := strings.Index(doc.Text, text)
	if idx < 0 {
		return fullDocRange(doc)
	}
	line := 0
	col := 0
	for i := 0; i < idx && i < len(doc.Text); i++ {
		if doc.Text[i] == '\n' {
			line++
			col = 0
		} else {
			col++
		}
	}
	start := lsp.Position{Line: line, Character: col}
	end := lsp.Position{Line: line, Character: col + len(text)}
	return lsp.Range{Start: start, End: end}
}

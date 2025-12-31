package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen // Logic requires length
func (s *Server) DocumentSymbols(_ context.Context, params lsp.DocumentSymbolParams) ([]lsp.SymbolInformation, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	program := doc.EnsureParsed()
	if program == nil || program.Model == nil {
		return nil, nil
	}

	var symbols []lsp.SymbolInformation

	// Extract symbols from Sruja Model
	var extractSymbols func(elem *language.ElementDef, containerName string)
	extractSymbols = func(elem *language.ElementDef, containerName string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		kind := elem.GetKind()
		loc := elem.Location()

		// Map element kind to LSP symbol kind
		var lspKind lsp.SymbolKind
		switch kind {
		case "system", "System":
			lspKind = lsp.SKClass
		case "container", "Container":
			lspKind = lsp.SKModule
		case "component", "Component":
			lspKind = lsp.SKFunction
		case "database", "Database", "datastore", "DataStore":
			lspKind = lsp.SKStruct
		case "queue", "Queue":
			lspKind = lsp.SKEnum
		case "person", "Person", "actor", "Actor", "user", "User":
			lspKind = lsp.SKInterface
		case "adr", "Adr", "ADR":
			lspKind = lsp.SKString
		case "requirement", "Requirement":
			lspKind = lsp.SKProperty
		case "policy", "Policy":
			lspKind = lsp.SKConstant
		case "scenario", "Scenario", "story", "Story":
			lspKind = lsp.SKEvent
		case "flow", "Flow":
			lspKind = lsp.SKMethod
		default:
			lspKind = lsp.SKVariable
		}

		// Find range in document
		sRange := findIDRange(doc, id)
		if sRange.Start.Line == 0 && sRange.End.Line == 0 {
			// Fallback to position from AST
			sRange = lsp.Range{
				Start: lsp.Position{Line: int(loc.Line) - 1, Character: int(loc.Column) - 1},
				End:   lsp.Position{Line: int(loc.Line) - 1, Character: int(loc.Column) - 1 + len(id)},
			}
		}

		symbols = append(symbols, lsp.SymbolInformation{
			Name:          id,
			Kind:          lspKind,
			Location:      lsp.Location{URI: params.TextDocument.URI, Range: sRange},
			ContainerName: containerName,
		})

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					extractSymbols(bodyItem.Element, id)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			extractSymbols(item.ElementDef, "")
		}
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

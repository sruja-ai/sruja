package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

func (s *Server) WorkspaceSymbols(_ context.Context, params lsp.WorkspaceSymbolParams) ([]lsp.SymbolInformation, error) {
	query := strings.TrimSpace(params.Query)
	var out []lsp.SymbolInformation
	for _, d := range s.workspace.AllDocuments() {
		for id, r := range d.defs {
			if query == "" || strings.Contains(id, query) {
				kind := d.defKinds[id]
				container := d.defContainers[id]
				out = append(out, lsp.SymbolInformation{
					Name:          id,
					Kind:          kind,
					Location:      lsp.Location{URI: d.URI, Range: r},
					ContainerName: container,
				})
			}
		}
	}
	return out, nil
}

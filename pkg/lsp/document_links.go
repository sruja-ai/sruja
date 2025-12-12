// pkg/lsp/document_links.go
package lsp

import (
	"context"
	"path/filepath"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

// DocumentLinkParams represents the parameters for the document link request.
type DocumentLinkParams struct {
	TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
}

// DocumentLink represents a link in a document.
type DocumentLink struct {
	Range   lsp.Range        `json:"range"`
	Target  *lsp.DocumentURI `json:"target,omitempty"`
	Tooltip *string          `json:"tooltip,omitempty"`
}

// DocumentLinks returns document links for the text document.
// Document links are clickable links in the editor (e.g., import statements, file references).
func (s *Server) DocumentLinks(_ context.Context, params DocumentLinkParams) ([]DocumentLink, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}

	links := make([]DocumentLink, 0, 8)
	lines := strings.Split(doc.Text, "\n")

	// Note: Import statements are not currently supported in the DSL parser.
	// If import support is added in the future, we can add import link detection here.

	// Look for file references in comments or metadata
	// Pattern: file:// or references to .sruja files
	for i, line := range lines {
		// Check for file:// URLs
		if idx := strings.Index(line, "file://"); idx >= 0 {
			endIdx := idx + 7 // "file://"
			for endIdx < len(line) && !strings.ContainsAny(string(line[endIdx:endIdx+1]), " \t\n\r\"'`") {
				endIdx++
			}
			if endIdx > idx+7 {
				filePath := line[idx+7 : endIdx]
				targetURI := lsp.DocumentURI("file://" + filePath)
				links = append(links, DocumentLink{
					Range: lsp.Range{
						Start: lsp.Position{Line: i, Character: idx},
						End:   lsp.Position{Line: i, Character: endIdx},
					},
					Target: &targetURI,
				})
			}
		}

		// Check for .sruja file references in comments or strings
		if idx := strings.Index(line, ".sruja"); idx >= 0 {
			// Find the start of the filename (look backwards for path start)
			startIdx := idx
			for startIdx > 0 && !strings.ContainsAny(string(line[startIdx-1]), " \t\n\r\"'`:/\\") {
				startIdx--
			}
			// Find the end of the filename
			endIdx := idx + 6 // ".sruja"
			for endIdx < len(line) && !strings.ContainsAny(string(line[endIdx:endIdx+1]), " \t\n\r\"'`") {
				endIdx++
			}

			if startIdx < idx {
				fileName := line[startIdx:endIdx]
				// Only create link if it looks like a file path
				if strings.Contains(fileName, "/") || strings.Contains(fileName, "\\") {
					currentDir := filepath.Dir(string(params.TextDocument.URI))
					resolvedPath := filepath.Join(currentDir, fileName)
					targetURI := lsp.DocumentURI("file://" + resolvedPath)
					links = append(links, DocumentLink{
						Range: lsp.Range{
							Start: lsp.Position{Line: i, Character: startIdx},
							End:   lsp.Position{Line: i, Character: endIdx},
						},
						Target: &targetURI,
					})
				}
			}
		}
	}

	return links, nil
}

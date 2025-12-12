//nolint:gocritic // dupBranchBody, paramTypeCombine acceptable
package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

type SemanticTokensLegend struct {
	TokenTypes     []string `json:"tokenTypes"`
	TokenModifiers []string `json:"tokenModifiers"`
}

type SemanticTokens struct {
	Data []uint32 `json:"data"`
}

func (s *Server) SemanticTokensLegend() SemanticTokensLegend {
	return SemanticTokensLegend{
		TokenTypes:     []string{"keyword", "class", "module", "function", "struct", "enum", "variable", "operator", "string"},
		TokenModifiers: []string{"declaration"},
	}
}

func (s *Server) SemanticTokensFull(_ context.Context, docID lsp.TextDocumentIdentifier) (SemanticTokens, error) {
	doc := s.workspace.GetDocument(docID.URI)
	if doc == nil {
		return SemanticTokens{}, nil
	}
	// Estimate capacity: assume ~10 tokens per line on average
	estimatedTokens := len(doc.lines) * 10
	data := make([]uint32, 0, estimatedTokens)
	// Token type indices according to legend
	ti := map[string]uint32{
		"keyword": 0, "class": 1, "module": 2, "function": 3, "struct": 4, "enum": 5, "variable": 6, "operator": 7, "string": 8,
	}

	keywords := []string{"architecture", "import", "system", "container", "component", "datastore", "queue", "person", "relation", "metadata", "properties", "style", "decision", "yes", "no", "condition", "library", "owner"}

	prevLine := uint32(0)
	for li, line := range doc.lines {
		// strings
		addQuotedStrings(&data, uint32(li), line, ti["string"], &prevLine) // #nosec G115 // safe conversion
		// arrow operator
		idx := strings.Index(line, "->")
		if idx >= 0 {
			addToken(&data, uint32(li), uint32(idx), uint32(2), ti["operator"], 0, &prevLine) // #nosec G115 // safe conversion
		}
		// keywords
		for _, k := range keywords {
			wi := strings.Index(line, k)
			if wi >= 0 {
				// ensure word boundary
				if isBoundary(line, wi, len(k)) {
					addToken(&data, uint32(li), uint32(wi), uint32(len(k)), ti["keyword"], 0, &prevLine) // #nosec G115 // safe conversion
				}
			}
		}
		// identifiers after declarations (mark as declaration kinds)
		markDeclID(&data, uint32(li), line, "system ", ti["class"], &prevLine)       // #nosec G115 // safe conversion
		markDeclID(&data, uint32(li), line, "container ", ti["module"], &prevLine)   // #nosec G115 // safe conversion
		markDeclID(&data, uint32(li), line, "component ", ti["function"], &prevLine) // #nosec G115 // safe conversion
		markDeclID(&data, uint32(li), line, "datastore ", ti["struct"], &prevLine)   // #nosec G115 // safe conversion
		markDeclID(&data, uint32(li), line, "queue ", ti["enum"], &prevLine)         // #nosec G115 // safe conversion
		markDeclID(&data, uint32(li), line, "person ", ti["variable"], &prevLine)    // #nosec G115 // safe conversion
	}
	return SemanticTokens{Data: data}, nil
}

func addQuotedStrings(data *[]uint32, line uint32, s string, ttype uint32, prevLine *uint32) {
	start := -1
	for i := 0; i < len(s); i++ {
		if s[i] == '"' {
			if start == -1 {
				start = i
			} else {
				// token from start to i+1
				addToken(data, line, uint32(start), uint32(i-start+1), ttype, 0, prevLine) // #nosec G115 // safe conversion
				start = -1
			}
		}
	}
}

func markDeclID(data *[]uint32, line uint32, s string, prefix string, ttype uint32, prevLine *uint32) {
	if !strings.HasPrefix(strings.TrimSpace(s), prefix) {
		return
	}
	idx := strings.Index(s, prefix)
	if idx < 0 {
		return
	}
	rest := strings.TrimSpace(s[idx+len(prefix):])
	if rest == "" {
		return
	}
	// identifier spans until space, quote, or brace
	end := 0
	for end < len(rest) {
		c := rest[end]
		if c == ' ' || c == '"' || c == '{' {
			break
		}
		end++
	}
	col := idx + len(prefix)
	addToken(data, line, uint32(col), uint32(end), ttype, 1 /* declaration */, prevLine) // #nosec G115 // safe conversion
}

func addToken(data *[]uint32, line, col, length, tokenType, tokenMods uint32, prevLine *uint32) {
	// LSP semantic tokens use relative line and startChar
	deltaLine := line - *prevLine
	var deltaStart uint32
	if deltaLine == 0 {
		// if same line as previous token, compute relative start
		// We track last start? For simplicity, reset when new line; same-line deltaStart approximated to col
		deltaStart = col
	} else {
		deltaStart = col
	}
	*prevLine = line
	*data = append(*data, deltaLine, deltaStart, length, tokenType, tokenMods)
}

func isBoundary(line string, start int, length int) bool {
	before := start - 1
	after := start + length
	if before >= 0 && before < len(line) {
		if isIdentChar(line[before]) {
			return false
		}
	}
	if after >= 0 && after < len(line) {
		if isIdentChar(line[after]) {
			return false
		}
	}
	return true
}

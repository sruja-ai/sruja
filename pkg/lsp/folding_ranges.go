// pkg/lsp/folding_ranges.go
package lsp

import (
	"context"
	"math"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
)

// FoldingRangeParams represents the parameters for the folding range request.
type FoldingRangeParams struct {
	TextDocument lsp.TextDocumentIdentifier `json:"textDocument"`
}

// FoldingRange represents a folding range in a document.
type FoldingRange struct {
	StartLine      uint32  `json:"startLine"`
	StartCharacter *uint32 `json:"startCharacter,omitempty"`
	EndLine        uint32  `json:"endLine"`
	EndCharacter   *uint32 `json:"endCharacter,omitempty"`
	Kind           *string `json:"kind,omitempty"`
}

// FoldingRangeKind constants
const (
	FoldingRangeKindComment = "comment"
	FoldingRangeKindImports = "imports"
	FoldingRangeKindRegion  = "region"
)

// FoldingRanges returns folding ranges for the document.
// Folding ranges allow users to collapse/expand code blocks in the editor.
func (s *Server) FoldingRanges(_ context.Context, params FoldingRangeParams) ([]FoldingRange, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}

	// Parse the document to get AST
	parser, err := language.NewParser()
	if err != nil {
		return nil, nil
	}

	program, _, err := parser.Parse(string(params.TextDocument.URI), doc.Text)
	if err != nil || program == nil {
		// If parsing fails, still try to provide basic folding based on braces
		return s.foldingRangesFromText(doc.Text), nil
	}

	ranges := make([]FoldingRange, 0, 32)

	// Add folding ranges for LikeC4 Model block
	if program.Model != nil {
		// Add folding ranges for elements in Model
		var addElementRanges func(elem *language.LikeC4ElementDef)
		addElementRanges = func(elem *language.LikeC4ElementDef) {
			if elem == nil {
				return
			}

			loc := elem.Location()
			body := elem.GetBody()
			if loc.Line > 0 && body != nil {
				// Find the closing brace of the element block
				endLine := s.findBlockEnd(doc.Text, loc.Line-1, loc.Column-1)
				if endLine > loc.Line {
					startCol := toUint32(loc.Column - 1)
					kind := FoldingRangeKindRegion
					ranges = append(ranges, FoldingRange{
						StartLine:      toUint32(loc.Line - 1),
						StartCharacter: &startCol,
						EndLine:        toUint32(endLine - 1),
						Kind:           &kind,
					})
				}

				// Recurse into nested elements
				for _, bodyItem := range body.Items {
					if bodyItem.Element != nil {
						addElementRanges(bodyItem.Element)
					}
				}
			}
		}

		// Process all top-level elements
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				addElementRanges(item.ElementDef)
			}
			// Also add folding ranges for scenarios/flows
			if item.Scenario != nil {
				loc := item.Scenario.Location()
				if loc.Line > 0 {
					endLine := s.findBlockEnd(doc.Text, loc.Line-1, loc.Column-1)
					if endLine > loc.Line {
						startCol := toUint32(loc.Column - 1)
						kind := FoldingRangeKindRegion
						ranges = append(ranges, FoldingRange{
							StartLine:      toUint32(loc.Line - 1),
							StartCharacter: &startCol,
							EndLine:        toUint32(endLine - 1),
							Kind:           &kind,
						})
					}
				}
			}
			if item.Flow != nil {
				loc := item.Flow.Location()
				if loc.Line > 0 {
					endLine := s.findBlockEnd(doc.Text, loc.Line-1, loc.Column-1)
					if endLine > loc.Line {
						startCol := toUint32(loc.Column - 1)
						kind := FoldingRangeKindRegion
						ranges = append(ranges, FoldingRange{
							StartLine:      toUint32(loc.Line - 1),
							StartCharacter: &startCol,
							EndLine:        toUint32(endLine - 1),
							Kind:           &kind,
						})
					}
				}
			}
		}
	}

	return ranges, nil
}

// findBlockEnd finds the line number of the closing brace for a block starting at the given position.
// Returns 0 if not found.
func (s *Server) findBlockEnd(text string, startLine, startCol int) int {
	lines := strings.Split(text, "\n")
	if startLine >= len(lines) {
		return 0
	}

	// Find the opening brace on the start line or subsequent lines
	braceCol := -1
	searchLine := startLine
	for searchLine < len(lines) {
		line := lines[searchLine]
		if searchLine == startLine {
			// On the start line, look for brace after the start column
			if idx := strings.Index(line[startCol:], "{"); idx >= 0 {
				braceCol = startCol + idx
				break
			}
		} else {
			// On subsequent lines, look for any opening brace
			if idx := strings.Index(line, "{"); idx >= 0 {
				braceCol = idx
				break
			}
		}
		searchLine++
	}

	if braceCol < 0 {
		return 0
	}

	// Count braces to find matching closing brace
	depth := 0
	for i := searchLine; i < len(lines); i++ {
		line := lines[i]
		startIdx := 0
		if i == searchLine {
			startIdx = braceCol + 1
		}

		for j := startIdx; j < len(line); j++ {
			char := line[j]
			if char == '{' {
				depth++
			} else if char == '}' {
				depth--
				if depth == 0 {
					return i + 1 // Return 1-based line number
				}
			}
		}
	}

	return 0
}

// foldingRangesFromText provides basic folding ranges based on brace matching
// when AST parsing fails.
func (s *Server) foldingRangesFromText(text string) []FoldingRange {
	lines := strings.Split(text, "\n")
	ranges := make([]FoldingRange, 0, 16)

	// Find all opening braces and their matching closing braces
	type braceInfo struct {
		line   int
		column int
	}

	openBraces := []braceInfo{}
	for i, line := range lines {
		for j, char := range line {
			if char == '{' {
				openBraces = append(openBraces, braceInfo{line: i, column: j})
			} else if char == '}' && len(openBraces) > 0 {
				// Found matching closing brace
				open := openBraces[len(openBraces)-1]
				openBraces = openBraces[:len(openBraces)-1]

				// Only add folding range if the block spans multiple lines
				if i > open.line {
					col := toUint32(open.column)
					kind := FoldingRangeKindRegion
					ranges = append(ranges, FoldingRange{
						StartLine:      toUint32(open.line),
						StartCharacter: &col,
						EndLine:        toUint32(i),
						Kind:           &kind,
					})
				}
			}
		}
	}

	return ranges
}

// toUint32 safely converts an int to uint32 by clamping to [0, MaxUint32].
func toUint32(v int) uint32 {
	if v <= 0 {
		return 0
	}
	if v > int(math.MaxUint32) {
		return math.MaxUint32
	}
	return uint32(v)
}

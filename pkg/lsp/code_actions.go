// pkg/lsp/code_actions.go
package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// CodeAction provides quick fixes and refactoring actions for diagnostics
// Note: Using Command-based approach as CodeAction type may not be available in this go-lsp version
func (s *Server) CodeAction(_ context.Context, params lsp.CodeActionParams) ([]lsp.Command, error) {
	var actions []lsp.Command

	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return actions, nil
	}

	// Get diagnostics for this document
	program := doc.EnsureParsed()
	if program == nil {
		return actions, nil
	}

	// Generate code actions based on diagnostics
	for _, diag := range params.Context.Diagnostics {
		action := s.generateCodeAction(doc, diag, params.TextDocument.URI)
		if action != nil {
			actions = append(actions, *action)
		}
	}

	return actions, nil
}

// generateCodeAction creates a code action for a specific diagnostic
func (s *Server) generateCodeAction(doc *Document, diag lsp.Diagnostic, uri lsp.DocumentURI) *lsp.Command {
	switch diag.Code {
	case diagnostics.CodeReferenceNotFound:
		return s.generateQuickFixForUndefinedRef(doc, diag, uri)
	case diagnostics.CodeDuplicateIdentifier:
		return s.generateQuickFixForDuplicateID(doc, diag, uri)
	case diagnostics.CodeOrphanElement:
		return s.generateQuickFixForOrphan(doc, diag, uri)
	case diagnostics.CodeUnexpectedToken:
		return s.generateQuickFixForSyntax(doc, diag, uri)
	default:
		return nil
	}
}

// generateQuickFixForUndefinedRef suggests fixes for undefined references
func (s *Server) generateQuickFixForUndefinedRef(doc *Document, diag lsp.Diagnostic, uri lsp.DocumentURI) *lsp.Command {
	// Extract the undefined element name from the message
	msg := diag.Message
	startIdx := strings.Index(msg, "'")
	if startIdx == -1 {
		return nil
	}
	endIdx := strings.Index(msg[startIdx+1:], "'")
	if endIdx == -1 {
		return nil
	}
	elementName := msg[startIdx+1 : startIdx+1+endIdx]

	// Find similar elements in the document
	program := doc.EnsureParsed()
	if program == nil || program.Architecture == nil {
		return nil
	}

	similar := s.findSimilarElementNames(elementName, program.Architecture)
	if len(similar) == 0 {
		return nil
	}

	// Create quick fix to replace with the most similar element
	bestMatch := similar[0]

	// Use Command approach for code actions
	return &lsp.Command{
		Title:     "Replace with '" + bestMatch + "'",
		Command:   "sruja.replaceElement",
		Arguments: []interface{}{string(uri), diag.Range.Start.Line, diag.Range.Start.Character, diag.Range.End.Line, diag.Range.End.Character, bestMatch},
	}
}

// generateQuickFixForDuplicateID suggests renaming for duplicate IDs
func (s *Server) generateQuickFixForDuplicateID(doc *Document, diag lsp.Diagnostic, uri lsp.DocumentURI) *lsp.Command {
	// Extract the duplicate ID from the message
	msg := diag.Message
	startIdx := strings.Index(msg, "'")
	if startIdx == -1 {
		return nil
	}
	endIdx := strings.Index(msg[startIdx+1:], "'")
	if endIdx == -1 {
		return nil
	}
	elementID := msg[startIdx+1 : startIdx+1+endIdx]

	// Suggest renaming with a suffix
	newName := elementID + "2"

	return &lsp.Command{
		Title:     "Rename to '" + newName + "'",
		Command:   "sruja.renameElement",
		Arguments: []interface{}{string(uri), diag.Range.Start.Line, diag.Range.Start.Character, diag.Range.End.Line, diag.Range.End.Character, newName},
	}
}

// generateQuickFixForOrphan suggests adding a relation for orphan elements
func (s *Server) generateQuickFixForOrphan(doc *Document, diag lsp.Diagnostic, uri lsp.DocumentURI) *lsp.Command {
	// Extract the orphan element name
	msg := diag.Message
	startIdx := strings.Index(msg, "'")
	if startIdx == -1 {
		return nil
	}
	endIdx := strings.Index(msg[startIdx+1:], "'")
	if endIdx == -1 {
		return nil
	}
	elementName := msg[startIdx+1 : startIdx+1+endIdx]

	// Get the line where the orphan is defined
	line := diag.Range.Start.Line

	// Find a good place to add a relation (after the element definition)
	// Simple heuristic: add after the current line
	insertLine := line + 1

	var relationSb strings.Builder
	relationSb.Grow(len(elementName) + 60)
	relationSb.WriteString("  // TODO: Add relation using this element\n")
	relationSb.WriteString("  // Example: SomeElement -> ")
	relationSb.WriteString(elementName)

	return &lsp.Command{
		Title:     "Add relation using '" + elementName + "'",
		Command:   "sruja.addRelation",
		Arguments: []interface{}{string(uri), insertLine, 0, relationSb.String()},
	}
}

// generateQuickFixForSyntax suggests fixes for syntax errors
func (s *Server) generateQuickFixForSyntax(doc *Document, diag lsp.Diagnostic, uri lsp.DocumentURI) *lsp.Command {
	msg := diag.Message
	// Check for common syntax errors and suggest fixes
	if strings.Contains(msg, "unexpected token") {
		// Extract the unexpected token
		startIdx := strings.Index(msg, "'")
		if startIdx != -1 {
			endIdx := strings.Index(msg[startIdx+1:], "'")
			if endIdx != -1 {
				token := msg[startIdx+1 : startIdx+1+endIdx]
				// For now, just provide a generic fix suggestion
				// More sophisticated fixes can be added based on token type
				return &lsp.Command{
					Title:     "Remove unexpected token '" + token + "'",
					Command:   "sruja.removeToken",
					Arguments: []interface{}{string(uri), diag.Range.Start.Line, diag.Range.Start.Character, diag.Range.End.Line, diag.Range.End.Character},
				}
			}
		}
	}
	return nil
}

// findSimilarElementNames finds element names similar to the given name
func (s *Server) findSimilarElementNames(name string, arch interface{}) []string {
	// Type assertion to get architecture
	archPtr, ok := arch.(*language.Architecture)
	if !ok {
		return nil
	}

	var similar []string
	nameLower := strings.ToLower(name)

	// Collect all element IDs
	allIDs := make([]string, 0, 32)

	// Add systems
	for _, sys := range archPtr.Systems {
		allIDs = append(allIDs, sys.ID)
		for _, cont := range sys.Containers {
			allIDs = append(allIDs, cont.ID)
			for _, comp := range cont.Components {
				allIDs = append(allIDs, comp.ID)
			}
		}
		for _, comp := range sys.Components {
			allIDs = append(allIDs, comp.ID)
		}
	}

	// Add top-level elements
	for _, cont := range archPtr.Containers {
		allIDs = append(allIDs, cont.ID)
	}
	for _, comp := range archPtr.Components {
		allIDs = append(allIDs, comp.ID)
	}
	for _, p := range archPtr.Persons {
		allIDs = append(allIDs, p.ID)
	}

	// Calculate similarity
	type candidate struct {
		id    string
		score float64
	}
	candidates := make([]candidate, 0, len(allIDs))

	for _, id := range allIDs {
		if id == name {
			continue // Skip exact match
		}
		score := calculateSimilarity(nameLower, strings.ToLower(id))
		if score > 0.3 {
			candidates = append(candidates, candidate{id: id, score: score})
		}
	}

	// Sort by score and return top 3
	for i := 0; i < len(candidates) && i < 3; i++ {
		maxIdx := i
		for j := i + 1; j < len(candidates); j++ {
			if candidates[j].score > candidates[maxIdx].score {
				maxIdx = j
			}
		}
		if maxIdx != i {
			candidates[i], candidates[maxIdx] = candidates[maxIdx], candidates[i]
		}
		similar = append(similar, candidates[i].id)
	}

	return similar
}

// calculateSimilarity calculates similarity between two strings
func calculateSimilarity(s1, s2 string) float64 {
	if s1 == s2 {
		return 1.0
	}
	if strings.Contains(s1, s2) || strings.Contains(s2, s1) {
		return 0.7
	}
	if len(s1) == 0 || len(s2) == 0 {
		return 0.0
	}
	matches := 0
	minLen := len(s1)
	if len(s2) < minLen {
		minLen = len(s2)
	}
	for i := 0; i < minLen; i++ {
		if s1[i] == s2[i] {
			matches++
		}
	}
	if minLen > 0 {
		return float64(matches) / float64(minLen)
	}
	return 0.0
}

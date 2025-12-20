//nolint:gocritic // paramTypeCombine, unnamedResult acceptable
package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
	"golang.org/x/text/cases"
	lang "golang.org/x/text/language"
)

func (s *Server) Hover(_ context.Context, params lsp.TextDocumentPositionParams) (*lsp.Hover, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	line := doc.GetLine(params.Position.Line)
	if params.Position.Character > len(line) {
		return nil, nil
	}
	start, end := wordBounds(line, params.Position.Character)
	word := strings.TrimSpace(line[start:end])

	program := doc.EnsureParsed()
	if program == nil || program.Model == nil {
		return nil, nil
	}

	if word != "" {
		t, label := findElementInModel(program.Model, word)
		if t != "" {
			// Build content efficiently
			var sb strings.Builder
			sb.Grow(len(t) + len(word) + len(label) + 10)
			sb.WriteString("**")
			sb.WriteString(t)
			sb.WriteString("** `")
			sb.WriteString(word)
			sb.WriteString("`\n")
			sb.WriteString(label)
			content := sb.String()
			return &lsp.Hover{Contents: []lsp.MarkedString{{Language: "markdown", Value: content}}, Range: &lsp.Range{Start: lsp.Position{Line: params.Position.Line, Character: start}, End: lsp.Position{Line: params.Position.Line, Character: end}}}, nil
		}
	}

	// If hovering over an arrow, show relation verb/label
	arrowIdx := strings.Index(line, "->")
	if arrowIdx >= 0 && params.Position.Character >= arrowIdx && params.Position.Character < arrowIdx+2 {
		// Find tokens around the arrow
		// Left token
		leftStart, leftEnd := wordBounds(line, arrowIdx)
		left := strings.TrimSpace(line[leftStart:leftEnd])
		// Right token begins after arrow
		rightPos := arrowIdx + 2
		for rightPos < len(line) && !isIdentChar(line[rightPos]) {
			rightPos++
		}
		rightStart, rightEnd := wordBounds(line, rightPos)
		right := strings.TrimSpace(line[rightStart:rightEnd])
		if left != "" && right != "" {
			verb, rlabel := findRelationInfoInModel(program.Model, left, right)
			// Build content efficiently
			var sb strings.Builder
			sb.Grow(len(left) + len(right) + len(verb) + len(rlabel) + 20)
			sb.WriteString("**Relation** `")
			sb.WriteString(left)
			sb.WriteString(" -> ")
			sb.WriteString(right)
			sb.WriteString("`\n")
			parts := filterNotEmpty([]string{verb, rlabel})
			if len(parts) > 0 {
				sb.WriteString(strings.TrimSpace(strings.Join(parts, " ")))
			}
			content := sb.String()
			rng := lsp.Range{Start: lsp.Position{Line: params.Position.Line, Character: arrowIdx}, End: lsp.Position{Line: params.Position.Line, Character: arrowIdx + 2}}
			return &lsp.Hover{Contents: []lsp.MarkedString{{Language: "markdown", Value: content}}, Range: &rng}, nil
		}
	}

	return nil, nil
}

func wordBounds(line string, pos int) (int, int) {
	if pos < 0 {
		pos = 0
	}
	if pos > len(line) {
		pos = len(line)
	}
	i := pos
	for i > 0 && isIdentChar(line[i-1]) {
		i--
	}
	j := pos
	for j < len(line) && isIdentChar(line[j]) {
		j++
	}
	return i, j
}

//nolint:gocyclo // Recursive search is complex
func findElementInModel(model *language.ModelBlock, id string) (string, string) {
	if model == nil {
		return "", ""
	}

	// Search for element in LikeC4 Model
	var findElement func(elem *language.LikeC4ElementDef, currentFQN string) (string, string)
	findElement = func(elem *language.LikeC4ElementDef, currentFQN string) (string, string) {
		if elem == nil {
			return "", ""
		}

		elemID := elem.GetID()
		if elemID == "" {
			return "", ""
		}

		fqn := elemID
		if currentFQN != "" {
			fqn = currentFQN + "." + elemID
		}

		// Check if this is the element we're looking for
		if fqn == id || elemID == id {
			kind := elem.GetKind()
			titlePtr := elem.GetTitle()
			title := elemID
			if titlePtr != nil {
				title = *titlePtr
			}
			if kind == "database" || kind == "datastore" {
				return "DataStore", title
			}
			return cases.Title(lang.Und).String(kind), title
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if kind, label := findElement(bodyItem.Element, fqn); kind != "" {
						return kind, label
					}
				}
			}
		}

		return "", ""
	}

	// Search all top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			if kind, label := findElement(item.ElementDef, ""); kind != "" {
				return kind, label
			}
		}
		// Also check scenarios, flows, ADRs, etc.
		if item.Scenario != nil && item.Scenario.ID == id {
			title := item.Scenario.ID
			if item.Scenario.Title != nil {
				title = *item.Scenario.Title
			}
			return "Scenario", title
		}
		if item.Flow != nil && item.Flow.ID == id {
			title := item.Flow.ID
			if item.Flow.Title != nil {
				title = *item.Flow.Title
			}
			return "Flow", title
		}
		if item.ADR != nil && item.ADR.ID == id {
			title := item.ADR.ID
			if item.ADR.Title != nil {
				title = *item.ADR.Title
			}
			return "ADR", title
		}
		if item.Requirement != nil && item.Requirement.ID == id {
			desc := item.Requirement.ID
			if item.Requirement.Description != nil {
				desc = *item.Requirement.Description
			}
			return "Requirement", desc
		}
	}

	return "", ""
}

// resolvedRel represents a relation with resolved FQNs
type resolvedRel struct {
	from, to    string
	verb, label string
}

//nolint:gocyclo // Relation lookup is complex
func findRelationInfoInModel(model *language.ModelBlock, from string, to string) (string, string) {
	if model == nil {
		return "", ""
	}

	var relations []resolvedRel
	var collectRelations func(elem *language.LikeC4ElementDef, currentFQN string)
	collectRelations = func(elem *language.LikeC4ElementDef, currentFQN string) {
		elemID := elem.GetID()
		fqn := elemID
		if currentFQN != "" {
			fqn = currentFQN + "." + elemID
		}

		body := elem.GetBody()
		if body == nil {
			return
		}
		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				rel := bodyItem.Relation
				var verb, label string
				if rel.Verb != nil {
					verb = *rel.Verb
				}
				if rel.Label != nil {
					label = *rel.Label
				}
				// Basic resolution: try current FQN and sibling lookups
				rFrom := rel.From.String()
				rTo := rel.To.String()

				relations = append(relations, resolvedRel{
					from: rFrom, to: rTo, verb: verb, label: label,
				})
				// Also add with context if available
				if fqn != "" {
					relations = append(relations, resolvedRel{
						from: fqn + "." + rFrom, to: fqn + "." + rTo, verb: verb, label: label,
					})
				}
			}
			if bodyItem.Element != nil {
				collectRelations(bodyItem.Element, fqn)
			}
		}
	}

	for _, item := range model.Items {
		if item.Relation != nil {
			var verb, label string
			if item.Relation.Verb != nil {
				verb = *item.Relation.Verb
			}
			if item.Relation.Label != nil {
				label = *item.Relation.Label
			}
			relations = append(relations, resolvedRel{
				from: item.Relation.From.String(),
				to:   item.Relation.To.String(),
				verb: verb, label: label,
			})
		}
		if item.ElementDef != nil {
			collectRelations(item.ElementDef, "")
		}
	}

	// Search relations
	for _, rel := range relations {
		if rel.from == from && rel.to == to {
			return rel.verb, rel.label
		}
	}

	return "", ""
}

func filterNotEmpty(items []string) []string {
	// Pre-allocate with estimated capacity
	out := make([]string, 0, len(items))
	for _, s := range items {
		if strings.TrimSpace(s) != "" {
			out = append(out, s)
		}
	}
	return out
}

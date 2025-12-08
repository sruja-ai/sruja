//nolint:gocritic // paramTypeCombine, unnamedResult acceptable
package lsp

import (
	"context"
	"fmt"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
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
	if program == nil || program.Architecture == nil {
		return nil, nil
	}

	if word != "" {
		t, label := findElement(program.Architecture, word)
		if t != "" {
			content := fmt.Sprintf("**%s** `%s`\n%s", t, word, label)
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
			verb, rlabel := findRelationInfo(program.Architecture, left, right)
			content := fmt.Sprintf("**Relation** `%s -> %s`\n%s", left, right, strings.TrimSpace(strings.Join(filterNotEmpty([]string{verb, rlabel}), " ")))
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
func findElement(arch *language.Architecture, id string) (string, string) {
	for _, s := range arch.Systems {
		if s.ID == id {
			return "System", s.Label
		}
	}
	for _, s := range arch.Systems {
		for _, c := range s.Containers {
			if c.ID == id || (s.ID+"."+c.ID) == id {
				return "Container", c.Label
			}
			for _, comp := range c.Components {
				if comp.ID == id || (s.ID+"."+c.ID+"."+comp.ID) == id {
					return "Component", comp.Label
				}
			}
		}
		for _, comp := range s.Components {
			if comp.ID == id || (s.ID+"."+comp.ID) == id {
				return "Component", comp.Label
			}
		}
		for _, ds := range s.DataStores {
			if ds.ID == id || (s.ID+"."+ds.ID) == id {
				return "DataStore", ds.Label
			}
		}
		for _, q := range s.Queues {
			if q.ID == id || (s.ID+"."+q.ID) == id {
				return "Queue", q.Label
			}
		}
	}
	for _, p := range arch.Persons {
		if p.ID == id {
			return "Person", p.Label
		}
	}
	for _, ds := range arch.DataStores {
		if ds.ID == id {
			return "DataStore", ds.Label
		}
	}
	for _, q := range arch.Queues {
		if q.ID == id {
			return "Queue", q.Label
		}
	}
	for _, lib := range arch.Libraries {
		if lib.ID == id {
			return "Library", lib.Label
		}
	}
	for _, sc := range arch.Scenarios {
		if sc.ID == id {
			return "Scenario", sc.Title
		}
	}
	for _, flow := range arch.Flows {
		if flow.ID == id {
			return "Flow", flow.Title
		}
		for _, item := range flow.Items {
			if item.Step != nil {
				if item.Step.From.String() == id || item.Step.To.String() == id {
					desc := ""
					if item.Step.Description != nil {
						desc = *item.Step.Description
					}
					return "Flow Step", fmt.Sprintf("%s -> %s: %s", item.Step.From.String(), item.Step.To.String(), desc)
				}
			}
		}
	}
	return "", ""
}

//nolint:gocyclo // Relation lookup is complex
func findRelationInfo(arch *language.Architecture, from string, to string) (string, string) {
	// search architecture-level relations first
	for _, rel := range arch.Relations {
		if rel.From.String() == from && rel.To.String() == to {
			var verb, label string
			if rel.Verb != nil {
				verb = *rel.Verb
			}
			if rel.Label != nil {
				label = *rel.Label
			}
			return verb, label
		}
	}
	for _, s := range arch.Systems {
		for _, rel := range s.Relations {
			if rel.From.String() == from && rel.To.String() == to {
				var verb, label string
				if rel.Verb != nil {
					verb = *rel.Verb
				}
				if rel.Label != nil {
					label = *rel.Label
				}
				return verb, label
			}
		}
		for _, c := range s.Containers {
			for _, rel := range c.Relations {
				if rel.From.String() == from && rel.To.String() == to {
					var verb, label string
					if rel.Verb != nil {
						verb = *rel.Verb
					}
					if rel.Label != nil {
						label = *rel.Label
					}
					return verb, label
				}
			}
		}
		for _, comp := range s.Components {
			for _, rel := range comp.Relations {
				if rel.From.String() == from && rel.To.String() == to {
					var verb, label string
					if rel.Verb != nil {
						verb = *rel.Verb
					}
					if rel.Label != nil {
						label = *rel.Label
					}
					return verb, label
				}
			}
		}
	}
	return "", ""
}

func filterNotEmpty(items []string) []string {
	var out []string
	for _, s := range items {
		if strings.TrimSpace(s) != "" {
			out = append(out, s)
		}
	}
	return out
}

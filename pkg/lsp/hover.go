//nolint:gocritic // paramTypeCombine, unnamedResult acceptable
package lsp

import (
	"context"
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
			verb, rlabel := findRelationInfo(program.Architecture, left, right)
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
func findElement(arch *language.Architecture, id string) (string, string) {
	// Helper to build qualified IDs efficiently
	buildQualifiedID := func(parts ...string) string {
		if len(parts) == 0 {
			return ""
		}
		if len(parts) == 1 {
			return parts[0]
		}
		totalLen := len(parts) - 1 // for dots
		for _, p := range parts {
			totalLen += len(p)
		}
		buf := make([]byte, 0, totalLen)
		buf = append(buf, parts[0]...)
		for i := 1; i < len(parts); i++ {
			buf = append(buf, '.')
			buf = append(buf, parts[i]...)
		}
		return string(buf)
	}

	for _, s := range arch.Systems {
		if s.ID == id {
			return "System", s.Label
		}
		for _, c := range s.Containers {
			contID := buildQualifiedID(s.ID, c.ID)
			if c.ID == id || contID == id {
				return "Container", c.Label
			}
			for _, comp := range c.Components {
				compID := buildQualifiedID(s.ID, c.ID, comp.ID)
				if comp.ID == id || compID == id {
					return "Component", comp.Label
				}
			}
		}
		for _, comp := range s.Components {
			compID := buildQualifiedID(s.ID, comp.ID)
			if comp.ID == id || compID == id {
				return "Component", comp.Label
			}
		}
		for _, ds := range s.DataStores {
			dsID := buildQualifiedID(s.ID, ds.ID)
			if ds.ID == id || dsID == id {
				return "DataStore", ds.Label
			}
		}
		for _, q := range s.Queues {
			qID := buildQualifiedID(s.ID, q.ID)
			if q.ID == id || qID == id {
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
					// Build description efficiently
					var sb strings.Builder
					sb.Grow(len(item.Step.From.String()) + len(item.Step.To.String()) + len(desc) + 10)
					sb.WriteString(item.Step.From.String())
					sb.WriteString(" -> ")
					sb.WriteString(item.Step.To.String())
					if desc != "" {
						sb.WriteString(": ")
						sb.WriteString(desc)
					}
					return "Flow Step", sb.String()
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
	// Pre-allocate with estimated capacity
	out := make([]string, 0, len(items))
	for _, s := range items {
		if strings.TrimSpace(s) != "" {
			out = append(out, s)
		}
	}
	return out
}

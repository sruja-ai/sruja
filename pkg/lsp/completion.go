package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
)

var keywordList = []string{
	"architecture", "import", "system", "container", "component", "datastore", "queue", "person", "relation", "description", "metadata", "properties", "style",
	"scenario", "story", "flow", "decision", "yes", "no", "condition", "library", "owner",
}

//nolint:funlen,gocyclo // Completion logic is complex and requires length
func (s *Server) Completion(_ context.Context, params lsp.CompletionParams) (*lsp.CompletionList, error) {
	doc := s.workspace.GetDocument(params.TextDocument.URI)
	if doc == nil {
		return nil, nil
	}
	line := doc.GetLine(params.Position.Line)
	if params.Position.Character > len(line) {
		return nil, nil
	}
	before := line[:params.Position.Character]
	token := lastToken(before)
	addItem := func(items *[]lsp.CompletionItem, label string, kind lsp.CompletionItemKind) {
		*items = append(*items, lsp.CompletionItem{Label: label, Kind: kind})
	}

	program := doc.EnsureParsed()
	// Pre-allocate with estimated capacity
	estimatedItems := len(keywordList) + 32
	items := make([]lsp.CompletionItem, 0, estimatedItems)

	for _, k := range keywordList {
		if token == "" || strings.HasPrefix(k, token) {
			addItem(&items, k, lsp.CIKKeyword)
		}
	}

	if program != nil && program.Architecture != nil {
		arch := program.Architecture
		// Estimate capacity: typically many completion items
		estimatedItems := len(arch.Systems) * 10
		if estimatedItems < 32 {
			estimatedItems = 32
		}
		seen := make(map[string]bool, estimatedItems)

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

		add := func(id string) {
			if id == "" || seen[id] {
				return
			}
			if token == "" || strings.HasPrefix(id, token) {
				addItem(&items, id, lsp.CIKText)
				seen[id] = true
			}
		}
		for _, s := range arch.Systems {
			add(s.ID)
			for _, c := range s.Containers {
				add(c.ID)
				// Qualified container
				add(buildQualifiedID(s.ID, c.ID))
				for _, comp := range c.Components {
					add(comp.ID)
					add(buildQualifiedID(s.ID, c.ID, comp.ID))
				}
				for _, ds := range c.DataStores {
					add(ds.ID)
					add(buildQualifiedID(s.ID, c.ID, ds.ID))
				}
				for _, q := range c.Queues {
					add(q.ID)
					add(buildQualifiedID(s.ID, c.ID, q.ID))
				}
			}
			for _, comp := range s.Components {
				add(comp.ID)
				add(buildQualifiedID(s.ID, comp.ID))
			}
			for _, ds := range s.DataStores {
				add(ds.ID)
				add(buildQualifiedID(s.ID, ds.ID))
			}
			for _, q := range s.Queues {
				add(q.ID)
				add(buildQualifiedID(s.ID, q.ID))
			}
		}
		for _, p := range arch.Persons {
			add(p.ID)
		}
		for _, ds := range arch.DataStores {
			add(ds.ID)
		}
		for _, q := range arch.Queues {
			add(q.ID)
		}
		// Scenario and flow IDs
		for _, sc := range arch.Scenarios {
			add(sc.ID)
		}
		for _, fl := range arch.Flows {
			add(fl.ID)
		}
	}

	// Include workspace-level declarations
	if s.workspace != nil {
		for _, d := range s.workspace.AllDocuments() {
			for id := range d.defs {
				if token == "" || strings.HasPrefix(id, token) {
					addItem(&items, id, lsp.CIKText)
				}
			}
		}
	}

	// Heuristic: after an arrow and target identifier, suggest common verbs
	if strings.Contains(before, "->") {
		idx := strings.Index(before, "->")
		after := before[idx+2:]
		// if after contains an identifier and some trailing space, we are likely at verb position
		ident := lastToken(after)
		if ident != "" && (len(after) > len(ident)) {
			for _, v := range []string{"reads", "writes", "calls", "uses", "publishes"} {
				addItem(&items, v, lsp.CIKFunction)
			}
		}
	}

	return &lsp.CompletionList{IsIncomplete: false, Items: items}, nil
}

func lastToken(s string) string {
	i := len(s) - 1
	for i >= 0 && !isIdentChar(s[i]) {
		i--
	}
	if i < 0 {
		return ""
	}
	j := i
	for j >= 0 && isIdentChar(s[j]) {
		j--
	}
	return strings.TrimSpace(s[j+1 : i+1])
}

func isIdentChar(b byte) bool {
	if b == '_' || b == '-' {
		return true
	}
	if b >= 'a' && b <= 'z' {
		return true
	}
	if b >= 'A' && b <= 'Z' {
		return true
	}
	if b >= '0' && b <= '9' {
		return true
	}
	return false
}

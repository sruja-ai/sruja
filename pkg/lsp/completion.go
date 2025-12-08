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
	var items []lsp.CompletionItem

	for _, k := range keywordList {
		if token == "" || strings.HasPrefix(k, token) {
			addItem(&items, k, lsp.CIKKeyword)
		}
	}

	if program != nil && program.Architecture != nil {
		arch := program.Architecture
		seen := make(map[string]bool)
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
				add(s.ID + "." + c.ID)
				for _, comp := range c.Components {
					add(comp.ID)
					add(s.ID + "." + c.ID + "." + comp.ID)
				}
				for _, ds := range c.DataStores {
					add(ds.ID)
					add(s.ID + "." + c.ID + "." + ds.ID)
				}
				for _, q := range c.Queues {
					add(q.ID)
					add(s.ID + "." + c.ID + "." + q.ID)
				}
			}
			for _, comp := range s.Components {
				add(comp.ID)
				add(s.ID + "." + comp.ID)
			}
			for _, ds := range s.DataStores {
				add(ds.ID)
				add(s.ID + "." + ds.ID)
			}
			for _, q := range s.Queues {
				add(q.ID)
				add(s.ID + "." + q.ID)
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

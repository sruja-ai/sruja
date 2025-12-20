package lsp

import (
	"context"
	"strings"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/language"
)

var keywordList = []string{
	// Top-level elements
	"specification", "model", "views", "view",
	"element", "system", "component", "container", "datastore", "queue", "person",
	// Extended elements
	"adr", "requirement", "policy",
	// Relationships
	"relationship", "extend", "include", "exclude", "extends",
	// Properties & Fields
	"tag", "tags", "link", "links", "color", "icon", "shape", "style",
	"title", "description", "technology", "tech", "owner",
	"autoLayout", "of", "dynamic",
	// Nested fields for extended elements
	"status", "context", "decision", "consequences", "date",
	"id", "text", "struct", "doc",
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

	if program != nil && program.Model != nil {
		seen := make(map[string]bool, 32)

		add := func(id string) {
			if id == "" || seen[id] {
				return
			}
			if token == "" || strings.HasPrefix(id, token) {
				addItem(&items, id, lsp.CIKText)
				seen[id] = true
			}
		}

		// Collect all element IDs from LikeC4 Model
		var collectIDs func(elem *language.LikeC4ElementDef, parentFQN string)
		collectIDs = func(elem *language.LikeC4ElementDef, parentFQN string) {
			if elem == nil {
				return
			}

			id := elem.GetID()
			if id == "" {
				return
			}

			fqn := id
			if parentFQN != "" {
				fqn = parentFQN + "." + id
			}

			add(id)
			add(fqn)

			// Recurse into nested elements
			body := elem.GetBody()
			if body != nil {
				for _, bodyItem := range body.Items {
					if bodyItem.Element != nil {
						collectIDs(bodyItem.Element, fqn)
					}
				}
			}
		}

		// Process all top-level elements
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				collectIDs(item.ElementDef, "")
			}
			// Also add scenario and flow IDs
			if item.Scenario != nil {
				add(item.Scenario.ID)
			}
			if item.Flow != nil {
				add(item.Flow.ID)
			}
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

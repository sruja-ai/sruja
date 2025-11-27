package lsp

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sourcegraph/jsonrpc2"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

type Handler struct {
	root  string
	index map[string]struct {
		File string
		Line int
	}
	semanticIndex      *SemanticIndex
	metadataRegistry   *MetadataRegistry
	completionProvider *CompletionProvider
}

type initializeResult struct {
	Capabilities map[string]any `json:"capabilities"`
}

func (h *Handler) Handle(ctx context.Context, conn *jsonrpc2.Conn, req *jsonrpc2.Request) {
	switch req.Method {
	case "initialize":
		var p struct {
			RootURI  string `json:"rootUri"`
			RootPath string `json:"rootPath"`
		}
		if req.Params != nil {
			_ = json.Unmarshal(*req.Params, &p)
		}
		if p.RootURI != "" && strings.HasPrefix(p.RootURI, "file:") {
			h.root = strings.TrimPrefix(p.RootURI, "file://")
		} else {
			h.root = p.RootPath
		}
		if h.root != "" {
			h.buildWorkspaceIndex()
		}

		// Initialize LSP infrastructure
		if h.semanticIndex == nil {
			h.semanticIndex = NewSemanticIndex()
		}
		if h.metadataRegistry == nil {
			h.metadataRegistry = NewMetadataRegistry()
		}
		if h.completionProvider == nil {
			h.completionProvider = NewCompletionProvider(h.semanticIndex, h.metadataRegistry)
		}

		res := initializeResult{Capabilities: map[string]any{
			"textDocumentSync":   2,
			"diagnosticProvider": map[string]any{"identifier": "sruja"},
			"completionProvider": map[string]any{
				"resolveProvider":   false,
				"triggerCharacters": []string{":", ".", ">", "{"},
			},
			"hoverProvider":      true,
			"definitionProvider": true,
			"codeActionProvider": map[string]any{
				"codeActionKinds": []string{"quickfix", "refactor", "source"},
			},
		}}
		conn.Reply(ctx, req.ID, res)
		return
	case "textDocument/didOpen":
		var p struct {
			TextDocument struct {
				Uri  string
				Text string
			}
		}
		json.Unmarshal(*req.Params, &p)
		diags := diagnostics(p.TextDocument.Text)
		publishDiagnostics(conn, p.TextDocument.Uri, diags)
		h.writeIndex(p.TextDocument.Uri, p.TextDocument.Text)
		h.updateSemanticIndex(p.TextDocument.Uri, p.TextDocument.Text)
		return
	case "textDocument/didChange":
		var p struct {
			TextDocument   struct{ Uri string }
			ContentChanges []struct{ Text string }
		}
		json.Unmarshal(*req.Params, &p)
		var text string
		if len(p.ContentChanges) > 0 {
			text = p.ContentChanges[0].Text
		}
		diags := diagnostics(text)
		publishDiagnostics(conn, p.TextDocument.Uri, diags)
		h.writeIndex(p.TextDocument.Uri, text)
		h.updateSemanticIndex(p.TextDocument.Uri, text)
		return
	case "textDocument/didSave":
		var p struct{ TextDocument struct{ Uri string } }
		json.Unmarshal(*req.Params, &p)
		return
	case "textDocument/completion":
		var p struct {
			TextDocument struct{ Uri string }
			Position     struct {
				Line      int
				Character int
			}
		}
		json.Unmarshal(*req.Params, &p)

		// Get document text
		text := readFileFromURI(p.TextDocument.Uri)

		// Use intelligent completion provider
		items, err := h.completionProvider.ProvideCompletions(
			p.TextDocument.Uri,
			text,
			p.Position.Line,
			p.Position.Character,
		)
		if err != nil {
			conn.Reply(ctx, req.ID, map[string]any{"isIncomplete": false, "items": []any{}})
			return
		}

		conn.Reply(ctx, req.ID, map[string]any{"isIncomplete": false, "items": items})
		return
	case "textDocument/hover":
		var p struct {
			TextDocument struct{ Uri string }
			Position     struct {
				Line      int
				Character int
			}
		}
		json.Unmarshal(*req.Params, &p)
		text := readFileFromURI(p.TextDocument.Uri)

		// Use enhanced hover provider
		if h.completionProvider == nil {
			// Fallback to basic hover
			id := identifierAt(text, p.Position.Line, p.Position.Character)
			if id == "" {
				conn.Reply(ctx, req.ID, nil)
				return
			}
			if entry, ok := h.index[id]; ok {
				conn.Reply(ctx, req.ID, map[string]any{"contents": map[string]any{"kind": "markdown", "value": fmt.Sprintf("**%s**\nFile: %s\nLine: %d", id, entry.File, entry.Line+1)}})
				return
			}
			conn.Reply(ctx, req.ID, nil)
			return
		}

		// Use enhanced hover provider
		hoverProvider := NewHoverProvider(h.semanticIndex, h.metadataRegistry)
		hoverInfo, err := hoverProvider.ProvideHover(p.TextDocument.Uri, text, p.Position.Line, p.Position.Character)
		if err != nil || hoverInfo == nil {
			conn.Reply(ctx, req.ID, nil)
			return
		}

		conn.Reply(ctx, req.ID, map[string]any{
			"contents": map[string]any{
				"kind":  "markdown",
				"value": hoverInfo.Contents,
			},
		})
		return
	case "textDocument/codeAction":
		var p struct {
			TextDocument struct{ Uri string }
			Range        struct {
				Start struct {
					Line      int
					Character int
				}
				End struct {
					Line      int
					Character int
				}
			}
			Context struct {
				Diagnostics []interface{} `json:"diagnostics"`
				Only        []string      `json:"only,omitempty"`
			}
		}
		json.Unmarshal(*req.Params, &p)
		text := readFileFromURI(p.TextDocument.Uri)

		// Use code action provider
		if h.semanticIndex != nil && h.metadataRegistry != nil {
			codeActionProvider := NewCodeActionProvider(h.semanticIndex, h.metadataRegistry)
			range_ := Range{
				Start: Position{Line: p.Range.Start.Line, Character: p.Range.Start.Character},
				End:   Position{Line: p.Range.End.Line, Character: p.Range.End.Character},
			}
			context := CodeActionContext{
				Diagnostics: p.Context.Diagnostics,
				Only:        p.Context.Only,
			}
			actions, err := codeActionProvider.ProvideCodeActions(p.TextDocument.Uri, text, range_, context)
			if err == nil {
				// Convert to LSP format
				actionList := make([]map[string]any, len(actions))
				for i, action := range actions {
					actionMap := map[string]any{
						"title": action.Title,
						"kind":  action.Kind,
					}
					if action.Edit != nil {
						changes := make(map[string]interface{})
						for uri, edits := range action.Edit.Changes {
							editList := make([]map[string]any, len(edits))
							for j, edit := range edits {
								editList[j] = map[string]any{
									"range": map[string]any{
										"start": map[string]int{"line": edit.Range.Start.Line, "character": edit.Range.Start.Character},
										"end":   map[string]int{"line": edit.Range.End.Line, "character": edit.Range.End.Character},
									},
									"newText": edit.NewText,
								}
							}
							changes[uri] = editList
						}
						actionMap["edit"] = map[string]any{"changes": changes}
					}
					actionList[i] = actionMap
				}
				conn.Reply(ctx, req.ID, actionList)
				return
			}
		}
		conn.Reply(ctx, req.ID, []any{})
		return
	case "textDocument/definition":
		var p struct {
			TextDocument struct{ Uri string }
			Position     struct {
				Line      int
				Character int
			}
		}
		json.Unmarshal(*req.Params, &p)
		text := readFileFromURI(p.TextDocument.Uri)
		id := identifierAt(text, p.Position.Line, p.Position.Character)
		if entry, ok := h.index[id]; ok {
			uri := "file://" + entry.File
			loc := map[string]any{"uri": uri, "range": map[string]any{"start": map[string]int{"line": entry.Line, "character": 0}, "end": map[string]int{"line": entry.Line, "character": 0}}}
			conn.Reply(ctx, req.ID, []any{loc})
			return
		}
		conn.Reply(ctx, req.ID, nil)
		return
	}
	return
}

type diagnostic struct {
	Range struct {
		Start struct {
			Line      int
			Character int
		}
		End struct {
			Line      int
			Character int
		}
	} `json:"range"`
	Severity int    `json:"severity"`
	Source   string `json:"source"`
	Message  string `json:"message"`
}

func diagnostics(text string) []diagnostic {
	var out []diagnostic
	pr, _ := language.NewParser()
	if pr == nil {
		return out
	}

	program, err := pr.Parse("", text)
	if err != nil {
		var lxErr *lexer.Error
		if errors.As(err, &lxErr) {
			pos := lxErr.Position()
			var d diagnostic
			d.Range.Start.Line = pos.Line - 1
			d.Range.Start.Character = pos.Column - 1
			d.Range.End.Line = pos.Line - 1
			d.Range.End.Character = pos.Column
			d.Severity = 1
			d.Source = "sruja"
			d.Message = lxErr.Message()
			out = append(out, d)
		} else {
			var d diagnostic
			d.Range.Start.Line = 0
			d.Range.End.Line = 0
			d.Severity = 1
			d.Source = "sruja"
			d.Message = fmt.Sprintf("parse error: %v", err)
			out = append(out, d)
		}
		return out
	}

	// Run validation if parsing succeeded
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})

	validationErrors := validator.Validate(program)
	for _, ve := range validationErrors {
		var d diagnostic
		d.Range.Start.Line = ve.Line - 1
		if d.Range.Start.Line < 0 {
			d.Range.Start.Line = 0
		}
		d.Range.Start.Character = ve.Column - 1
		if d.Range.Start.Character < 0 {
			d.Range.Start.Character = 0
		}
		d.Range.End.Line = d.Range.Start.Line
		lines := strings.Split(text, "\n")
		if d.Range.Start.Line < len(lines) {
			d.Range.End.Character = len(lines[d.Range.Start.Line])
		} else {
			d.Range.End.Character = d.Range.Start.Character
		}
		d.Severity = 1
		d.Source = "sruja"
		d.Message = ve.Message
		out = append(out, d)
	}

	// Check for unbalanced quotes
	lines := strings.Split(text, "\n")
	for i, l := range lines {
		c := strings.Count(l, "\"")
		if c%2 == 1 {
			var d diagnostic
			d.Range.Start.Line = i
			d.Range.End.Line = i
			d.Severity = 1
			d.Source = "sruja"
			d.Message = "unbalanced quotes"
			out = append(out, d)
		}
	}
	return out
}

func publishDiagnostics(conn *jsonrpc2.Conn, uri string, diags []diagnostic) {
	p := map[string]any{"uri": uri, "diagnostics": diags}
	conn.Notify(context.Background(), "textDocument/publishDiagnostics", p)
}

func (h *Handler) writeIndex(uri string, text string) {
	if !strings.HasPrefix(uri, "file:") {
		return
	}
	path := strings.TrimPrefix(uri, "file://")
	root := filepath.Dir(path)
	for {
		if _, err := os.Stat(filepath.Join(root, ".architecture")); err == nil {
			break
		}
		next := filepath.Dir(root)
		if next == root {
			break
		}
		root = next
	}
	idxDir := filepath.Join(root, ".architecture")
	os.MkdirAll(idxDir, 0o755)
	nodes := map[string]map[string]any{}
	lines := strings.Split(text, "\n")
	for i, l := range lines {
		id, _ := idFromLine(l)
		if id != "" {
			nodes[id] = map[string]any{"line": i, "file": path}
			if h.index == nil {
				h.index = map[string]struct {
					File string
					Line int
				}{}
			}
			h.index[id] = struct {
				File string
				Line int
			}{File: path, Line: i}
		}
	}
	data := map[string]any{"nodes": nodes}
	b, _ := json.MarshalIndent(data, "", "  ")
	os.WriteFile(filepath.Join(idxDir, "index.json"), b, 0o644)
}

func readFileFromURI(uri string) string {
	if !strings.HasPrefix(uri, "file:") {
		return ""
	}
	p := strings.TrimPrefix(uri, "file://")
	b, _ := os.ReadFile(p)
	return string(b)
}

func identifierAt(text string, line, ch int) string {
	lines := strings.Split(text, "\n")
	if line < 0 || line >= len(lines) {
		return ""
	}
	l := lines[line]
	if ch < 0 {
		ch = 0
	}
	if ch > len(l) {
		ch = len(l)
	}
	start := ch
	for start > 0 && (isIdentChar(l[start-1])) {
		start--
	}
	end := ch
	for end < len(l) && isIdentChar(l[end]) {
		end++
	}
	return strings.TrimSpace(l[start:end])
}

func isIdentChar(b byte) bool {
	return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') || (b >= '0' && b <= '9') || b == '_' || b == '.'
}

func (h *Handler) buildWorkspaceIndex() {
	if h.root == "" {
		return
	}
	h.index = map[string]struct {
		File string
		Line int
	}{}

	if h.semanticIndex == nil {
		h.semanticIndex = NewSemanticIndex()
	}

	filepath.Walk(h.root, func(path string, info os.FileInfo, err error) error {
		if err != nil || info == nil || info.IsDir() {
			return nil
		}
		if strings.HasSuffix(path, ".sruja") {
			b, _ := os.ReadFile(path)
			text := string(b)

			// Build legacy index
			lines := strings.Split(text, "\n")
			for i, l := range lines {
				parts := strings.Split(l, ":")
				if len(parts) > 1 {
					id := strings.TrimSpace(parts[0])
					if id != "" {
						h.index[id] = struct {
							File string
							Line int
						}{File: path, Line: i}
					}
				}
			}

			// Build semantic index
			pr, _ := language.NewParser()
			if pr != nil {
				program, err := pr.Parse(path, text)
				if err == nil && program != nil {
					h.semanticIndex.IndexFile(path, program)
				}
			}
		}
		return nil
	})
}

// updateSemanticIndex updates the semantic index when a file changes.
func (h *Handler) updateSemanticIndex(uri string, text string) {
	if h.semanticIndex == nil {
		h.semanticIndex = NewSemanticIndex()
	}

	if !strings.HasPrefix(uri, "file:") {
		return
	}

	filePath := strings.TrimPrefix(uri, "file://")

	pr, _ := language.NewParser()
	if pr == nil {
		return
	}

	// Try to parse (may fail for incomplete code)
	program, err := pr.Parse(filePath, text)
	if err == nil && program != nil {
		h.semanticIndex.IndexFile(filePath, program)
	}
}

// idFromLine extracts a simple identifier from a DSL declaration line.
// Recognizes: system <ID>, container <ID>, component <ID>; also legacy "ID: type" lines.
func idFromLine(line string) (string, string) {
	s := strings.TrimSpace(line)
	if s == "" || strings.HasPrefix(s, "//") || strings.HasPrefix(s, "/*") {
		return "", ""
	}
	parts := strings.Fields(s)
	if len(parts) >= 2 {
		kw := parts[0]
		id := strings.Trim(parts[1], ":{}")
		switch kw {
		case "system", "container", "component":
			if id != "" {
				return id, kw
			}
		}
	}
	// Legacy: "ID: type ..."
	if idx := strings.Index(s, ":"); idx > 0 {
		candidate := strings.TrimSpace(s[:idx])
		if candidate != "" {
			return candidate, "legacy"
		}
	}
	return "", ""
}

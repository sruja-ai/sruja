package lsp

import (
	"strings"
	"sync"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/stdlib"
)

type Document struct {
	URI           lsp.DocumentURI
	Text          string
	Version       int
	lines         []string
	defs          map[string]lsp.Range
	defKinds      map[string]lsp.SymbolKind
	defContainers map[string]string
	program       *language.Program
}

func NewDocument(uri lsp.DocumentURI, text string, version int) *Document {
	d := &Document{URI: uri, Text: text, Version: version}
	d.lines = strings.Split(text, "\n")
	// Estimate capacity: typical DSL files have ~50-200 definitions
	estimatedDefs := len(d.lines) / 10
	if estimatedDefs < 32 {
		estimatedDefs = 32
	}
	d.defs = make(map[string]lsp.Range, estimatedDefs)
	d.defKinds = make(map[string]lsp.SymbolKind, estimatedDefs)
	d.defContainers = make(map[string]string, estimatedDefs/2)
	d.rebuildDefs()
	return d
}

func (d *Document) GetLine(n int) string {
	if n < 0 || n >= len(d.lines) {
		return ""
	}
	return d.lines[n]
}

func (d *Document) SetText(text string) {
	d.Text = text
	d.lines = strings.Split(text, "\n")
	d.rebuildDefs()
	d.program = nil
}

func (d *Document) ApplyChange(change lsp.TextDocumentContentChangeEvent) {
	if change.Range == nil {
		d.SetText(change.Text)
		return
	}
	// Basic incremental apply by positions
	start := *change.Range
	// Compute offsets by summing line lengths
	var offsetStart, offsetEnd int
	for i := 0; i < start.Start.Line; i++ {
		offsetStart += len(d.GetLine(i)) + 1
	}
	offsetStart += start.Start.Character
	for i := 0; i < start.End.Line; i++ {
		offsetEnd += len(d.GetLine(i)) + 1
	}
	offsetEnd += start.End.Character

	if offsetStart < 0 {
		offsetStart = 0
	}
	if offsetEnd > len(d.Text) {
		offsetEnd = len(d.Text)
	}

	// Use strings.Builder for better performance with large texts
	var sb strings.Builder
	sb.Grow(len(d.Text) - (offsetEnd - offsetStart) + len(change.Text))
	sb.WriteString(d.Text[:offsetStart])
	sb.WriteString(change.Text)
	sb.WriteString(d.Text[offsetEnd:])
	d.Text = sb.String()
	d.lines = strings.Split(d.Text, "\n")
	d.rebuildDefs()
	d.program = nil
}

// buildQualifiedIDForWorkspace builds a qualified ID efficiently.
func buildQualifiedIDForWorkspace(parts ...string) string {
	// Filter empty parts
	nonEmpty := make([]string, 0, len(parts))
	for _, p := range parts {
		if p != "" {
			nonEmpty = append(nonEmpty, p)
		}
	}
	if len(nonEmpty) == 0 {
		return ""
	}
	if len(nonEmpty) == 1 {
		return nonEmpty[0]
	}
	totalLen := len(nonEmpty) - 1 // for dots
	for _, p := range nonEmpty {
		totalLen += len(p)
	}
	buf := make([]byte, 0, totalLen)
	buf = append(buf, nonEmpty[0]...)
	for i := 1; i < len(nonEmpty); i++ {
		buf = append(buf, '.')
		buf = append(buf, nonEmpty[i]...)
	}
	return string(buf)
}

//nolint:funlen,gocyclo // Logic requires length and is complex
func (d *Document) rebuildDefs() {
	// Estimate capacity based on document size
	estimatedDefs := len(d.lines) / 10
	if estimatedDefs < 32 {
		estimatedDefs = 32
	}
	d.defs = make(map[string]lsp.Range, estimatedDefs)
	d.defKinds = make(map[string]lsp.SymbolKind, estimatedDefs)
	d.defContainers = make(map[string]string, estimatedDefs/2)
	type ctx struct{ kind, id string }
	// Estimate stack depth: typically 2-5 levels (system -> container -> component)
	stack := make([]ctx, 0, 8)
	currentSystem := ""
	currentContainer := ""

	extractID := func(trimmed, keyword string) (string, bool) {
		// Try simple prefix: keyword ID ...
		prefix := keyword + " "
		if strings.HasPrefix(trimmed, prefix) {
			rest := strings.TrimSpace(strings.TrimPrefix(trimmed, prefix))
			if rest != "" {
				end := 0
				for end < len(rest) {
					c := rest[end]
					if c == ' ' || c == '"' || c == '{' {
						break
					}
					end++
				}
				return rest[:end], true
			}
		}

		// Try assignment: ID = keyword ...
		eqIdx := strings.Index(trimmed, "=")
		if eqIdx != -1 {
			left := strings.TrimSpace(trimmed[:eqIdx])
			right := strings.TrimSpace(trimmed[eqIdx+1:])
			if strings.HasPrefix(right, keyword) {
				// Check boundary
				if len(right) == len(keyword) || right[len(keyword)] == ' ' || right[len(keyword)] == '"' || right[len(keyword)] == '{' {
					return left, true
				}
			}
		}

		return "", false
	}

	for i, line := range d.lines {
		trimmed := strings.TrimSpace(line)
		// System
		id, ok := extractID(trimmed, "system")
		if !ok {
			id, ok = extractID(trimmed, "System")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKClass
			d.defContainers[id] = ""
			currentSystem = id
			if strings.Contains(trimmed, "{") {
				stack = append(stack, ctx{kind: "system", id: id})
			}
			continue
		}
		// Container
		id, ok = extractID(trimmed, "container")
		if !ok {
			id, ok = extractID(trimmed, "Container")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKModule
			d.defContainers[id] = currentSystem
			if currentSystem != "" {
				q := buildQualifiedIDForWorkspace(currentSystem, "", id)
				d.defs[q] = r
				d.defKinds[q] = lsp.SKModule
				d.defContainers[q] = currentSystem
			}
			currentContainer = id
			if strings.Contains(trimmed, "{") {
				stack = append(stack, ctx{kind: "container", id: id})
			}
			continue
		}
		// Component
		id, ok = extractID(trimmed, "component")
		if !ok {
			id, ok = extractID(trimmed, "Component")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKFunction
			d.defContainers[id] = currentSystem
			if currentSystem != "" && currentContainer != "" {
				q := buildQualifiedIDForWorkspace(currentSystem, currentContainer, id)
				d.defs[q] = r
				d.defKinds[q] = lsp.SKFunction
				d.defContainers[q] = buildQualifiedIDForWorkspace(currentSystem, currentContainer, "")
			} else if currentSystem != "" {
				q := buildQualifiedIDForWorkspace(currentSystem, "", id)
				d.defs[q] = r
				d.defKinds[q] = lsp.SKFunction
				d.defContainers[q] = currentSystem
			}
			if strings.Contains(trimmed, "{") {
				stack = append(stack, ctx{kind: "component", id: id})
			}
			continue
		}
		// DataStore
		id, ok = extractID(trimmed, "datastore")
		if !ok {
			id, ok = extractID(trimmed, "DataStore")
		}
		if !ok {
			id, ok = extractID(trimmed, "database")
		}
		if !ok {
			id, ok = extractID(trimmed, "Database")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKStruct
			d.defContainers[id] = currentSystem
			if currentSystem != "" {
				q := buildQualifiedIDForWorkspace(currentSystem, "", id)
				d.defs[q] = r
				d.defKinds[q] = lsp.SKStruct
				d.defContainers[q] = currentSystem
			}
			continue
		}
		// Queue
		id, ok = extractID(trimmed, "queue")
		if !ok {
			id, ok = extractID(trimmed, "Queue")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKEnum
			d.defContainers[id] = currentSystem
			if currentSystem != "" {
				q := buildQualifiedIDForWorkspace(currentSystem, "", id)
				d.defs[q] = r
				d.defKinds[q] = lsp.SKEnum
				d.defContainers[q] = currentSystem
			}
			continue
		}
		// Person
		id, ok = extractID(trimmed, "person")
		if !ok {
			id, ok = extractID(trimmed, "Person")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKVariable
			d.defContainers[id] = ""
			continue
		}
		// ADR
		id, ok = extractID(trimmed, "adr")
		if !ok {
			id, ok = extractID(trimmed, "Adr")
		}
		if !ok {
			id, ok = extractID(trimmed, "ADR")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKString
			d.defContainers[id] = ""
			continue
		}
		// Requirement
		id, ok = extractID(trimmed, "requirement")
		if !ok {
			id, ok = extractID(trimmed, "Requirement")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKProperty
			d.defContainers[id] = ""
			continue
		}
		// Policy
		id, ok = extractID(trimmed, "policy")
		if !ok {
			id, ok = extractID(trimmed, "Policy")
		}
		if ok {
			col := strings.Index(line, id)
			if col < 0 {
				col = 0
			}
			r := lsp.Range{Start: lsp.Position{Line: i, Character: col}, End: lsp.Position{Line: i, Character: col + len(id)}}
			d.defs[id] = r
			d.defKinds[id] = lsp.SKConstant
			d.defContainers[id] = ""
			continue
		}
		// Handle closing braces to maintain context
		if strings.Contains(trimmed, "}") {
			// Pop for each closing brace
			closeCount := strings.Count(trimmed, "}")
			for j := 0; j < closeCount && len(stack) > 0; j++ {
				top := stack[len(stack)-1]
				stack = stack[:len(stack)-1]
				if top.kind == "container" {
					currentContainer = ""
					// Find previous container if nested (unlikely);
					for k := len(stack) - 1; k >= 0; k-- {
						if stack[k].kind == "container" {
							currentContainer = stack[k].id
							break
						}
					}
				}
				if top.kind == "system" {
					currentSystem = ""
					for k := len(stack) - 1; k >= 0; k-- {
						if stack[k].kind == "system" {
							currentSystem = stack[k].id
							break
						}
					}
				}
			}
		}
	}
}

// EnsureParsed parses the document text and caches the program for reuse.
// It also loads stdlib and merges imports so that imported kinds are available
// for LSP features like completion and validation.
func (d *Document) EnsureParsed() *language.Program {
	if d.program != nil {
		return d.program
	}
	p, err := language.NewParser()
	if err != nil {
		return nil
	}
	program, _, err := p.Parse(string(d.URI), d.Text)
	if err != nil {
		return nil
	}

	// Create a workspace to merge imports (including stdlib)
	ws := language.NewWorkspace()
	ws.AddProgram(string(d.URI), program, nil)

	// Load stdlib into workspace for import resolution
	loadStdLibIntoWorkspace(p, ws)

	// Merge imports from stdlib into the program
	ws.ResolveAndMergeImports()

	// Resolve references so LSP features work on canonical IDs
	engine.RunResolution(program)

	d.program = program
	return program
}

// loadStdLibIntoWorkspace loads the embedded stdlib files into a workspace.
func loadStdLibIntoWorkspace(p *language.Parser, ws *language.Workspace) {
	// Load core.sruja from embedded stdlib
	stdlibFiles := []string{"core.sruja", "styles.sruja"}
	for _, filename := range stdlibFiles {
		content, err := stdlib.FS.ReadFile(filename)
		if err != nil {
			continue
		}
		path := "sruja.ai/stdlib/" + filename
		if _, exists := ws.Programs[path]; !exists {
			prog, diags, err := p.Parse(path, string(content))
			if err == nil {
				ws.AddProgram(path, prog, diags)
			}
		}
	}
}

type Workspace struct {
	docs map[lsp.DocumentURI]*Document
	mu   sync.RWMutex
}

func NewWorkspace() *Workspace {
	return &Workspace{docs: make(map[lsp.DocumentURI]*Document)}
}

func (w *Workspace) AddDocument(uri lsp.DocumentURI, text string, version int) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.docs[uri] = NewDocument(uri, text, version)
}

func (w *Workspace) RemoveDocument(uri lsp.DocumentURI) {
	w.mu.Lock()
	defer w.mu.Unlock()
	delete(w.docs, uri)
}

func (w *Workspace) GetDocument(uri lsp.DocumentURI) *Document {
	w.mu.RLock()
	defer w.mu.RUnlock()
	return w.docs[uri]
}

func (w *Workspace) AllDocuments() []*Document {
	w.mu.RLock()
	defer w.mu.RUnlock()
	res := make([]*Document, 0, len(w.docs))
	for _, d := range w.docs {
		res = append(res, d)
	}
	return res
}

func (w *Workspace) FindDefinition(id string) (lsp.DocumentURI, lsp.Range, bool) {
	// Try exact match (including qualified form)
	w.mu.RLock()
	defer w.mu.RUnlock()
	for _, d := range w.docs {
		if r, ok := d.defs[id]; ok {
			return d.URI, r, true
		}
	}
	// Try by last segment for qualified tokens
	if idx := strings.LastIndex(id, "."); idx > 0 {
		last := id[idx+1:]
		for _, d := range w.docs {
			if r, ok := d.defs[last]; ok {
				return d.URI, r, true
			}
		}
	}
	return "", lsp.Range{}, false
}

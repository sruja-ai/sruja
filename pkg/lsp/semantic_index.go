// pkg/lsp/semantic_index.go
package lsp

import (
	"fmt"
	"path/filepath"
	"strings"
	"sync"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ElementReference represents a reference to an element in the semantic model.
//
// Used for cross-file references, go-to-definition, and autocomplete.
type ElementReference struct {
	ID          string // Element ID (e.g., "BillingAPI")
	Type        string // Element type: "system", "container", "component", "person", "datastore", "queue"
	Label       string // Element label
	File        string // File path where this element is defined
	Line        int    // 1-based line number
	Column      int    // 1-based column number
	QualifiedID string // Fully qualified ID (e.g., "Billing.Arch.InventoryAPI" for imported elements)
	ArchName    string // Architecture name this element belongs to
}

// SemanticIndex maintains an index of all elements across the workspace.
//
// The index allows fast lookup of:
// - Elements by ID (for autocomplete and go-to-definition)
// - Elements by type (for context-aware suggestions)
// - Elements by architecture (for qualified references)
// - Import relationships (for qualified reference completion)
type SemanticIndex struct {
	mu sync.RWMutex

	// Element lookup: ID -> ElementReference
	elements map[string]*ElementReference

	// Type index: type -> []ElementReference
	byType map[string][]*ElementReference

	// Architecture index: architecture name -> []ElementReference
	byArch map[string][]*ElementReference

	// Import index: file -> []ImportSpec
	imports map[string][]*language.ImportSpec

	// Imported architectures: alias -> architecture name
	importedArchs map[string]string

	// File -> Architecture mapping
	fileToArch map[string]string
}

// NewSemanticIndex creates a new semantic index.
func NewSemanticIndex() *SemanticIndex {
	return &SemanticIndex{
		elements:      make(map[string]*ElementReference),
		byType:        make(map[string][]*ElementReference),
		byArch:        make(map[string][]*ElementReference),
		imports:       make(map[string][]*language.ImportSpec),
		importedArchs: make(map[string]string),
		fileToArch:    make(map[string]string),
	}
}

// IndexFile indexes all elements in a parsed program.
//
// This should be called whenever a file is parsed (on open, change, or save).
func (idx *SemanticIndex) IndexFile(filePath string, program *language.Program) {
	idx.mu.Lock()
	defer idx.mu.Unlock()

	if program == nil || program.Architecture == nil {
		return
	}

	arch := program.Architecture
	archName := arch.Name

	// Store architecture name for this file
	idx.fileToArch[filePath] = archName

	// Remove old elements from this file
	idx.removeFile(filePath)

	// Index systems
	for _, sys := range arch.Systems {
		ref := &ElementReference{
			ID:          sys.ID,
			Type:        "system",
			Label:       sys.Label,
			File:        filePath,
			Line:        1, // TODO: get actual line from AST
			Column:      1,
			QualifiedID: sys.ID,
			ArchName:    archName,
		}
		idx.addElement(ref)

		// Index containers within system
		for _, cont := range sys.Containers {
			ref := &ElementReference{
				ID:          cont.ID,
				Type:        "container",
				Label:       cont.Label,
				File:        filePath,
				Line:        1,
				Column:      1,
				QualifiedID: sys.ID + "." + cont.ID,
				ArchName:    archName,
			}
			idx.addElement(ref)

			// Index components within container
			for _, comp := range cont.Components {
				ref := &ElementReference{
					ID:          comp.ID,
					Type:        "component",
					Label:       comp.Label,
					File:        filePath,
					Line:        1,
					Column:      1,
					QualifiedID: sys.ID + "." + cont.ID + "." + comp.ID,
					ArchName:    archName,
				}
				idx.addElement(ref)
			}
		}

		// Index components directly in system
		for _, comp := range sys.Components {
			ref := &ElementReference{
				ID:          comp.ID,
				Type:        "component",
				Label:       comp.Label,
				File:        filePath,
				Line:        1,
				Column:      1,
				QualifiedID: sys.ID + "." + comp.ID,
				ArchName:    archName,
			}
			idx.addElement(ref)
		}

		// Index data stores
		for _, ds := range sys.DataStores {
			ref := &ElementReference{
				ID:          ds.ID,
				Type:        "datastore",
				Label:       ds.Label,
				File:        filePath,
				Line:        1,
				Column:      1,
				QualifiedID: sys.ID + "." + ds.ID,
				ArchName:    archName,
			}
			idx.addElement(ref)
		}

		// Index queues
		for _, q := range sys.Queues {
			ref := &ElementReference{
				ID:          q.ID,
				Type:        "queue",
				Label:       q.Label,
				File:        filePath,
				Line:        1,
				Column:      1,
				QualifiedID: sys.ID + "." + q.ID,
				ArchName:    archName,
			}
			idx.addElement(ref)
		}
	}

	// Index persons
	for _, person := range arch.Persons {
		ref := &ElementReference{
			ID:          person.ID,
			Type:        "person",
			Label:       person.Label,
			File:        filePath,
			Line:        1,
			Column:      1,
			QualifiedID: person.ID,
			ArchName:    archName,
		}
		idx.addElement(ref)
	}

	// Index imports
	if len(arch.Imports) > 0 {
		idx.imports[filePath] = arch.Imports
		for _, imp := range arch.Imports {
			if imp.Alias != nil && *imp.Alias != "" {
				// TODO: Resolve imported architecture name
				idx.importedArchs[*imp.Alias] = imp.Path
			}
		}
	}
}

// removeFile removes all elements from a file from the index.
func (idx *SemanticIndex) removeFile(filePath string) {
	// Remove from elements map
	for id, ref := range idx.elements {
		if ref.File == filePath {
			delete(idx.elements, id)
		}
	}

	// Remove from type index
	for typ := range idx.byType {
		filtered := idx.byType[typ][:0]
		for _, ref := range idx.byType[typ] {
			if ref.File != filePath {
				filtered = append(filtered, ref)
			}
		}
		idx.byType[typ] = filtered
	}

	// Remove from architecture index
	for arch := range idx.byArch {
		filtered := idx.byArch[arch][:0]
		for _, ref := range idx.byArch[arch] {
			if ref.File != filePath {
				filtered = append(filtered, ref)
			}
		}
		idx.byArch[arch] = filtered
	}

	// Remove imports
	delete(idx.imports, filePath)

	// Remove file-to-arch mapping
	delete(idx.fileToArch, filePath)
}

// addElement adds an element reference to the index.
func (idx *SemanticIndex) addElement(ref *ElementReference) {
	idx.elements[ref.ID] = ref
	idx.byType[ref.Type] = append(idx.byType[ref.Type], ref)
	idx.byArch[ref.ArchName] = append(idx.byArch[ref.ArchName], ref)

	// Also add qualified ID
	if ref.QualifiedID != ref.ID {
		idx.elements[ref.QualifiedID] = ref
	}
}

// GetElement returns an element reference by ID.
func (idx *SemanticIndex) GetElement(id string) (*ElementReference, bool) {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	ref, ok := idx.elements[id]
	return ref, ok
}

// GetByType returns all elements of a given type.
//
// Type can be: "system", "container", "component", "person", "datastore", "queue"
func (idx *SemanticIndex) GetByType(typ string) []*ElementReference {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	return idx.byType[typ]
}

// GetByPrefix returns all elements whose ID starts with the given prefix.
//
// Useful for filtering autocomplete suggestions as the user types.
func (idx *SemanticIndex) GetByPrefix(prefix string) []*ElementReference {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	var result []*ElementReference
	prefixLower := strings.ToLower(prefix)
	for id, ref := range idx.elements {
		if strings.HasPrefix(strings.ToLower(id), prefixLower) {
			result = append(result, ref)
		}
	}
	return result
}

// GetByArchitecture returns all elements in a given architecture.
func (idx *SemanticIndex) GetByArchitecture(archName string) []*ElementReference {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	return idx.byArch[archName]
}

// GetQualifiedElements returns elements that match a qualified ID pattern.
//
// Example: "Billing." returns all elements from the "Billing" imported architecture.
func (idx *SemanticIndex) GetQualifiedElements(prefix string) []*ElementReference {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	var result []*ElementReference
	prefixLower := strings.ToLower(prefix)

	for _, ref := range idx.elements {
		qualified := strings.ToLower(ref.QualifiedID)
		if strings.HasPrefix(qualified, prefixLower) {
			// Check if this is a qualified reference match
			parts := strings.Split(qualified, ".")
			if len(parts) > 1 && strings.Join(parts[:len(parts)-1], ".") == strings.TrimSuffix(prefixLower, ".") {
				result = append(result, ref)
			}
		}
	}

	return result
}

// GetImportAliases returns all import aliases in a file.
func (idx *SemanticIndex) GetImportAliases(filePath string) []string {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	imports, ok := idx.imports[filePath]
	if !ok {
		return nil
	}

	var aliases []string
	for _, imp := range imports {
		if imp.Alias != nil && *imp.Alias != "" {
			aliases = append(aliases, *imp.Alias)
		}
	}
	return aliases
}

// ResolveQualifiedID resolves a qualified ID like "Billing.API" to an element reference.
//
// Returns the element reference and whether it was found.
func (idx *SemanticIndex) ResolveQualifiedID(qualifiedID string, filePath string) (*ElementReference, bool) {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	// Try direct lookup
	if ref, ok := idx.elements[qualifiedID]; ok {
		return ref, true
	}

	// Try resolving via imports in the file
	imports, ok := idx.imports[filePath]
	if !ok {
		return nil, false
	}

	parts := strings.Split(qualifiedID, ".")
	if len(parts) < 2 {
		return nil, false
	}

	alias := parts[0]
	elementID := strings.Join(parts[1:], ".")

	// Find import with matching alias
	var importPath string
	for _, imp := range imports {
		if imp.Alias != nil && *imp.Alias == alias {
			importPath = imp.Path
			break
		}
	}

	if importPath == "" {
		return nil, false
	}

	// Resolve imported file path
	baseDir := filepath.Dir(filePath)
	resolvedPath := filepath.Join(baseDir, importPath)

	// Find architecture for imported file
	importedArch, ok := idx.fileToArch[resolvedPath]
	if !ok {
		return nil, false
	}

	// Look for element in imported architecture
	for _, ref := range idx.byArch[importedArch] {
		if ref.ID == elementID || ref.QualifiedID == elementID {
			return ref, true
		}
	}

	return nil, false
}

// ToCompletionItem converts an element reference to an LSP completion item.
func (ref *ElementReference) ToCompletionItem() CompletionItem {
	kind := 6 // Default to Variable
	switch ref.Type {
	case "system":
		kind = 23 // Class
	case "container":
		kind = 22 // Interface
	case "component":
		kind = 21 // Function
	case "person":
		kind = 13 // User
	case "datastore":
		kind = 10 // File
	case "queue":
		kind = 18 // Event
	}

	return CompletionItem{
		Label:         ref.ID,
		Kind:          kind,
		Detail:        fmt.Sprintf("%s: %s", ref.Type, ref.Label),
		Documentation: fmt.Sprintf("%s in %s", ref.Type, filepath.Base(ref.File)),
		InsertText:    ref.ID,
	}
}

// GetADRs returns all ADR IDs from the semantic index.
func (idx *SemanticIndex) GetADRs() []string {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	// ADRs are stored as elements with type "adr"
	adrs := idx.byType["adr"]
	result := make([]string, 0, len(adrs))
	for _, adr := range adrs {
		result = append(result, adr.ID)
	}
	return result
}

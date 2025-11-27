// pkg/kernel/symbol_table.go
// Package kernel provides symbol table for LSP features.
package kernel

import (
	"sync"

	"github.com/sruja-ai/sruja/pkg/model"
)

// SymbolKind represents the type of a symbol.
type SymbolKind string

const (
	SymbolKindSystem      SymbolKind = "System"
	SymbolKindContainer   SymbolKind = "Container"
	SymbolKindComponent   SymbolKind = "Component"
	SymbolKindEntity      SymbolKind = "Entity"
	SymbolKindEvent       SymbolKind = "Event"
	SymbolKindAPI         SymbolKind = "API"
	SymbolKindContract    SymbolKind = "Contract"
	SymbolKindPolicy      SymbolKind = "Policy"
	SymbolKindRule        SymbolKind = "Rule"
	SymbolKindRelation    SymbolKind = "Relation"
	SymbolKindRequirement SymbolKind = "Requirement"
	SymbolKindADR         SymbolKind = "ADR"
)

// SymbolEntry represents an entry in the symbol table.
type SymbolEntry struct {
	ID       string         `json:"id"`
	Kind     SymbolKind     `json:"kind"`
	Name     string         `json:"name"`
	Location model.Location `json:"location"`
	// References tracks where this symbol is referenced
	References []SymbolReference `json:"references,omitempty"`
}

// SymbolReference represents a reference to a symbol.
type SymbolReference struct {
	File   string `json:"file"`
	Line   int    `json:"line"`
	Column int    `json:"column"`
}

// SymbolTable maintains a registry of all defined symbols.
//
// Used for:
//   - Go-to-definition
//   - Rename symbol
//   - Hovers & autocompletion
//   - Cross-references
//   - Semantic linking
type SymbolTable struct {
	mu      sync.RWMutex
	symbols map[string]*SymbolEntry
}

// NewSymbolTable creates a new empty symbol table.
func NewSymbolTable() *SymbolTable {
	return &SymbolTable{
		symbols: make(map[string]*SymbolEntry),
	}
}

// AddSymbol adds or updates a symbol in the table.
func (st *SymbolTable) AddSymbol(id string, kind SymbolKind, name string, location model.Location) {
	st.mu.Lock()
	defer st.mu.Unlock()

	entry := &SymbolEntry{
		ID:       id,
		Kind:     kind,
		Name:     name,
		Location: location,
	}

	// Preserve existing references if updating
	if existing, ok := st.symbols[id]; ok {
		entry.References = existing.References
	}

	st.symbols[id] = entry
}

// GetSymbol retrieves a symbol by ID.
func (st *SymbolTable) GetSymbol(id string) (*SymbolEntry, bool) {
	st.mu.RLock()
	defer st.mu.RUnlock()
	symbol, ok := st.symbols[id]
	return symbol, ok
}

// AddReference adds a reference to a symbol.
func (st *SymbolTable) AddReference(symbolID string, ref SymbolReference) {
	st.mu.Lock()
	defer st.mu.Unlock()

	if entry, ok := st.symbols[symbolID]; ok {
		entry.References = append(entry.References, ref)
	}
}

// FindSymbolsByKind returns all symbols of a given kind.
func (st *SymbolTable) FindSymbolsByKind(kind SymbolKind) []*SymbolEntry {
	st.mu.RLock()
	defer st.mu.RUnlock()

	var results []*SymbolEntry
	for _, entry := range st.symbols {
		if entry.Kind == kind {
			results = append(results, entry)
		}
	}
	return results
}

// RemoveSymbolsByFile removes all symbols defined in a specific file (cell).
func (st *SymbolTable) RemoveSymbolsByFile(file string) {
	st.mu.Lock()
	defer st.mu.Unlock()

	var toRemove []string
	for id, entry := range st.symbols {
		if entry.Location.File == file {
			toRemove = append(toRemove, id)
		}
	}

	for _, id := range toRemove {
		delete(st.symbols, id)
	}
}

// Clear removes all symbols from the table.
func (st *SymbolTable) Clear() {
	st.mu.Lock()
	defer st.mu.Unlock()
	st.symbols = make(map[string]*SymbolEntry)
}

// GetAllSymbols returns all symbols (for debugging/introspection).
func (st *SymbolTable) GetAllSymbols() map[string]*SymbolEntry {
	st.mu.RLock()
	defer st.mu.RUnlock()

	// Return a copy
	result := make(map[string]*SymbolEntry)
	for id, entry := range st.symbols {
		result[id] = entry
	}
	return result
}
